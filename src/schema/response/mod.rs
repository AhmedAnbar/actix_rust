use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod admin;
pub mod api_response;
pub mod api_response_collection;
pub mod api_response_error;
pub mod api_response_object;
pub mod project;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Pagination {
    pub total_items: i64,
    pub total_pages: i64,
    pub current_page: i64,
    pub per_page: i64,
}
