use std::future::{self, Pending};

use crate::receivers::{
    impls::shared::ReceiverImpl, services::available_config::LockedAvailableConfigService,
};

pub struct EmptyReceiver;

impl ReceiverImpl for EmptyReceiver {
    async fn run(
        _address: String,
        _update_service: crate::receivers::services::update::LockedUpdateService,
        _available_config_service: LockedAvailableConfigService,
    ) -> Result<(), Box<dyn std::error::Error>> {
        future::pending().await
    }
}
