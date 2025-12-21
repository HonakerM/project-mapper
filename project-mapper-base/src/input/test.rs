use std::any::Any;

use project_mapper_core::runtime_config::input::common::InputConfigTrait;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use std::sync::mpsc;

use anyhow::{Error, Result, anyhow};
use project_mapper_core::runtime_config::{
    input::InputComponentConfig,
    shared::{ComponentConfig, Uid},
};
use project_mapper_runtime::gst::{Element, info, prelude::*};
use project_mapper_runtime::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    gst,
    types::message::RuntimeMessage,
};

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(default)]

pub struct TestConfig {
    pub fps: i32,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig { fps: 30 }
    }
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl InputConfigTrait for TestConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn InputConfigTrait> {
        Box::new(self.clone())
    }
}

pub struct TestComponent {
    config: InputComponentConfig,
    element: Element,
    capsfilter: Element,
    branch: BranchControl,
}

#[project_mapper_macro::input_component(config = {TestConfig::default()}, schema = {serde_json::to_value(schema_for!(TestConfig)).unwrap()})]
impl Component for TestComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(unknown_config: &dyn ComponentConfig) -> Result<TestComponent> {
        // parse config and ensure it's correct types
        let config: InputComponentConfig = match unknown_config
            .as_any()
            .downcast_ref::<InputComponentConfig>()
        {
            Some(b) => Ok(b.clone()),
            None => Err(Error::msg(
                "ComponentConfig can not be typed to InputComponentConfig",
            )),
        }?;

        // ensure we have a test config
        println!("Creating TestComponent with config: {:?}", config.config);
        let test_config = match config.config.as_any().downcast_ref::<TestConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("InputComponentConfig is not TestConfig")),
        }?;

        // construct test gstreamer element
        let element = gst::ElementFactory::make("videotestsrc")
            .name(config.name())
            .build()?;

        // Add a caps filter to ensure correct fps
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .name(format!("{}-capsfilter", config.name()))
            .build()?;
        let caps = gst::Caps::builder("video/x-raw")
            .field("framerate", &gst::Fraction::new(test_config.fps, 1))
            .build();
        capsfilter.set_property("caps", &caps);

        let comp = Self {
            branch: BranchControl::new(config.name(), false, true)?,
            config: config,
            element: element,
            capsfilter: capsfilter,
        };

        Ok(comp)
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        _message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        // Add elements to the pipelines and sync status
        pipeline.add(&self.element)?;
        pipeline.add(&self.capsfilter)?;
        self.element.sync_state_with_parent()?;
        self.capsfilter.sync_state_with_parent()?;

        // link the element to the capsfilter
        self.element.link(&self.capsfilter)?;

        self.branch.add_to_pipeline(pipeline)?;
        // link the capsfilter to the branch output
        self.capsfilter.link(self.branch.get_output().unwrap())?;

        Ok(())
    }

    // accessor functions
    fn input_element(&self) -> Option<&Element> {
        None
    }
    fn output_element(&self) -> Option<&Element> {
        // return the tee element since that's what people should
        // be linking against
        self.branch.get_output()
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
