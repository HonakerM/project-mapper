use std::sync::mpsc;

use gst::{prelude::{ElementExt, ElementExtManual, GstBinExt, GstBinExtManual}, Element, Pad, Pipeline};
use anyhow::{Result};

pub fn unlink_pads(src_pad: Pad, sink_pad: Pad){
    
}

const NULL_ELEMENT_NAME: &str = "null_element_name";


pub fn get_or_create_null_element(pipeline: &Pipeline)->Result<Element> {
    let element = if let Some(element) = pipeline.by_name(NULL_ELEMENT_NAME){
        element
    } else {
        let element = gst::ElementFactory::make("fakesink")
            .name(NULL_ELEMENT_NAME)
            .build()?;
        pipeline.add_many([&element])?;
        element.sync_state_with_parent()?;
        element
    };
    Ok(element)

}
pub fn remove_element(element: &Element, pipeline: &Pipeline)->Result<()>{
    // get null element
    let null_element = get_or_create_null_element(pipeline)?;

    // start by removing all srcs from element

    Ok(())
}