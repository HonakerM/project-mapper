use std::{
    convert::Infallible,
    future::{Future, ready},
    pin::Pin,
    sync::{Mutex, mpsc},
    task::{Context, Poll},
};

use anyhow::Result;
use project_mapper_core::{
    available_config::config::AvailableConfig, loader::runtime_loader::load_config_json,
    runtime_config::RuntimeConfig,
};
use std::sync::Arc;
use tokio::runtime::Builder;
use tower::Service; // for `oneshot`

use crate::{components::available_config::AvailableConfigHelper, types::message::RuntimeMessage};

const AVAILABLE_CONFIG_REFRESH_DELAY: std::time::Duration = std::time::Duration::from_secs(1);

pub type LockedAvailableConfigService = Arc<Mutex<AvailableConfigService>>;

// Generic Service for updating the runtime
#[derive(Debug, Clone)]
pub struct AvailableConfigService {
    config: AvailableConfig,
    last_update: std::time::Instant,
}

impl AvailableConfigService {
    pub fn new() -> Self {
        Self {
            config: AvailableConfigHelper::get_config(),
            last_update: std::time::Instant::now(),
        }
    }

    pub fn get_or_update(&mut self) -> AvailableConfig {
        if self.last_update.elapsed() > AVAILABLE_CONFIG_REFRESH_DELAY {
            self.config = AvailableConfigHelper::get_config();
            self.last_update = std::time::Instant::now();
        }
        self.config.clone()
    }
}

#[derive(Clone)]
pub struct LockedGetAvailableConfigService(pub LockedAvailableConfigService);

impl LockedGetAvailableConfigService {
    pub fn new() -> LockedGetAvailableConfigService {
        LockedGetAvailableConfigService(Arc::new(Mutex::new(AvailableConfigService::new())))
    }

    pub fn from_service(service: LockedAvailableConfigService) -> LockedGetAvailableConfigService {
        LockedGetAvailableConfigService(service)
    }
}

impl Service<()> for LockedGetAvailableConfigService {
    type Response = serde_json::Value;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(())) // Always ready
    }

    fn call(&mut self, input: ()) -> Self::Future {
        // Access the underlying type of the service and ensure it's locked
        let svc_arc = self.0.clone();
        let mut svc = match svc_arc.lock() {
            Ok(svc) => svc,
            Err(err) => {
                return Box::pin(ready(Err(format!(
                    "Unable to lock service due to poison error: {:#}",
                    err
                ))));
            }
        };

        // Lock the current config and get its value
        let available_config = svc.get_or_update();

        // send back the successful runtime
        Box::pin(ready(Ok(available_config.get_schema().to_json_value())))
    }
}
