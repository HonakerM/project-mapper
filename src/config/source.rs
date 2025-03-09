use std::io::Sink;

use gst::Element;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceType {
    Test {},
}

impl crate::config::source::SourceType {
    pub fn create_element(&self) -> Result<Element, glib::BoolError> {
        return gst::ElementFactory::make("videotestsrc")
            .name("test-src")
            .build();
    }
}

#[derive(Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub id: u32,
    pub source: SourceType,
}
