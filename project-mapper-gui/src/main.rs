use anyhow::Result;

#[path = "./core/mod.rs"]
pub mod core;

#[path = "./runtime_api/mod.rs"]
pub mod runtime_api;

#[path = "./config/mod.rs"]
pub mod config;
#[path = "./wigets/mod.rs"]
pub mod wigets;

fn main() -> Result<()> {
    // let available_config = runtime_api::config::get_available_config()?;

    // println!("{:?}", available_config);

    // let current_config: RuntimeConfig = serde_json::from_str(CONFIG)?;
    // let mut run_api = runtime_api::runtime::RunApi::construct_and_start_runtime(current_config)?;

    // run_api.runtime_process.wait()?;
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(core::app::CoreApp::new(cc)?))),
    );

    Ok(())
}
