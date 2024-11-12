use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ApiResponseObject {
    #[serde(flatten)]
    pub properties: Map<String, Value>,
}

impl ApiResponseObject {
    // Constructor function to create an ApiResponseObject from a serde_json::Value
    pub fn new(value: Value) -> Result<Self, &'static str> {
        if let Some(map) = value.as_object() {
            Ok(ApiResponseObject {
                properties: map.clone(),
            })
        } else {
            Err("Provided value is not an object")
        }
    }
}
