use anyhow::Result;
use project_mapper_core::config::{options::AvailableConfig, runtime::RuntimeConfig};

#[path = "./runtime_api/mod.rs"]
pub mod runtime_api;

const CONFIG: &str = r#"{"sinks":[{"name":"main monitor","id":1,"sink":{"type":"OpenGLWindow","full_screen":{"type":"Windowed"}}},{"name":"other monitor","id":2,"sink":{"type":"OpenGLWindow","full_screen":{"type":"Borderless","name":".DISPLAY1"}}},{"name":"other monitor","id":3,"sink":{"type":"OpenGLWindow","full_screen":{"type":"Exclusive","info":{"name":".DISPLAY2","resolution":"2560x1440","refresh_rate_mhz":120000}}}}],"sources":[{"name":"test source","id":1,"source":{"type":"Test"}},{"name":"uri source","id":2,"source":{"type":"Test"}}],"regions":[{"name":"main region","id":1,"region":{"type":"Display","source":1,"sink":1}},{"name":"secondary region","id":2,"region":{"type":"Display","source":2,"sink":2}},{"name":"secondary region","id":3,"region":{"type":"Display","source":1,"sink":3}}]}"#;

fn main() -> Result<()> {
    let available_config = runtime_api::config::get_available_config()?;

    println!("{:?}", available_config);

    let current_config: RuntimeConfig = serde_json::from_str(CONFIG)?;
    let mut run_api = runtime_api::runtime::RunApi::construct_and_start_runtime(current_config)?;

    run_api.runtime_process.wait()?;

    Ok(())
}
