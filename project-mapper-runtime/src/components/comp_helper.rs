use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::mpsc,
};

use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::{
    components::{
        factory::create_default_component,
        shared::{Component, ComponentFactory, ComponentLookupHelper},
    },
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};

pub struct DefaultComponentHelper {
    main_comp_id: Option<Uid>,
    component_map: HashMap<Uid, Rc<RefCell<Box<dyn Component>>>>,
    setup_tracker: Rc<RefCell<HashSet<Uid>>>,
}

impl DefaultComponentHelper {
    pub fn new() -> Self {
        Self {
            main_comp_id: None,
            component_map: HashMap::new(),
            setup_tracker: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

impl ComponentLookupHelper for DefaultComponentHelper {
    fn create_and_insert_comp(
        &mut self,
        config: &dyn ComponentConfig,
        factory: &dyn ComponentFactory,
    ) -> Result<()> {
        // create the component
        let comp = factory.create_component(config)?;

        // if this component requires main then update the main_id
        if comp.requires_main() {
            if let Some(_) = self.main_comp_id {
                return Err(Error::msg(
                    "Component that requires main already exists. Can not have two main components",
                ));
            }
            self.main_comp_id = Some(comp.uid());
        }

        // add the comp to the map
        let comp_id = comp.uid();
        let rc_comp = Rc::new(RefCell::new(comp));
        self.component_map.insert(comp_id, rc_comp);
        Ok(())
    }

    fn lookup_and_setup(
        &self,
        uid: Uid,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<Rc<RefCell<Box<dyn Component>>>> {
        let comp_rc = {
            self.component_map
                .get(&uid)
                .cloned()
                .ok_or_else(|| Error::msg(format!("Unknown UID: {}", uid)))?
        };

        // check if we've setup this component before. Ensure we don't borrow the tracker
        // multiple times
        let has_setup = self.setup_tracker.borrow().contains(&uid);
        if !has_setup {
            comp_rc
                .borrow_mut()
                .setup(pipeline, message_sender.clone(), self)?;
            self.setup_tracker.borrow_mut().insert(uid);
        }
        Ok(comp_rc)
    }

    fn start_or_resume(&self, pipeline: &gst::Pipeline) -> Result<()> {
        for comp in self.component_map.values() {
            let mut mutable_comp = comp.borrow_mut();
            // start all components
            mutable_comp.start_or_resume(pipeline)?;
        }
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        for comp in self.component_map.values() {
            let mut mutable_comp = comp.borrow_mut();
            // start all components
            mutable_comp.stop()?;
        }
        Ok(())
    }

    fn run(
        &self,
        pipeline: &gst::Pipeline,
        message_broker: std::sync::Arc<
            std::sync::Mutex<std::sync::mpsc::Receiver<crate::types::message::RuntimeMessage>>,
        >,
    ) -> Result<RuntimeMessage> {
        // if there is a main component then run it
        if let Some(comp_id) = self.main_comp_id {
            let mutable_comp = self.component_map.get(&comp_id);
            if let Some(mutable_comp) = mutable_comp {
                return mutable_comp.borrow().run(pipeline, message_broker);
            } else {
                return Err(Error::msg(
                    "Unable to find component. This should not happen due to previous checks",
                ));
            }
        } else {
            Err(anyhow!("No main component in component helper"))
        }
    }

    fn destory(&self) -> Result<()> {
        for comp in self.component_map.values() {
            let mut mutable_comp = comp.borrow_mut();
            // start all components
            mutable_comp.destroy()?;
        }
        Ok(())
    }

    fn has_main_requirement(&self) -> bool {
        !self.main_comp_id.is_none()
    }
}
