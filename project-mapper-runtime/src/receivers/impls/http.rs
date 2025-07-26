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

#[derive(Clone)]
pub struct AxumUpdateService {
    base: LockedUpdateService,
}

impl AxumUpdateService {
    fn new(base: LockedUpdateService) -> Self {
        Self { base }
    }

    async fn run_request(&mut self, req: &mut Request<Body>) -> Result<Response<Body>> {
        let Ok(body) = req.body_mut().collect().await else {
            return Err(anyhow!("Failed to read body"));
        };
        let body: Bytes = body.to_bytes();
        let body = String::from_utf8(body.to_vec())?;

        let result = self.base.call(body).await;
        match result {
            Ok(message) => {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(message))?;
                Ok(response)
            }
            Err(err) => {
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(err))?;
                Ok(response)
            }
        }
    }
}
impl Service<Request<Body>> for AxumUpdateService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Always ready; locking happens in call
        match self.base.poll_ready(cx) {
            Poll::Ready(_) => Poll::Ready(Ok(())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let mut this = self.clone();
        async move {
            match this.run_request(&mut req).await {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    let response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!("Unable to run request: {:#}", err)))
                        .unwrap();
                    Ok(response)
                }
            }
        }
        .boxed()
    }
}

fn build_axum(update_service: LockedUpdateService) -> Router {
    let http_update_wrapper = AxumUpdateService::new(update_service);

    Router::new().route("/v1/update_runtime", post_service(http_update_wrapper))
}
