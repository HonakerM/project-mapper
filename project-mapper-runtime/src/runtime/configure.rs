use core::time;
use std::any::type_name_of_val;
use std::collections::{HashMap, HashSet};
use std::path::Path;
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
    component_helper: Box<dyn ComponentLookupHelper>,
) -> Result<()> {
    let graph = RuntimeConfigGraph::new(config)?;

    // pause all unused components
    for unused_config in &graph.unused_nodes {
        if let Some(unused_comp) = component_helper.get_comp(&unused_config.uid()) {
            let mut mut_unused_comp = unused_comp.borrow_mut();
            mut_unused_comp.pause();
        }
    }


    // For each component in order
    for comp_config in graph.bfs_traverse() {
        // get the raw component
        let comp = component_helper.get_comp(&comp_config.uid()).ok_or(anyhow!("Tried to configure components that don't exist"))?;
        let mut local_comp = comp.borrow_mut();

        // perform normal update
        local_comp.update(comp_config.as_ref());

        if let Some(output_comp) = local_comp.output_element() {
            
        }
        // get new downstream components
        let mut expected_upstreams = graph.get_downstream_components(comp_config.uid());
        
    }
    Ok(())
}


fn update_components(
    old_config: &RuntimeConfig,
    new_config: &RuntimeConfig,
    pipeline: &gst::Pipeline,
    component_helper: Box<dyn ComponentLookupHelper>,
) {

}