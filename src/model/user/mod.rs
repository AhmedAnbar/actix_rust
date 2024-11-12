use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow, ToSchema, Clone)]
#[allow(non_snake_case)]
pub struct UserModel {
    pub id: String,
    pub name: String,
    pub mobile: String,
    pub mobile_token: Option<String>,
    pub mobile_token_expire_at: Option<chrono::DateTime<chrono::Utc>>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub role_id: i32,
    pub active: i8,
    pub protected: i8,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[allow(non_snake_case)]
pub struct CreatedByResponse {
    pub id: String,
    pub name: String,
    pub roleId: i32,
    pub active: bool,
}

impl CreatedByResponse {
    pub fn filter_db(user: &UserModel) -> Self {
        Self {
            id: user.id.to_owned(),
            name: user.name.to_owned(),
            roleId: user.role_id.to_owned(),
            active: user.active != 0,
        }
    }
}
