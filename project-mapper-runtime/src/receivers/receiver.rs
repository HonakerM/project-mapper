#[cfg(not(feature = "http-receiver"))]
use std::future;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

use project_mapper_core::runtime_config::RuntimeConfig;
use tokio::runtime::Builder;
use tokio_util::sync::CancellationToken;

#[cfg(not(feature = "http-receiver"))]
use crate::receivers::impls::empty::EmptyReceiver;
use crate::{
    receivers::{
        impls::shared::ReceiverImpl,
        services::{available_config::LockedAvailableConfigService, update::LockedUpdateService},
    },
    types::message::RuntimeMessage,
};
// only import http if we have the feature enabled
#[cfg(feature = "http-receiver")]
use crate::receivers::impls::http::HttpReceiver;

use anyhow::{Result, anyhow};

pub struct ReceiverHandle {
    join_handle: Option<JoinHandle<Result<()>>>,
    cancel_token: CancellationToken,
}

impl ReceiverHandle {
    pub fn new(join_handle: JoinHandle<Result<()>>, cancel_token: CancellationToken) -> Self {
        Self {
            join_handle: Some(join_handle),
            cancel_token,
        }
    }

    pub fn cancel(&mut self) -> Result<()> {
        self.cancel_token.cancel();
        let local_join_handle = self.join_handle.take();
        if let Some(join_handle) = local_join_handle {
            join_handle
                .join()
                .map_err(|e| anyhow!("Received error from receiver thread: {:?}", e))?
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
struct Receiver {
    update_service: LockedUpdateService,
    available_config_service: LockedAvailableConfigService,
}

impl Receiver {
    pub fn new(sender: mpsc::Sender<RuntimeMessage>, config: Arc<Mutex<RuntimeConfig>>) -> Self {
        Self {
            update_service: LockedUpdateService::new(sender, config),
            available_config_service: LockedAvailableConfigService::new(),
        }
    }

    pub fn start(&self) -> Result<ReceiverHandle> {
        let cancel_token = CancellationToken::new();

        let local_receiver = self.clone();
        let local_cancel_token = cancel_token.clone();
        let join_handle = thread::spawn(move || {
            let rt = Builder::new_current_thread().enable_all().build()?;

            rt.block_on(local_receiver.run_async(local_cancel_token))
        });

        Ok(ReceiverHandle::new(join_handle, cancel_token))
    }

    async fn run_async(&self, cancel_token: CancellationToken) -> Result<()> {
        #[cfg(feature = "http-receiver")]
        let http_fut = HttpReceiver::run(
            "localhost:3000".to_string(),
            self.update_service.clone(),
            self.available_config_service.clone(),
        );
        #[cfg(not(feature = "http-receiver"))]
        let http_fut =
            EmptyReceiver::run("localhost:3000".to_string(), self.update_service.clone());

        tokio::select! {
            // Step 3: Using cloned token to listen to cancellation requests
            _ = cancel_token.cancelled() => {
                Ok(())
            }
            res = http_fut => {
                Ok(res.map_err(|e|anyhow!("Unable to run http receiver due to err: {:?}",e))?)
            }
        }
    }
}

pub fn start_receiver(
    sender: mpsc::Sender<RuntimeMessage>,
    config: Arc<Mutex<RuntimeConfig>>,
) -> Result<ReceiverHandle> {
    let local_receiver = Receiver::new(sender, config);
    local_receiver.start()
}
