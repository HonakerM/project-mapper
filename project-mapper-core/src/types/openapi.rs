use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPISchema(serde_json::Value);

impl TryFrom<serde_json::Value> for OpenAPISchema {
    type Error = String;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        if value.get("type") != Some(&serde_json::Value::String("object".to_string())) {
            return Err(
                "Invalid OpenAPI format: missing 'type' field or not an object".to_string(),
            );
        }
        Ok(OpenAPISchema(value))
    }
}

impl Default for OpenAPISchema {
    fn default() -> Self {
        OpenAPISchema(serde_json::json!({
            "type": "object",
            "properties": {},
        }))
    }
}
