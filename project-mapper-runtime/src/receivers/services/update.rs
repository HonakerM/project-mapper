use std::{
    convert::Infallible,
    future::{Future, ready},
    pin::Pin,
    sync::mpsc,
    task::{Context, Poll},
};

use project_mapper_core::{
    loader::runtime_loader::load_config_json, runtime_config::RuntimeConfig,
};
use std::sync::Arc;
use tokio::runtime::Builder;
use tower::Service; // for `oneshot`

use crate::types::message::RuntimeMessage;

// Generic Service for updating the runtime
#[derive(Clone)]
pub struct UpdateRuntimeService {
    sender: mpsc::Sender<RuntimeMessage>,
    config: RuntimeConfig,
}

impl UpdateRuntimeService {
    pub fn new(sender: mpsc::Sender<RuntimeMessage>, config: RuntimeConfig) -> Self {
        Self { sender, config }
    }
}

impl Service<String> for UpdateRuntimeService {
    type Response = String;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(())) // Always ready
    }

    fn call(&mut self, input: String) -> Self::Future {
        // attempt to load the string as a json
        let new_config = match load_config_json(&input) {
            Ok(config) => config,
            Err(err) => {
                return Box::pin(ready(Err(format!(
                    "Unable to deserialize config due to: {:#}",
                    err
                ))));
            }
        };

        // validate that the new config can be converted from the old
        match self.config.validate_changes(&new_config) {
            Err(err) => {
                return Box::pin(ready(Err(format!(
                    "Unable to update to new config due to: {:#}",
                    err
                ))));
            }
            _ => {}
        }

        // send the update message and update our local conifg
        match self
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
        self.config = new_config;

        // send back the successful runtime
        Box::pin(ready(Ok("Updated runtime".to_string())))
    }
}
