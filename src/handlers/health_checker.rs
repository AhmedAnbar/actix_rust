use actix_web::Responder;

use crate::schema::response::api_response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/check",
    tag = "Health Checker Endpoint",
    responses(
        (status = 200, description= "Health Checker", body = ApiResponse),       
    )
)]
pub async fn health_checker_handler() -> impl Responder {
    let message = "API with Rust, SQLX, MySQL, and Actix web".to_string();

    ApiResponse::new(200, message, None)
}

#[utoipa::path(
    get,
    path = "/api/check/auth",
    tag = "Health Checker Endpoint",
    responses(
        (status = 200, description= "Authenticated Health Checker", body = ApiResponse),       
        (status = 401, description= "Unauthorized", body = ApiResponseError),       
    ),
    security(
       ("auth_token" = [])
   )
)]
pub async fn health_checker_auth_handler() -> impl Responder {
    let message = "API with Rust, SQLX, MySQL, and Actix web - Authinticated".to_string();

    ApiResponse::new(200, message, None)
}

#[cfg(test)]
mod tests {
    use crate::core::app_state::AppState;
    use crate::core::utils::jwt::Claims;
    use crate::routes;
    use crate::schema::response::api_response::ApiResponse;
    use crate::{config::CONFIG, middlewares::auth_middleware::RequireAuth};

    use actix_web::http::header;
    use actix_web::{test, web, App};
    use chrono::{Duration, Utc};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use sqlx::MySqlPool;

    fn generate_jwt() -> String {
        let expire = Duration::minutes(60);
        let now = Utc::now();
        let claims = Claims {
            exp: (now + expire).timestamp() as usize,
            iat: now.timestamp() as usize,
            id: "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b".to_owned(),
        };
        let encoding_key = EncodingKey::from_secret(CONFIG.jwt.secret.as_ref());
        encode(&Header::default(), &claims, &encoding_key).unwrap()
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
    async fn test_health_checker_handler() {
        let app = test::init_service(
            App::new().service(web::scope("/api").configure(routes::health_checker::config)),
        )
        .await;
        let req = test::TestRequest::get().uri("/api/check").to_request();
        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "API with Rust, SQLX, MySQL, and Actix web");
    }

    #[actix_web::test]
    async fn test_health_checker_auth_handler() {
        // Create app state
        let app_state = create_test_app_state().await;

        // Create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .wrap(RequireAuth {})
                .service(web::scope("/api").configure(routes::health_checker::config)),
        )
        .await;

        let jwt = generate_jwt();

        let req = test::TestRequest::get()
            .uri("/api/check/auth")
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", jwt)))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(
            resp.message,
            "API with Rust, SQLX, MySQL, and Actix web - Authinticated"
        );
    }
}
