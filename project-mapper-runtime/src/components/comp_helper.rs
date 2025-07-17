use std::{cell::RefCell, collections::HashMap, rc::Rc};

use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::{
    factory::create_default_component,
    shared::{Component, ComponentLookupHelper},
};
use anyhow::{Error, Result};

pub struct ComponentHelper {
    main_comp_id: Option<Uid>,
    component_map: HashMap<Uid, Rc<RefCell<Box<dyn Component>>>>,
}

impl ComponentHelper {
    pub fn new() -> Self {
        Self {
            main_comp_id: None,
            component_map: HashMap::new(),
        }
    }
}

impl ComponentLookupHelper for ComponentHelper {
    fn create_and_insert_comp(&mut self, config: &dyn ComponentConfig) -> Result<()> {
        // create the component
        let comp = create_default_component(config)?;

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
    ) -> Result<Rc<RefCell<Box<dyn Component>>>> {
        let comp_rc = {
            self.component_map
                .get(&uid)
                .cloned()
                .ok_or_else(|| Error::msg(format!("Unknown UID: {}", uid)))?
        };

        let has_setup = comp_rc.borrow().has_setup();
        if !has_setup {
            comp_rc.borrow_mut().setup(pipeline, self)?;
        }
        Ok(comp_rc)
    }

    fn start_and_run(&self, pipeline: &gst::Pipeline) -> Result<()> {
        for comp in self.component_map.values() {
            let mutable_comp = comp.borrow();
            // if we don't require main then start. Else mark the component
            // for later starting. This ensures we start all components before running the `main` one
            if !mutable_comp.requires_main() {
                mutable_comp.start_or_run(pipeline)?;
            }
        }

        // if there is a main component then run it
        if let Some(comp_id) = self.main_comp_id {
            let mutable_comp = self.component_map.get(&comp_id);
            if let Some(mutable_comp) = mutable_comp {
                return mutable_comp.borrow().start_or_run(pipeline);
            } else {
                return Err(Error::msg(
                    "Unable to find component. This should not happen due to previous checks",
                ));
            }
        }
        Ok(())
    }

    fn has_main_requirement(&self) -> bool {
        !self.main_comp_id.is_none()
    }
}
