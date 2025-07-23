use project_mapper_core::runtime_config::RuntimeConfig;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeMessage {
    ExitRuntime(),
    UpdateRuntime(RuntimeConfig),
}
