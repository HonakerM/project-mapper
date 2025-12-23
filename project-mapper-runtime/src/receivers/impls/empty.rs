use std::future::{self, Pending};

use crate::receivers::{
    impls::shared::ReceiverImpl,
    services::{
        available_config::LockedAvailableConfigService, config::LockedRuntimeConfigService,
    },
};

pub struct EmptyReceiver;

impl ReceiverImpl for EmptyReceiver {
    async fn run(
        address: String,
        _config_service: LockedRuntimeConfigService,
        _available_config_service: LockedAvailableConfigService,
    ) -> Result<(), Box<dyn std::error::Error>> {
        future::pending().await
    }
}
