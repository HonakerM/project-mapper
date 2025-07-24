use std::sync::{Arc, mpsc};

use project_mapper_core::runtime_config::RuntimeConfig;
use tokio::runtime::Builder;

use crate::{
    receivers::{
        impls::{http::HttpReceiver, shared::ReceiverImpl},
        receiver,
        services::update::UpdateRuntimeService,
    },
    types::message::RuntimeMessage,
};

use anyhow::{Result, anyhow};

#[derive(Clone)]
struct Receiver {
    update_service: Arc<tokio::sync::Mutex<UpdateRuntimeService>>,
}

impl Receiver {
    pub fn new(sender: mpsc::Sender<RuntimeMessage>, config: RuntimeConfig) -> Self {
        Self {
            update_service: Arc::new(tokio::sync::Mutex::new(UpdateRuntimeService::new(
                sender, config,
            ))),
        }
    }

    pub fn run(&self) -> Result<()> {
        let rt = Builder::new_current_thread().enable_all().build()?;

        rt.block_on(self.run_async())
    }

    async fn run_async(&self) -> Result<()> {
        HttpReceiver::run("localhost:3000".to_string(), self.update_service.clone())
            .await
            .map_err(|e| anyhow!("Failed to run HTTP service {:#}", e))
    }
}

pub fn run_receiver(sender: mpsc::Sender<RuntimeMessage>, config: RuntimeConfig) -> Result<()> {
    let local_receiver = Receiver::new(sender, config);
    local_receiver.run()
}
