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
    fn new(&mut self, config: &dyn ComponentConfig, factory: &dyn ComponentFactory) -> Result<()> {
        // else create the component
        let mut comp = factory.create_component(config)?;

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

    fn update(&mut self, config: &dyn ComponentConfig) -> Result<()> {
        // if the component map already contains the key then just update it
        if let Some(comp) = self.component_map.get(&config.uid()) {
            let mut mut_comp = comp.borrow_mut();
            mut_comp.update(config)?;

            // if this update affected the main requirements of the component check it.
            if mut_comp.requires_main() {
                if let Some(current_id) = self.main_comp_id {
                    if current_id != mut_comp.uid() {
                        return Err(Error::msg(
                            "Component that requires main already exists. Can not have two main components",
                        ));
                    }
                } else {
                    // if we don't have a main component id then update it
                    self.main_comp_id = Some(mut_comp.uid());
                }
            }
            Ok(())
        } else {
            Err(anyhow!(
                "Component {} is has not been created yet.",
                config.uid()
            ))
        }
    }

    fn setup(
        &self,
        uid: Uid,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
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
                .setup(pipeline, message_sender.clone())?;
            self.setup_tracker.borrow_mut().insert(uid);
        }
        Ok(())
    }

    fn resume(&self) -> Result<()> {
        for comp in self.component_map.values() {
            let mut mutable_comp = comp.borrow_mut();
            // start all components
            mutable_comp.resume()?;
        }
        Ok(())
    }
    fn pause_comp(&mut self, uid: &Uid) -> Result<()> {
        if let Some(comp) = self.get_comp(uid) {
            let mut local_comp = comp.borrow_mut();
            local_comp.pause();
        }
        Ok(())
    }
    fn run(
        &self,
        message_broker: std::sync::Arc<
            std::sync::Mutex<std::sync::mpsc::Receiver<crate::types::message::RuntimeMessage>>,
        >,
    ) -> Result<RuntimeMessage> {
        // if there is a main component then run it
        if let Some(comp_id) = self.main_comp_id {
            let mutable_comp = self.component_map.get(&comp_id);
            if let Some(mutable_comp) = mutable_comp {
                return mutable_comp.borrow().run(message_broker);
            } else {
                return Err(Error::msg(
                    "Unable to find component. This should not happen due to previous checks",
                ));
            }
        } else {
            Err(anyhow!("No main component in component helper"))
        }
    }

    fn destroy_comp(&mut self, uid: &Uid) -> Result<()> {
        // start by destroying the component
        if let Some(comp) = self.component_map.get(uid) {
            let mut mut_comp = comp.borrow_mut();
            mut_comp.destroy()?;
        }

        // ensure to remove it from the component map and update our main_comp_id tracker
        // if it has changed
        self.component_map.remove(uid);
        if let Some(main_uid) = self.main_comp_id
            && main_uid == *uid
        {
            self.main_comp_id = None;
        }
        Ok(())
    }
    fn destory(&mut self) -> Result<()> {
        let keys: Vec<Uid> = self.component_map.keys().copied().collect();
        for uid in keys {
            self.destroy_comp(&uid)?;
        }
        Ok(())
    }

    fn has_main_requirement(&self) -> bool {
        !self.main_comp_id.is_none()
    }
    fn contains_comp(&self, uid: &Uid) -> bool {
        self.component_map.contains_key(uid)
    }
    fn get_comp(&self, uid: &Uid) -> Option<Rc<RefCell<Box<dyn Component>>>> {
        self.component_map.get(uid).cloned()
    }
    fn components(&self) -> Vec<Rc<RefCell<Box<dyn Component>>>> {
        Vec::from_iter(self.component_map.values().map(|x| x.clone()).into_iter())
    }
}
