use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::user::{CreatedByResponse, UserModel};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, ToSchema, Clone)]
#[allow(non_snake_case)]
pub struct ContentModel {
    pub id: String,
    pub content_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub details: Option<String>,
    pub content_image: Option<String>,
    pub record_state: i8,
    pub protected: i8,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[allow(non_snake_case)]
pub struct ContentModelResponse {
    pub id: String,
    pub content_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub details: Option<String>,
    pub contentImage: Option<String>,
    pub recordState: bool,
    pub protected: bool,
    pub createdAt: chrono::DateTime<chrono::Utc>,
    pub updatedAt: chrono::DateTime<chrono::Utc>,
    pub deletedAt: Option<chrono::DateTime<chrono::Utc>>,
    pub createdBy: CreatedByResponse,
}

impl ContentModelResponse {
    pub fn filter_db(content: &mut ContentModel, user: &UserModel) -> Self {
        Self {
            id: content.id.to_owned(),
            content_type: content.content_type.to_owned(),
            title: content.title.to_owned(),
            summary: content.summary.to_owned(),
            details: content.details.to_owned(),
            contentImage: content.content_image.to_owned(),
            recordState: content.record_state != 0,
            protected: content.protected != 0,
            createdAt: content.created_at.unwrap(),
            updatedAt: content.updated_at.unwrap(),
            deletedAt: content.deleted_at,
            createdBy: CreatedByResponse::filter_db(&user),
        }
    }
}
