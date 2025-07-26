use std::{
    convert::Infallible,
    future::{Future, ready},
    pin::Pin,
    sync::{Mutex, mpsc},
    task::{Context, Poll},
};

use anyhow::Result;
use project_mapper_core::{
    loader::runtime_loader::load_config_json, runtime_config::RuntimeConfig,
};
use std::sync::Arc;
use tokio::runtime::Builder;
use tower::Service; // for `oneshot`

use crate::types::message::RuntimeMessage;

// Generic Service for updating the runtime
#[derive(Debug, Clone)]
pub struct UpdateRuntimeService {
    sender: mpsc::Sender<RuntimeMessage>,
    config: Arc<Mutex<RuntimeConfig>>,
}

impl UpdateRuntimeService {
    pub fn new(sender: mpsc::Sender<RuntimeMessage>, config: Arc<Mutex<RuntimeConfig>>) -> Self {
        Self { sender, config }
    }

    pub fn new_locked(
        sender: mpsc::Sender<RuntimeMessage>,
        config: Arc<Mutex<RuntimeConfig>>,
    ) -> LockedUpdateService {
        LockedUpdateService(Arc::new(Mutex::new(UpdateRuntimeService::new(
            sender, config,
        ))))
    }
}

#[derive(Clone)]
pub struct LockedUpdateService(pub Arc<Mutex<UpdateRuntimeService>>);

impl Service<String> for LockedUpdateService {
    type Response = String;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(())) // Always ready
    }

    fn call(&mut self, input: String) -> Self::Future {
        // attempt to load the string as a json before locking the service
        let new_config = match load_config_json(&input) {
            Ok(config) => config,
            Err(err) => {
                return Box::pin(ready(Err(format!(
                    "Unable to deserialize config due to: {:#}",
                    err
                ))));
            }
        };

        // Access the underlying type of the service and ensure it's locked
        let svc_arc = self.0.clone();
        let svc = match svc_arc.lock() {
            Ok(svc) => svc,
            Err(err) => {
                return Box::pin(ready(Err(format!(
                    "Unable to lock service due to poison error: {:#}",
                    err
                ))));
            }
        };

        // Lock the current config and get its value
        {
            let current_config = match svc.config.lock() {
                Ok(config) => config,
                Err(err) => {
                    return Box::pin(ready(Err(format!(
                        "Unable to capture config due to : {:#}",
                        err
                    ))));
                }
            };

            // validate that the new config can be converted from the old
            match current_config.validate_changes(&new_config) {
                Err(err) => {
                    return Box::pin(ready(Err(format!(
                        "Unable to update to new config due to: {:#}",
                        err
                    ))));
                }
                _ => {}
            }

            // send the update message and update our local conifg
            match svc
                .sender
                .send(RuntimeMessage::UpdateRuntime(new_config.clone()))
            {
                Err(err) => {
                    return Box::pin(ready(Err(format!(
                        "Unable to send runtime message due to err: {:#}. Runtime is most likely shutting down",
                        err
                    ))));
                }
                _ => {}
            }
        }

        // send back the successful runtime
        Box::pin(ready(Ok("Updated runtime".to_string())))
    }
}
