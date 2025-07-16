use std::{cell::RefCell, collections::HashMap, rc::Rc};

use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::{
    factory::create_component,
    shared::{Component, ComponentLookupHelper},
};
use anyhow::{Error, Result};

pub struct ComponentHelper {
    component_map: HashMap<Uid, Rc<RefCell<Box<dyn Component>>>>,
}

impl ComponentHelper {
    pub fn new() -> Self {
        Self {
            component_map: HashMap::new(),
        }
    }
    pub fn create_and_insert_comp(&mut self, config: &dyn ComponentConfig) -> Result<()> {
        let comp = create_component(config)?;
        let comp_id = comp.uid();
        let rc_comp = Rc::new(RefCell::new(comp));
        self.component_map.insert(comp_id, rc_comp);
        Ok(())
    }

    pub fn start(&mut self, pipeline: &gst::Pipeline) -> Result<()> {
        for comp in self.component_map.values() {
            comp.borrow_mut().start(pipeline)?;
        }
        Ok(())
    }
}

impl ComponentLookupHelper for ComponentHelper {
    fn lookup_and_setup(
        &self,
        uid: Uid,
        pipeline: &gst::Pipeline,
    ) -> Result<Rc<RefCell<Box<dyn Component>>>> {
        let comp_rc = {
            self.component_map
                .get(&uid)
                .cloned()
                .ok_or_else(|| Error::msg(format!("Unknown UID: {}", uid)))?
        };

        comp_rc.borrow_mut().setup(pipeline, self)?;

        Ok(comp_rc)
    }
}
