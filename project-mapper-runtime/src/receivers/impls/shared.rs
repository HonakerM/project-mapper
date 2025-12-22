use std::{error::Error, sync::Arc};

use crate::receivers::services::{
    available_config::LockedAvailableConfigService, update::LockedUpdateService,
};

pub trait ReceiverImpl {
    async fn run(
        address: String,
        update_service: LockedUpdateService,
        available_config_service: LockedAvailableConfigService,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
