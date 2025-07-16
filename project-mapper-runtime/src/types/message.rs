pub enum RuntimeMessage {
    StartPipeline(),
    StopPipeline(),
    GSTMessage(gst::Message),
}
