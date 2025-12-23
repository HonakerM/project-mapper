use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPISchema(serde_json::Value);

impl OpenAPISchema {
    pub fn to_json_value(&self) -> serde_json::Value {
        self.0.clone()
    }
}
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPI(serde_json::Value);

impl OpenAPI {
    pub fn to_json_value(&self) -> serde_json::Value {
        self.0.clone()
    }
}
impl TryFrom<serde_json::Value> for OpenAPI {
    type Error = String;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        if value.get("openapi") == None {
            return Err("Invalid OpenAPI format: missing 'openapi' field missing".to_string());
        }
        if value.get("info") == None {
            return Err("Invalid OpenAPI format: missing 'info' field missing".to_string());
        }
        Ok(OpenAPI(value))
    }
}

impl Default for OpenAPI {
    fn default() -> Self {
        OpenAPI(serde_json::json!({
            "opnapi": "3.1.0",
            "info": {
                "title":"ProjectMapper"
            },
        }))
    }
}
