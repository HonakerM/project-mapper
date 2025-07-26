use std::{error::Error, sync::Arc};

use crate::receivers::services::update::LockedUpdateService;

pub trait ReceiverImpl {
    async fn run(
        address: String,
        update_service: LockedUpdateService,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
