use crate::receivers::{impls::shared::ReceiverImpl, services::update::UpdateRuntimeService};
use axum::{Router, body::Bytes, extract::State, response::IntoResponse, routing::post};
use http::StatusCode;
use std::sync::{Arc, Mutex};
use tokio::{runtime::Builder, task::JoinHandle};
use tower::Service;
use tower::util::ServiceExt;

pub struct HttpReceiver;

impl ReceiverImpl for HttpReceiver {
    async fn run(
        address: String,
        update_service: Arc<tokio::sync::Mutex<UpdateRuntimeService>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app = build_axum(update_service);
        println!("Serving HTTP on http://{}", address);
        let listener = tokio::net::TcpListener::bind(address).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn axum_handler(
    State(service): State<Arc<tokio::sync::Mutex<UpdateRuntimeService>>>,
    body: Bytes,
) -> impl IntoResponse {
    let input = match String::from_utf8(body.to_vec()) {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UTF-8").into_response(),
    };

    let mut locked_service = service.lock().await;
    match locked_service.ready().await {
        Ok(_) => match locked_service.call(input).await {
            Ok(resp) => resp.into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Service not ready").into_response(),
    }
}

fn build_axum(service: Arc<tokio::sync::Mutex<UpdateRuntimeService>>) -> Router {
    Router::new()
        .route("/v1/update_runtime", post(axum_handler))
        .with_state(service)
}
