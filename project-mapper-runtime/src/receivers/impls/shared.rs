use std::{error::Error, sync::Arc};

use crate::receivers::services::{
    available_config::LockedAvailableConfigService, config::LockedRuntimeConfigService,
};

pub trait ReceiverImpl {
    async fn run(
        address: String,
        config_service: LockedRuntimeConfigService,
        available_config_service: LockedAvailableConfigService,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
