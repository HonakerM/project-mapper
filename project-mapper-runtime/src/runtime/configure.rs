use core::time;
use std::any::type_name_of_val;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::thread;

use anyhow::{Context, Result, anyhow};
use gst::{DebugGraphDetails, StateChangeSuccess, prelude::*};
use project_mapper_core::runtime_config::output::OutputComponentConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};
use project_mapper_core::runtime_config::utils::changes::RuntimeConfigChangeTracker;
use project_mapper_core::runtime_config::utils::graph::RuntimeConfigGraph;
use project_mapper_core::runtime_config::{RuntimeConfig, output};

use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{ComponentFactory, ComponentLookupHelper};
use crate::receivers::receiver::start_receiver;
use crate::types::message::RuntimeMessage;
use log::{info, warn};

pub fn configure_components(
    config: &RuntimeConfig,
    pipeline: &gst::Pipeline,
    component_helper: &mut Box<dyn ComponentLookupHelper>,
    component_factory: &Box<dyn ComponentFactory>,
    message_sender: mpsc::Sender<RuntimeMessage>,
) -> Result<()> {
    let graph = RuntimeConfigGraph::new(config)?;
    internal_configure_components(
        graph,
        pipeline,
        component_helper,
        component_factory,
        message_sender.clone(),
    )?;
    Ok(())
}

pub fn update_components(
    change_tracker: RuntimeConfigChangeTracker,
    pipeline: &gst::Pipeline,
    component_helper: &mut Box<dyn ComponentLookupHelper>,
    component_factory: &Box<dyn ComponentFactory>,
    message_sender: mpsc::Sender<RuntimeMessage>,
) -> Result<()> {
    // if the component helper contains the default component then remove it before creating
    // or updating components. This ensures if we add a component that could be main the default
    // runtime doesn't affect it
    if component_helper.contains_comp(&DefaultRuntimeComponent::get_default_uid()) {
        component_helper.destroy_comp(&DefaultRuntimeComponent::get_default_uid())?;
    }

    // pause the deleted components
    for deleted_id in &change_tracker.deletes {
        component_helper.pause_comp(deleted_id);
    }

    // configure the components
    internal_configure_components(
        change_tracker.graph,
        pipeline,
        component_helper,
        component_factory,
        message_sender.clone(),
    )?;

    // destroy the deleted components
    for deleted_id in &change_tracker.deletes {
        component_helper.destroy_comp(&deleted_id);
    }

    // if there is no component that requires main then add the default runtime component.
    // this keeps the logic the same
    if !component_helper.has_main_requirement() {
        let default_config = DefaultRuntimeComponent::new_config()
            .context("Failed to contstruct default runtime component config")?;
        component_helper
            .new(&default_config, component_factory.as_ref())
            .context(format!("failed to create default runtime component"))?;
        component_helper
            .setup(default_config.uid(), pipeline, message_sender.clone())
            .context(format!("failed to create default runtime component"))?;
    }

    Ok(())
}

fn internal_configure_components(
    graph: RuntimeConfigGraph,
    pipeline: &gst::Pipeline,
    component_helper: &mut Box<dyn ComponentLookupHelper>,
    component_factory: &Box<dyn ComponentFactory>,
    message_sender: mpsc::Sender<RuntimeMessage>,
) -> Result<()> {
    // pause all unused components
    for unused_config in &graph.unused_nodes {
        if let Some(unused_comp) = component_helper.get_comp(&unused_config.uid()) {
            let mut mut_unused_comp = unused_comp.borrow_mut();
            mut_unused_comp.pause();
        }
    }

    // construct all compontns that don't exist
    let mut comps_to_setup = vec![];
    for comp_config in graph.traverse() {
        // get the component or construct it if needed
        if let None = component_helper.get_comp(&comp_config.uid()) {
            info!("Constructing component {:?}", comp_config.uid());
            component_helper.new(comp_config.as_ref(), component_factory.as_ref())?;
            comps_to_setup.push(comp_config.uid());
        }
    }
    for uid in comps_to_setup {
        // get the component or construct it if needed
        info!("Setup component {:?}", uid);
        component_helper.setup(uid, pipeline, message_sender.clone())?;
    }

    // For each component in order
    for comp_config in graph.traverse() {
        // get the component or construct it if needed
        let comp = if let Some(comp) = component_helper.get_comp(&comp_config.uid()) {
            comp
        } else {
            component_helper.new(comp_config.as_ref(), component_factory.as_ref())?;
            component_helper.setup(comp_config.uid(), pipeline, message_sender.clone())?;
            component_helper.get_comp(&comp_config.uid()).unwrap()
        };
        let mut local_comp = comp.borrow_mut();

        // perform normal update
        info!("Update component {:?}", comp_config.uid());
        local_comp.update(comp_config.as_ref())?;

        if let Some(output_comp) = local_comp.output_element() {
            // get the set of names we're expecting to be linked up to
            let mut expected_upstreams = HashSet::new();
            let mut name_to_uid = HashMap::new();
            for expected_cfg in graph.get_downstream_components(comp_config.uid()) {
                if let Some(expected_comp_ref) = component_helper.get_comp(&expected_cfg.uid()) {
                    let expected_comp = expected_comp_ref.borrow_mut();
                    let name = expected_comp.input_element().unwrap().name();
                    expected_upstreams.insert(name.clone());
                    name_to_uid.insert(name, expected_cfg.uid());
                }
            }

            // track outputs that we have already linked. If there are any extra then remove them
            let (sender, recv) = mpsc::channel();
            let mut total_probes = 0;
            for output_pad in output_comp.src_pads() {
                if let Some(linked_input_pad) = output_pad.peer() {
                    let linked_element_name = linked_input_pad.parent_element().unwrap().name();
                    // if the link name wasn't in the expected set then remove it
                    if !expected_upstreams.remove(&linked_element_name) {
                        let local_sender = sender.clone();
                        output_pad.add_probe(gst::PadProbeType::IDLE, move |pad, info| {
                            pad.unlink(&linked_input_pad).unwrap();
                            local_sender.send(true).unwrap();
                            gst::PadProbeReturn::Remove
                        });
                        total_probes += 1;
                    }
                }
            }
            // wait for all probes to complete
            while total_probes > 0 {
                recv.recv()?;
                total_probes -= 1;
            }

            // link components we haven't yet
            for comp_name in expected_upstreams {
                if let Some(comp_uid) = name_to_uid.get(&comp_name) {
                    if let Some(expected_comp_ref) = component_helper.get_comp(&comp_uid) {
                        let expected_comp = expected_comp_ref.borrow_mut();
                        output_comp.link(expected_comp.input_element().unwrap())?;
                        info!("Linking comp {:?} to {:?}", comp_config.uid(), comp_uid);
                    }
                }
            }
        }
    }
    Ok(())
}
