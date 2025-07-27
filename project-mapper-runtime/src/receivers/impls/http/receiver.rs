use crate::receivers::impls::http::wrapper::AxumUpdateService;
use crate::receivers::services::update::LockedUpdateService;
use crate::receivers::{impls::shared::ReceiverImpl, services::update::UpdateRuntimeService};
use anyhow::{Result, anyhow};
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use axum::routing::post_service;
use axum::{Router, body::Bytes, extract::State, response::IntoResponse, routing::post};
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use http::StatusCode;
use http_body_util::BodyExt;
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = build_axum(update_service);
        println!("Serving HTTP on http://{}", address);
        let listener = tokio::net::TcpListener::bind(address).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

fn build_axum(update_service: LockedUpdateService) -> Router {
    let http_update_wrapper = AxumUpdateService::new(update_service);

    Router::new().route("/v1/config", post_service(http_update_wrapper))
}
