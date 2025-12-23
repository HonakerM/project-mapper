use axum::{body::Body, response::Response};
use http::{StatusCode, header};

use anyhow::Result;

pub fn process_value_to_response(val: serde_json::Value) -> Result<Response> {
    let str_message_or_fail_response = match serde_json::to_string(&val) {
        Ok(str_message) => Ok(str_message),
        Err(err) => {
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Failed to serialize available config: {:?}",
                    err
                )))?;
            Err(response)
        }
    };

    let result = match str_message_or_fail_response {
        Ok(message) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json") // Set the Content-Type header
                .body(Body::from(message))?;
            response
        }
        Err(err) => err,
    };
    Ok(result)
}
