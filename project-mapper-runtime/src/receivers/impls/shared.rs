use std::{error::Error, sync::Arc};

use crate::receivers::services::update::UpdateRuntimeService;

pub trait ReceiverImpl {
    async fn run(
        address: String,
        update_service: Arc<tokio::sync::Mutex<UpdateRuntimeService>>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
