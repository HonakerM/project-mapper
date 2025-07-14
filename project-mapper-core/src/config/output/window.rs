pub enum WindowMode {
    Windowed {},
    Borderless { name: String },
    //Exclusive { info: MonitorInfo },
}

pub struct WindowConfig {
    mode: WindowMode,
}
