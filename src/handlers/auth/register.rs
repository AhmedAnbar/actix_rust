use crate::schema::response::{
    api_response::ApiResponse, api_response_error::ApiResponseError,
    api_response_object::ApiResponseObject,
};
use actix_web::{post, web};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    core::{app_state::AppState, utils::transform_mobile::validate_and_transform_mobile},
    model::user::UserModel,
};

#[derive(Deserialize, ToSchema)]
pub struct RegisterUserRequest {
    pub name: Option<String>,
    pub mobile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Auth Endpoint",
    request_body(content = RegisterUserRequest, description = "Credentials to create new user", example = json!({"name": "Ahmed", "mobile": "+201018898522", "email": "ahmed@example.com", "gender": "Male"})),
    responses(
        (status = 201, description= "User Created", body = ApiResponse),       
        (status = 409, description= "Duplicate entry", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    )
)]
#[post("/register")]
pub async fn register_user_handler(
    app_state: web::Data<AppState>,
    data: web::Json<RegisterUserRequest>,
) -> Result<ApiResponse, ApiResponseError> {
    let user_id = uuid::Uuid::new_v4().to_string();

    let mobile = match validate_and_transform_mobile(data.mobile.as_str()) {
        Ok(mobile) => mobile,
        Err(e) => return Err(ApiResponseError::new(400, format!("{:?}", e), None)),
    };

    let query_result =
        sqlx::query("INSERT INTO users (id,name,mobile,email, gender) VALUES (?, ?, ?, ?, ?)")
            .bind(user_id.clone())
            .bind(data.name.to_owned().unwrap_or_default())
            .bind(mobile)
            .bind(data.email.to_owned().unwrap_or_default())
            .bind(data.gender.to_owned().unwrap_or_default())
            .execute(&app_state.pool)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            return Err(ApiResponseError::new(
                409,
                "User already exists".to_string(),
                None,
            ));
        }
        return Err(ApiResponseError::new(500, format!("{:?}", err), None));
    }

    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&app_state.pool)
        .await
        .map_err(|err| {
            ApiResponseError::new(500, format!("Internal Server Error: {:?}", err), None)
        });

    match query_result {
        Ok(user) => {
            let user_response = ApiResponseObject::new(serde_json::json!({"user": user}))
                .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;
            return Ok(ApiResponse::new(
                201,
                "User Created. Now you can login with your mobile".to_string(),
                Some(user_response),
            ));
        }
        Err(e) => return Err(ApiResponseError::new(500, format!("{:?}", e), None)),
    }
}
