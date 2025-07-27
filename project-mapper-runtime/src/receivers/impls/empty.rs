use std::future::{self, Pending};

use crate::receivers::impls::shared::ReceiverImpl;

pub struct EmptyReceiver;

impl ReceiverImpl for EmptyReceiver {
    async fn run(
        _address: String,
        _update_service: crate::receivers::services::update::LockedUpdateService,
    ) -> Result<(), Box<dyn std::error::Error>> {
        future::pending().await
    }
}
