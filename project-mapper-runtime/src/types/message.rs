pub enum RuntimeMessage {
    UserExit(),
    GSTMessage(gst::Message),
}
