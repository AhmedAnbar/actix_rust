use std::future;

use actix_web::{FromRequest, HttpMessage as _};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    core::enums::UserRole, model::user::UserModel,
    schema::response::api_response_error::ApiResponseError,
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[allow(non_snake_case)]
pub struct UserModelResponse {
    pub id: String,
    pub name: String,
    pub mobile: String,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub role: String,
    pub active: bool,
    pub protected: bool,
    pub createdAt: chrono::DateTime<chrono::Utc>,
    pub updatedAt: chrono::DateTime<chrono::Utc>,
}

impl UserModelResponse {
    pub fn filter_db(user: &mut UserModel) -> Self {
        let role = match UserRole::from_i32(user.role_id) {
            Some(user_role) => user_role.to_str().to_owned(),
            None => "unknown".to_string(),
        };
        println!("User role1: {}", role);
        println!("User role2: {}", user.role_id);
        Self {
            id: user.id.to_owned(),
            name: user.name.to_owned(),
            mobile: user.mobile.to_owned(),
            email: user.email.to_owned(),
            gender: user.gender.to_owned(),
            role,
            active: user.active != 0,
            protected: user.protected != 0,
            createdAt: user.created_at.unwrap(),
            updatedAt: user.updated_at.unwrap(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
#[allow(non_snake_case)]
pub struct AuthUser {
    pub id: String,
    pub name: String,
    pub mobile: String,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub role_id: i32,
    pub active: bool,
    pub protected: bool,
    pub createdAt: chrono::DateTime<chrono::Utc>,
    pub updatedAt: chrono::DateTime<chrono::Utc>,
}

impl AuthUser {
    pub fn filter_db(user: &mut UserModel) -> Self {
        Self {
            id: user.id.to_owned(),
            name: user.name.to_owned(),
            mobile: user.mobile.to_owned(),
            email: user.email.to_owned(),
            gender: user.gender.to_owned(),
            role_id: user.role_id,
            active: user.active != 0,
            protected: user.protected != 0,
            createdAt: user.created_at.unwrap(),
            updatedAt: user.updated_at.unwrap(),
        }
    }
}
impl FromRequest for AuthUser {
    type Error = ApiResponseError;

    type Future = future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> std::future::Ready<Result<AuthUser, ApiResponseError>> {
        match req.extensions().get::<AuthUser>() {
            Some(auth) => future::ready(Ok((*auth).clone())),
            None => future::ready(Err(ApiResponseError::new(
                400,
                "Bad Auth Data".to_string(),
                None,
            ))),
        }
    }
}
