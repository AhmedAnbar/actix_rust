use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateContentSchema {
    pub content_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateContentSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
pub struct ContentsFilterOptions {
    #[param(example = 10)]
    pub limit: Option<i64>,
    #[param(example = 1)]
    pub page: Option<i64>,
    #[param(example = "page")]
    pub content_type: Option<String>,
    #[param(example = "content")]
    pub title: Option<String>,
    #[param(example = "false")]
    pub export: Option<bool>,
}
