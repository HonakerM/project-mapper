use crate::receivers::impls::http::utils::process_value_to_response;
use crate::receivers::services::available_config::{
    LockedAvailableConfigService, LockedGetAvailableConfigService,
};
use crate::receivers::services::config::{
    LockedGetRuntimeConfigService, LockedRuntimeConfigService, LockedUpdateRuntimeConfigService,
};
use anyhow::{Result, anyhow};
use axum::body::Body;
use axum::extract::Request;
use axum::http::header;
use axum::response::Response;
use axum::routing::post_service;
use axum::{Router, body::Bytes, extract::State, response::IntoResponse, routing::post};
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use http::StatusCode;
use http_body_util::BodyExt;
use log::debug;
use std::convert::Infallible;
use std::future::Ready;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::{runtime::Builder, task::JoinHandle};
use tower::Service;
use tower::util::ServiceExt;

#[derive(Clone)]
pub struct AxumUpdateRuntimeConfigService {
    base: LockedUpdateRuntimeConfigService,
}

impl AxumUpdateRuntimeConfigService {
    pub fn new(base: LockedRuntimeConfigService) -> Self {
        Self {
            base: LockedUpdateRuntimeConfigService::from_service(base),
        }
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
impl Service<Request<Body>> for AxumUpdateRuntimeConfigService {
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

#[derive(Clone)]
pub struct AxumGetRuntimeConfigService {
    base: LockedGetRuntimeConfigService,
}

impl AxumGetRuntimeConfigService {
    pub fn new(base: LockedRuntimeConfigService) -> Self {
        Self {
            base: LockedGetRuntimeConfigService::from_service(base),
        }
    }

    async fn run_request(&mut self, _req: &mut Request<Body>) -> Result<Response<Body>> {
        let result = self.base.call(()).await;

        match result {
            Ok(message) => match process_value_to_response(message) {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    let response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!(
                            "Failed to serialize available config: {:?}",
                            err
                        )))?;
                    Ok(response)
                }
            },
            Err(err) => {
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!(
                        "Failed to serialize available config: {:?}",
                        err
                    )))?;
                Ok(response)
            }
        }
    }
}
impl Service<Request<Body>> for AxumGetRuntimeConfigService {
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

#[derive(Clone)]
pub struct AxumGetAvailableConfigService {
    base: LockedGetAvailableConfigService,
}

impl AxumGetAvailableConfigService {
    pub fn new(base: LockedAvailableConfigService) -> Self {
        Self {
            base: LockedGetAvailableConfigService::from_service(base),
        }
    }

    async fn run_request(&mut self, _req: &mut Request<Body>) -> Result<Response<Body>> {
        let result = self.base.call(()).await;

        match result {
            Ok(message) => match process_value_to_response(message) {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    let response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!(
                            "Failed to serialize available config: {:?}",
                            err
                        )))?;
                    Ok(response)
                }
            },
            Err(err) => {
                let response = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!(
                        "Failed to serialize available config: {:?}",
                        err
                    )))?;
                Ok(response)
            }
        }
    }
}
impl Service<Request<Body>> for AxumGetAvailableConfigService {
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
