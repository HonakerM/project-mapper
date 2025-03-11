use super::options::FullscreenOptions;

pub enum RuntimeEvent {
    UserExit(),
    StopThread(),
}

pub enum OptionEvent {
    OpenGLWindowOptions(FullscreenOptions),
}
