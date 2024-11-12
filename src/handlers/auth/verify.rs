use crate::schema::response::{
    api_response::ApiResponse, api_response_error::ApiResponseError,
    api_response_object::ApiResponseObject, project::profile::ProfileResponse,
};
use actix_web::{
    cookie::{time, Cookie, SameSite},
    post, web, HttpResponse, Responder,
};
use chrono::{Duration, Utc};
use humantime::format_duration;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    core::{app_state::AppState, utils::jwt::encode_jwt},
    model::user::UserModel,
};

// representing the request body for OTP verification
#[derive(Serialize, Deserialize, ToSchema)]
pub struct VerifyOtpRequest {
    pub mobile: String,
    pub otp: String,
}

// Endpoint definition and documentation for the OTP verification API
#[utoipa::path(
    post,
    path = "/api/auth/verify",
    tag = "Auth Endpoint",
    request_body(content = VerifyOtpRequest, description = "Credentials to verify OTP and generate auth_token", example = json!({"mobile": "+201018898522", "otp": "12345"})),
    responses(
        (status = 200, description= "OTP verified and auth_token is generated", body = ApiResponse),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    )
)]
#[post("/verify")]
pub async fn verify_otp_handler(
    data: web::Json<VerifyOtpRequest>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, ApiResponseError> {
    let mobile = data.mobile.to_owned();
    let otp = data.otp.to_owned();

    // Query the user from the database
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE mobile = ?", mobile)
        .fetch_one(&app_state.pool)
        .await;

    match query_result {
        Ok(user) => {
            // Check if mobile token exists
            if user.mobile_token.is_none() {
                return Err(ApiResponseError::new(
                    400,
                    "Please provide OTP".to_string(),
                    None,
                ));
            }

            // Verify OTP
            if user.mobile_token != Some(otp) {
                return Err(ApiResponseError::new(
                    400,
                    "Incorrect OTP".to_string(),
                    None,
                ));
            }

            // Check mobile token expiration
            if let Some(expiry) = user.mobile_token_expire_at {
                if expiry < chrono::Utc::now() {
                    return Err(ApiResponseError::new(
                        400,
                        "Mobile token has expired".to_string(),
                        None,
                    ));
                }
            } else {
                return Err(ApiResponseError::new(
                    400,
                    "Mobile token expiration not set".to_string(),
                    None,
                ));
            }

            let one_minute_from_now = Utc::now() - Duration::hours(1);
            let update_query_result = sqlx::query(
                "UPDATE users SET mobile_token = ?, mobile_token_expire_at = ? WHERE id = ?",
            )
            .bind(Option::<String>::None)
            .bind(&one_minute_from_now)
            .bind(&user.id)
            .execute(&app_state.pool)
            .await;

            match update_query_result {
                Ok(_) => {
                    // Generate JWT token
                    let token_duration = Duration::days(100);
                    let token = match encode_jwt(user.id.clone(), token_duration) {
                        Ok(token) => token,
                        Err(e) => {
                            return Err(ApiResponseError::new(
                                500,
                                format!("Failed to generate JWT: {:?}", e),
                                None,
                            ));
                        }
                    };

                    // Convert token duration to a human-readable format
                    let human_readable_duration = format_duration(std::time::Duration::from_secs(
                        token_duration.num_seconds() as u64,
                    ))
                    .to_string();
                    let cookie = Cookie::build("auth_token", token.clone())
                        .path("/")
                        .secure(false)
                        .http_only(true)
                        .same_site(SameSite::Strict)
                        .max_age(time::Duration::days(100))
                        .finish();

                    // Construct success response with JWT token
                    let response_body = ApiResponseObject::new(serde_json::json!({
                        "auth_token": token,
                        "token_expires_in": human_readable_duration,
                        "user": ProfileResponse::new(&user),
                    }))
                    .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;

                    // Return the HTTP response with the cookie
                    Ok(HttpResponse::Ok().cookie(cookie).json(ApiResponse::new(
                        200,
                        "User Verified".to_string(),
                        Some(response_body),
                    )))
                }
                Err(e) => {
                    return Err(ApiResponseError::new(
                        500,
                        format!("Failed to update user: {:?}", e),
                        None,
                    ));
                }
            }
        }
        Err(e) => {
            // Handle database query error
            Err(ApiResponseError::new(
                500,
                format!("Failed to fetch user: {:?}", e),
                None,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::utils::test_utils::create_test_app_state,
        handlers::auth::verify::VerifyOtpRequest,
        routes,
        schema::{admin::user::CreateUserSchema, response::api_response::ApiResponse},
    };
    use actix_web::{test, web, App};
    use chrono::{Duration, Utc};
    use fake::{
        faker::{internet::en::SafeEmail, name::en::Name},
        Fake,
    };
    use rand::Rng;
    #[actix_web::test]
    async fn test_verify_handler() {
        let app_state = create_test_app_state().await;

        // create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .service(web::scope("/api").configure(routes::auth::config)),
        )
        .await;

        // insert test user data into the database
        let user_id = uuid::Uuid::new_v4().to_string();
        let mobile_token = "12345"; // Set a test mobile token
        let mobile_token_expire_at = Utc::now() + Duration::minutes(5); // Set an expiration time

        let user_mobile = rand::thread_rng().gen_range(10000000..99999999);
        let mobile = format!("9665{}", user_mobile);
        let create_user_data = CreateUserSchema {
            name: Some(Name().fake()),
            mobile: mobile.clone(),
            email: Some(SafeEmail().fake()),
        };

        let _insert_result = sqlx::query(
            "insert into users (id, name, mobile, gender, mobile_token, mobile_token_expire_at, email, protected, active) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(&create_user_data.name.as_deref())
        .bind(&create_user_data.mobile)
        .bind("Male".to_string())
        .bind(&mobile_token)
        .bind(&mobile_token_expire_at)
        .bind(&create_user_data.email.as_deref())
        .bind(0)
        .bind(1)
        .execute(&app_state.pool)
        .await
        .expect("Failed to insert test user");

        let login_data = VerifyOtpRequest {
            mobile,
            otp: mobile_token.to_string(),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/auth/verify"))
            .set_json(&login_data)
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "User Verified");
    }
}
