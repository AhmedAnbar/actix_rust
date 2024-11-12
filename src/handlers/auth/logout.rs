use crate::schema::response::{api_response::ApiResponse, api_response_error::ApiResponseError};
use actix_web::cookie::{time, Cookie};
use actix_web::{HttpResponse, Responder};

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Auth Endpoint",
    responses(
        (status = 200, description= "User logged out successfully", body = ApiResponse),
        (status = 500, description= "Internal Server Error", body = ApiResponseError),
    ),
    security(
       ("auth_token" = [])
   )
)]
// #[post("/logout")]
pub async fn logout_user_handler() -> Result<impl Responder, ApiResponseError> {
    let logout_cookie = Cookie::build("auth_token", "")
        .path("/")
        .secure(false)
        .http_only(true)
        .max_age(time::Duration::seconds_f32(0.5))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(logout_cookie)
        .json(ApiResponse::new(
            200,
            "User logged out successfully".to_string(),
            None,
        )))
}
