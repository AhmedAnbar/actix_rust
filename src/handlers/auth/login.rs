use crate::{
    config::CONFIG,
    core::mail::email_queue::{EmailJob, EmailQueue},
    schema::response::{
        api_response::ApiResponse,
        api_response_error::{ApiResponseError, ValidationErrorDetail},
        api_response_object::ApiResponseObject,
    },
};
use actix_web::{post, web};
use chrono::{Duration, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::core::{
    app_state::AppState,
    sms::sms_queue::{SmsJob, SmsQueue},
    utils::{generate_opt::generate_otp, transform_mobile::validate_and_transform_mobile},
};

// Structure representing the request body for login
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginUserRequest {
    pub mobile: String,
}

// Structure representing the query result from the database
#[derive(Deserialize, Serialize)]
struct LoginUserQueryResult {
    id: String,
    mobile: String,
    email: Option<String>,
    active: i8,
}

// Endpoint definition and documentation for the login API
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth Endpoint",
    request_body(content = LoginUserRequest, description = "Credentials to generate and send the OTP", example = json!({"mobile": "+201018898522"})),
    responses(
        (status = 200, description= "OTP Generated and sms is sent", body = ApiResponse),       
        (status = 400, description= "Validation Error", body = ApiResponseError),       
        (status = 403, description= "User not active", body = ApiResponseError),       
        (status = 404, description= "User not found", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    )
)]
#[post("/login")]
pub async fn login_user_handler(
    app_state: web::Data<AppState>,
    sms_queue: web::Data<SmsQueue>,
    email_queue: web::Data<EmailQueue>,
    data: web::Json<LoginUserRequest>,
) -> Result<ApiResponse, ApiResponseError> {
    let user_mobile = match validate_and_transform_mobile(data.mobile.as_str()) {
        Ok(mobile) => mobile,
        Err(e) => {
            return Err(ApiResponseError::new(
                400,
                "Validation Error".to_string(),
                Some(vec![ValidationErrorDetail {
                    field: "mobile".to_string(),
                    error: format!("{:?}", e),
                }]),
            ));
        }
    };

    let pool = app_state.clone().pool.clone();

    let query_result = sqlx::query_as!(
        LoginUserQueryResult,
        "SELECT id, mobile, email, active FROM users WHERE mobile = ?",
        user_mobile
    )
    .fetch_one(&pool)
    .await;

    match query_result {
        Ok(user) => {
            if user.active != 1 {
                return Err(ApiResponseError::new(
                    403,
                    "User is not active".to_string(),
                    None,
                ));
            }
            let user_response = ApiResponseObject::new(serde_json::json!({"user": user}))
                .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;

            // Generate OTP
            let otp = generate_otp(&CONFIG);

            // Calculate expiry time (one minute from now)
            let one_minute_from_now = Utc::now() + Duration::minutes(1);

            // Update mobile_token and mobile_token_expire_at columns
            let update_query = sqlx::query(
                "UPDATE users SET mobile_token = ?, mobile_token_expire_at = ? WHERE id = ?",
            )
            .bind(&otp)
            .bind(&one_minute_from_now)
            .bind(&user.id)
            .execute(&pool)
            .await;

            match update_query {
                // Handle the case where the database update is successful
                Ok(_) => {
                    if CONFIG.env.eq_ignore_ascii_case("production") {
                        let job = SmsJob {
                            to: user_mobile.clone(),
                            body: format!("Login OTP: {}", otp),
                        };
                        if let Err(e) = sms_queue.sender.send(job).await {
                            error!("Failed to queue SMS to: {}. Error: {:?}", user_mobile, e);
                            return Err(ApiResponseError::new(
                                500,
                                "Server Error".to_string(),
                                None,
                            ));
                        }
                        if user.email.is_some() {
                            let job = EmailJob {
                                subject: "Login OTP".to_string(),
                                to: user.email.clone().unwrap(),
                                body: format!("Login OTP: {}", otp),
                            };
                            if let Err(e) = email_queue.sender.send(job).await {
                                error!("Failed to queue Email: {:?}", e);
                                return Err(ApiResponseError::new(
                                    500,
                                    "Server Error".to_string(),
                                    None,
                                ));
                            }
                        }
                    }
                    return Ok(ApiResponse::new(
                        200,
                        "OTP generated for this mobile... Sending SMS".to_string(),
                        Some(user_response),
                    ));
                }
                // Handle the case where the database update fails
                Err(err) => {
                    return Err(ApiResponseError::new(
                        500,
                        format!("Failed to update token: {:?}", err),
                        None,
                    ));
                }
            }
        }
        // Handle the case where the user is not found in the database
        Err(e) => {
            return Err(ApiResponseError::new(
                404,
                format!("User not found: {:?}", e),
                None,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{
            mail::email_queue::EmailQueue, sms::sms_queue::SmsQueue,
            utils::test_utils::create_test_app_state,
        },
        handlers::auth::login::LoginUserRequest,
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
    async fn test_login_handler() {
        let app_state = create_test_app_state().await;

        let (sms_queue, _) = SmsQueue::new();
        let (email_queue, _) = EmailQueue::new();
        // create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .app_data(web::Data::new(email_queue.clone()))
                .app_data(web::Data::new(sms_queue.clone()))
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
            "insert into users (id, name, mobile, gender, mobile_token, mobile_token_expire_at, email, configurations, protected, active) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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

        let login_data = LoginUserRequest { mobile };

        let req = test::TestRequest::post()
            .uri(&format!("/api/auth/login"))
            .set_json(&login_data)
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "OTP generated for this mobile... Sending SMS");
    }
}
