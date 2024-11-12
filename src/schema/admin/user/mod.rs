use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateUserSchema {
    pub name: Option<String>,
    pub mobile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct UpdateUserSchema {
    pub name: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub active: Option<bool>,
    pub role_id: Option<i32>,
    pub protected: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, IntoParams)]
pub struct UsersFilterOptions {
    #[param(example = 10)]
    pub limit: Option<i64>,
    #[param(example = 1)]
    pub page: Option<i64>,
    #[param(example = "1234567890")]
    pub mobile: Option<String>,
    #[param(example = "false")]
    pub export: Option<bool>,
}
