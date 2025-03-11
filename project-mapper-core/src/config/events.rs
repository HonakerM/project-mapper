use super::options::SinkTypeOptions;

pub enum RuntimeEvent {
    UserExit(),
    StopThread(),
}

pub enum OptionEvent {
    OpenGLWindowOptions(SinkTypeOptions),
}
