use crate::{
    model::user::UserModel,
    schema::response::{
        admin::users::AuthUser, api_response::ApiResponse, api_response_error::ApiResponseError,
        api_response_object::ApiResponseObject, project::profile::ProfileResponse,
    },
};
use actix_web::{get, web};

use crate::core::app_state::AppState;

// Endpoint to fetch user profile data
#[utoipa::path(
    get,
    path = "/api/profile",
    tag = "Profile Endpoint",
    responses(
        (status = 200, description= "Get Profile Data", body = ApiResponse),       
        (status = 401, description= "Unauthorized", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    ),
    security(
       ("auth_token" = [])
   )
)]
#[get("")]
pub async fn profile_handler(
    app_state: web::Data<AppState>, // Application state shared across handlers
    auth: AuthUser,                 // Authentication token data
) -> Result<ApiResponse, ApiResponseError> {
    println!("From profile handler: {}", auth.id);
    // Query user data from the database based on the authenticated user's ID
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", auth.id)
        .fetch_one(&app_state.pool)
        .await
        .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {:?}", e), None));

    // Handle query result
    match query_result {
        Ok(user) => {
            // Construct response object with profile data
            let user_info = ApiResponseObject::new(serde_json::json!({
                "profile": ProfileResponse::new(&user),
            }))
            .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;
            Ok(ApiResponse::new(
                200,
                "Profile Data".to_string(),
                Some(user_info),
            ))
        }
        Err(e) => Err(ApiResponseError::new(
            404,
            format!("No user found from claim id: {:?}", e),
            None,
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::CONFIG, core::app_state::AppState, middlewares::auth_middleware::RequireAuth,
        routes, schema::response::api_response::ApiResponse,
    };
    use actix_web::{test, web, App};
    use sqlx::MySqlPool;

    fn generate_jwt() -> String {
        let expire = chrono::Duration::minutes(60);
        let now = chrono::Utc::now();
        let claims = crate::core::utils::jwt::Claims {
            exp: (now + expire).timestamp() as usize,
            iat: now.timestamp() as usize,
            id: "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b".to_owned(),
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
    async fn test_get_profile_handler() {
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
        println!("JWT: {}", jwt);

        let req = test::TestRequest::get()
            .uri("/api/profile")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Profile Data");
    }
}
