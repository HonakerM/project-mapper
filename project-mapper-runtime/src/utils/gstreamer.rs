use std::sync::mpsc;

use gst::{prelude::*, Element, Pad, PadProbeType, Pipeline};
use anyhow::{Result};

use crate::components::bin_wrapper::BinWrapper;

const NULL_ELEMENT_NAME: &str = "null_element_name";


pub fn get_or_create_null_element(pipeline: &Pipeline)->Result<Element> {
    let element = if let Some(element) = pipeline.by_name(NULL_ELEMENT_NAME){
        element
    } else {
        let element = gst::ElementFactory::make("fakesink")
            .name(NULL_ELEMENT_NAME)
            .build()?;
        let funnel = gst::ElementFactory::make("funnel")
            .name(format!("{}-funnel", NULL_ELEMENT_NAME))
            .build()?;
        let bin_element = BinWrapper::new(&[&funnel, &element], false, false);
        
        pipeline.add_many([&bin_element])?;
        bin_element.sync_state_with_parent()?;
        bin_element.upcast()
    };
    Ok(element)

}


pub fn unlink_element(element: &Element, pipeline: &Pipeline) -> Result<()> {
    let null_element = get_or_create_null_element(pipeline)?;

    let (unlink_sender, unlink_rec) = mpsc::channel();
    let mut total_count = 0;
    for input_sink_pad in element.sink_pads() {
        if input_sink_pad.is_linked() {
            if let Some(peer_pad) = input_sink_pad.peer() {
                total_count += 1;
                let local_unlink_sender = unlink_sender.clone();
                let local_null_element = null_element.clone();
                peer_pad.add_probe(PadProbeType::BLOCK, move |pad, _probe_info| {
                    // send and wait for a eos event
                    let (eos_sender, eos_rec) = mpsc::channel();
                    input_sink_pad.add_probe(
                        PadProbeType::EVENT_DOWNSTREAM,
                        move |sink_pad, sink_probe_inf| {
                            if let Some(event) = sink_probe_inf.event() {
                                if event.type_() == gst::EventType::Eos {
                                    eos_sender.send(true);
                                    gst::PadProbeReturn::Remove
                                } else {
                                    gst::PadProbeReturn::Ok
                                }
                            } else {
                                gst::PadProbeReturn::Ok
                            }
                        },
                    );
                    eos_rec.recv();
                    // unlink
                    pad.unlink(&input_sink_pad);

                    local_unlink_sender.send(true);

                    gst::PadProbeReturn::Remove
                });
            }
        }
    }

    for _ in unlink_rec {
        total_count-=1;
        if total_count < 0  {
            break
        }
    }
    Ok(())
}
pub fn remove_element(element: &Element, pipeline: &Pipeline)->Result<()>{
    // get null element
    let null_element = get_or_create_null_element(pipeline)?;

    // start by removing all srcs from element

    Ok(())
}