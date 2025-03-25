use std::env;

pub struct EnvConfig {
    pub runtime_bin: String,
}

impl EnvConfig {
    pub fn get_config() -> EnvConfig {
        #[cfg(target_os = "windows")]
        let default_binary = "project-mapper-runtime.exe";
        #[cfg(target_os = "linux")]
        let default_binary = "project-mapper-runtime";
        #[cfg(target_os = "macos")]
        let default_binary = "project-mapper-runtime";

        let runtime_bin = env::var("RUNTIME_BIN").unwrap_or(String::from(default_binary));
        EnvConfig {
            runtime_bin: runtime_bin,
        }
    }
}
