use crate::receivers::services::update::LockedUpdateService;
use crate::receivers::{impls::shared::ReceiverImpl, services::update::UpdateRuntimeService};
use axum::body::{Body};
use axum::extract::Request;
use axum::response::Response;
use axum::{Router, body::Bytes, extract::State, response::IntoResponse, routing::post};
use http::StatusCode;
use std::convert::Infallible;
use http_body_util::BodyExt;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
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


pub struct AxumUpdateService {
    base: LockedUpdateService
}

impl Service<Request<Body>> for AxumUpdateService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = futures_util::future::Ready<Result<Self::Response, Self::Error>>;

    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Always ready; locking happens in call
        match self.base.poll_ready(cx) {
            Poll::Ready(_) => Poll::Ready(Ok(())),
            Poll::Pending => Poll::Pending,
        }
        
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let thing = req.body_mut().collect();

        // get the data as a string
        let body_bytes = body::to_bytes(req);

        let str_data = req.body();
        // get the inner service
        let inner = self.base.clone();
        let result = inner.call(req);

        let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Hello from Tower service!"))
        .unwrap();

        futures_util::future::ready(Ok(response))
    }
}

async fn axum_handler(
    State(service): State<Arc<Mutex<UpdateRuntimeService>>>,
    body: Bytes,
) -> impl IntoResponse {
    let input = match String::from_utf8(body.to_vec()) {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UTF-8").into_response(),
    };

    match service.ready().await {
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
