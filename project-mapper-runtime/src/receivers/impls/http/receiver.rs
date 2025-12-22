use crate::receivers::impls::http::wrapper::{AxumAvailableConfigService, AxumUpdateService};
use crate::receivers::services::available_config::LockedAvailableConfigService;
use crate::receivers::services::update::LockedUpdateService;
use crate::receivers::{impls::shared::ReceiverImpl, services::update::UpdateRuntimeService};
use anyhow::{Result, anyhow};
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use axum::routing::{get_service, post_service};
use axum::{Router, body::Bytes, extract::State, response::IntoResponse, routing::post};
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use http::StatusCode;
use http_body_util::BodyExt;
use log::info;
use std::convert::Infallible;
use std::future::Ready;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::{runtime::Builder, task::JoinHandle};
use tower::Service;
use tower::util::ServiceExt;

pub struct HttpReceiver;

impl ReceiverImpl for HttpReceiver {
    async fn run(
        address: String,
        update_service: LockedUpdateService,
        available_config_service: LockedAvailableConfigService,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = build_axum(update_service, available_config_service);
        info!("Serving HTTP on http://{}", address);
        let listener = tokio::net::TcpListener::bind(address).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

fn build_axum(
    update_service: LockedUpdateService,
    available_config_service: LockedAvailableConfigService,
) -> Router {
    let http_update_wrapper = AxumUpdateService::new(update_service);
    let http_available_config = AxumAvailableConfigService::new(available_config_service);

    Router::new()
        .route("/v1/config", post_service(http_update_wrapper))
        .route("/v1/available_config", get_service(http_available_config))
}
