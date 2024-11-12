use actix_web::{patch, web};

use crate::{
    core::app_state::AppState,
    model::user::UserModel,
    schema::{
        project::profile::update_profile::UpdateProfileSchema,
        response::{
            admin::users::AuthUser, api_response::ApiResponse,
            api_response_error::ApiResponseError, api_response_object::ApiResponseObject,
            project::profile::ProfileResponse,
        },
    },
};

// Endpoint to update user profile
#[utoipa::path(
    patch,
    path = "/api/profile/update",
    tag = "Profile Endpoint",
    request_body(content = UpdateProfileSchema, description = "Credentials to update profile", example = json!({"name": "Ahmed","mobile": "0123456789","email": "ahmed@example.com","gender": "Male"})),
    responses(
        (status = 204, description= "Profile Updated", body = ApiResponse),       
        (status = 210, description= "Profile Is Protected", body = ApiResponseError),       
        (status = 401, description= "Unauthorized", body = ApiResponseError),       
        (status = 404, description= "User Not Found", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    ),
    security(
       ("auth_token" = [])
   )
)]
#[patch("/update")]
pub async fn update_profile_handler(
    auth: AuthUser,
    app_state: web::Data<AppState>,
    body: web::Json<UpdateProfileSchema>,
) -> Result<ApiResponse, ApiResponseError> {
    let user_id = auth.id;

    // Query user from the database based on the user ID from JWT claims
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&app_state.pool)
        .await
        .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {:?}", e), None));

    // Handle query result
    let user = match query_result {
        Ok(user) => user,
        Err(e) => {
            return Err(ApiResponseError::new(
                404,
                format!("User not found: {:?}", e),
                None,
            ));
        }
    };

    // Check if the user is protected (cannot be updated)
    if user.protected == 1 {
        return Err(ApiResponseError::new(
            210,
            "User is protected".to_string(),
            None,
        ));
    }

    // Update user profile in the database
    let update_result =
        sqlx::query("UPDATE users SET name = ?, mobile = ?, email = ?, gender = ? WHERE id = ?")
            .bind(body.name.to_owned().unwrap_or_else(|| user.name.clone()))
            .bind(
                body.mobile
                    .to_owned()
                    .unwrap_or_else(|| user.mobile.clone()),
            )
            .bind(
                body.email
                    .to_owned()
                    .unwrap_or_else(|| user.email.clone().unwrap()),
            )
            .bind(
                body.gender
                    .to_owned()
                    .unwrap_or_else(|| user.gender.clone().unwrap()),
            )
            .bind(user_id.clone())
            .execute(&app_state.pool)
            .await;

    // Handle update result
    match update_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                return Err(ApiResponseError::new(
                    500,
                    format!("User not updated"),
                    None,
                ));
            }
        }
        Err(e) => {
            return Err(ApiResponseError::new(
                500,
                format!("Internal server error: {:?}", e),
                None,
            ));
        }
    }

    // Query the updated user profile from the database
    let updated_user = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&app_state.pool)
        .await;

    // Handle query result for updated user profile
    match updated_user {
        Ok(user) => {
            let user_response = ApiResponseObject::new(serde_json::json!({
                "profile": ProfileResponse::new(&user)
            }))
            .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;
            return Ok(ApiResponse::new(
                200,
                "Profile Updated".to_string(),
                Some(user_response),
            ));
        }
        Err(e) => {
            return Err(ApiResponseError::new(
                500,
                format!("Internal server error: {:?}", e),
                None,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::CONFIG,
        core::app_state::AppState,
        middlewares::auth_middleware::RequireAuth,
        model::user::UserModel,
        routes,
        schema::{
            project::profile::update_profile::UpdateProfileSchema,
            response::api_response::ApiResponse,
        },
    };
    use actix_web::{test, web, App};
    use fake::{
        faker::{internet::en::SafeEmail, name::en::Name},
        Fake,
    };
    use rand::Rng;
    use sqlx::MySqlPool;

    const USER_ID: &str = "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b";

    fn generate_jwt() -> String {
        let expire = chrono::Duration::minutes(60);
        let now = chrono::Utc::now();
        let claims = crate::core::utils::jwt::Claims {
            exp: (now + expire).timestamp() as usize,
            iat: now.timestamp() as usize,
            id: USER_ID.to_owned(),
        };
        let encoding_key = jsonwebtoken::EncodingKey::from_secret(CONFIG.jwt.secret.as_ref());
        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &encoding_key).unwrap()
    }

    pub async fn create_test_app_state() -> web::Data<AppState> {
        // Setup database connection pool
        let database_url = CONFIG.clone().database.url;
        let pool = MySqlPool::connect(&database_url)
            .await
            .expect("Failed to create pool");

        // Create app state
        let app_state = web::Data::new(AppState { pool });
        app_state
    }

    #[actix_web::test]
    async fn test_update_profile_handler() {
        let app_state = create_test_app_state().await;

        // Create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .wrap(RequireAuth {})
                .service(web::scope("/api").configure(routes::project::profile::config)),
        )
        .await;

        let jwt = generate_jwt();

        let name: Option<String> = Some(Name().fake());
        let mobile = Some(format!(
            "9665{}",
            rand::thread_rng().gen_range(10000000..99999999)
        ));
        let email: Option<String> = Some(SafeEmail().fake());
        // Update profile data
        let update_profile_data = UpdateProfileSchema {
            name: name.clone(),
            mobile: mobile.clone(),
            email: email.clone(),
            gender: Some("Male".to_string()),
        };

        let req = test::TestRequest::patch()
            .uri("/api/profile/update")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .set_json(update_profile_data)
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Profile Updated");

        // Verify the user was updated in the database
        let updated_user = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", USER_ID)
            .fetch_one(&app_state.pool)
            .await
            .expect("Failed to fetch updated user");

        assert_eq!(updated_user.name, name.unwrap());
        assert_eq!(updated_user.mobile, mobile.unwrap());
        assert_eq!(updated_user.email, Some(email.unwrap()));
        assert_eq!(updated_user.gender, Some("Male".to_string()));
    }
}
