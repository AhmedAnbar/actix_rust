use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ApplicationModel {
    pub id: String,
    pub app_name: String,
    pub app_version: String,
    pub app_key: String,
    pub app_secret: String,
    pub app_requests: i64,
    pub record_state: i8,
    pub protected: i8,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ApplicationModelResponse {
    pub id: String,
    pub app_name: String,
    pub app_version: String,
    pub app_key: String,
    pub app_secret: String,
    pub app_requests: i64,
    pub record_state: bool,
    pub protected: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
