use crate::config::output::window::WindowConfig;
use crate::config::shared::Component;

pub enum OutputConfig {
    Window(WindowConfig),
}

// OutputComponent is the generic component for
// all output types
pub struct OutputComponent {
    uid: u32,
    name: String,
    config: OutputConfig,
}

// Implmement the Shared component trait to allow name/id fetching
impl Component for OutputComponent {
    fn name(self) -> String {
        self.name.clone()
    }

    fn uid(self) -> u32 {
        self.uid
    }
}
