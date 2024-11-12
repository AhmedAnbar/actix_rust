use crate::schema::response::{
    admin::users::UserModelResponse, api_response::ApiResponse,
    api_response_error::ApiResponseError, api_response_object::ApiResponseObject,
};
use actix_web::{get, web};
use uuid::Uuid;

use crate::{core::app_state::AppState, model::user::UserModel};

// Endpoint metadata using `utoipa` attributes for API documentation
#[utoipa::path(
    get,
    path = "/admin/users/{id}",
    tag = "Admin: Users Endpoint",
    params(
        ("id" = Uuid, Path, description = "UUID of the user to get", example = "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b")
    ),
    responses(
        (status = 200, description= "Get User By ID", body = ApiResponse),
        (status = 404, description= "User Not Found", body = ApiResponseError),
        (status = 500, description= "Internal Server Error", body = ApiResponseError),
    ),
    security(
       ("auth_token" = [])
   )
)]
#[get("/{id}")]
pub async fn get_user_by_id_handler(
    path: web::Path<Uuid>,     // Extracts the `Uuid` path parameter into `path`
    data: web::Data<AppState>, // Shared application state containing database connection pool
) -> Result<ApiResponse, ApiResponseError> {
    let user_id = path.into_inner().to_string(); // Convert `Uuid` to string for SQL query
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&data.pool) // Execute query using database connection pool from `AppState`
        .await;

    match query_result {
        Ok(mut user) => {
            // If user is found, create API response with user data
            let user_response = ApiResponseObject::new(
                serde_json::json!({"user": UserModelResponse::filter_db(&mut user)}),
            )
            .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;
            return Ok(ApiResponse::new(
                200,
                "Get User By Id".to_string(),
                Some(user_response),
            ));
        }
        Err(sqlx::Error::RowNotFound) => {
            // If user not found, return 404 error
            return Err(ApiResponseError::new(
                404,
                format!("User with ID: {} not found", user_id),
                None,
            ));
        }
        Err(e) => {
            // Handle other SQL errors and return 500 error
            return Err(ApiResponseError::new(404, format!("{:?}", e), None));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::CONFIG,
        core::app_state::AppState,
        middlewares::auth_middleware::RequireAuth,
        routes,
        schema::{admin::user::CreateUserSchema, response::api_response::ApiResponse},
    };
    use actix_web::{test, web, App};
    use fake::{
        faker::{internet::en::SafeEmail, name::en::Name},
        Fake,
    };
    use rand::Rng;
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
    async fn test_get_user_by_id_handler() {
        let app_state = create_test_app_state().await;

        // Create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .wrap(RequireAuth {})
                .service(web::scope("/admin").configure(routes::admin::user::config)),
        )
        .await;

        let jwt = generate_jwt();

        // Insert test user data into the database
        let user_id = uuid::Uuid::new_v4().to_string();
        let mobile = format!("9665{}", rand::thread_rng().gen_range(10000000..99999999));
        let create_user_data = CreateUserSchema {
            name: Some(Name().fake()),
            mobile: mobile.clone(),
            email: Some(SafeEmail().fake()),
        };

        let _insert_result = sqlx::query(
            "INSERT INTO users (id, name, mobile, email, configurations, protected) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(&create_user_data.name.as_deref())
        .bind(&create_user_data.mobile)
        .bind(create_user_data.email.as_deref())
        .bind(0) // Not protected
        .execute(&app_state.pool)
        .await
        .expect("Failed to insert test user");

        let req = test::TestRequest::get()
            .uri(&format!("/admin/users/{}", user_id))
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Get User By Id");
    }
}
