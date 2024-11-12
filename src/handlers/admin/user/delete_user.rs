use actix_web::{delete, web};
use log::info;
use uuid::Uuid;

use crate::{
    core::app_state::AppState,
    schema::response::{api_response::ApiResponse, api_response_error::ApiResponseError},
};

// Endpoint handler for deleting a user
#[utoipa::path(
    delete,
    path = "/admin/users/delete/{id}",
    tag = "Admin: Users Endpoint",
    // Specify path parameters
    params(
        ("id" = Uuid, Path, description = "UUID of the user to delete"),
    ),
    // Specify possible responses
    responses(
        (status = 204, description= "User Deleted"),       
        (status = 404, description= "User Not Found", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    ),
    // Specify security requirements
    security(
       ("auth_token" = [])
   )
)]
#[delete("/delete/{id}")]
pub async fn delete_user_handler(
    path: web::Path<Uuid>,          // Path parameter representing the user's UUID
    app_state: web::Data<AppState>, // Application state containing database pool
) -> Result<ApiResponse, ApiResponseError> {
    let user_id = path.into_inner().to_string(); // Extract the UUID from the path parameter

    // Execute SQL query to delete user from the database
    let query_result = sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(&app_state.pool)
        .await;

    // Handle the result of the SQL query
    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                // Return a not found error if no user was deleted
                return Err(ApiResponseError::new(
                    404,
                    format!("No user found with id {}", user_id),
                    None,
                ));
            } else {
                // Log the number of rows affected by the delete operation
                info!("Deleted user with id: {}", user_id);
                // Return a successful API response indicating user deletion
                return Ok(ApiResponse::new(204, format!("User deleted"), None));
            }
        }
        Err(e) => {
            // Return a generic internal server error for database errors
            return Err(ApiResponseError::new(
                500,
                format!("Internal server error: {}", e),
                None,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::app_state::AppState,
        middlewares::auth_middleware::RequireAuth,
        routes,
        schema::{admin::user::CreateUserSchema, response::api_response::ApiResponse},
    };
    use actix_web::{test, web, App};
    use fake::faker::{internet::en::SafeEmail, name::en::Name};
    use fake::Fake;
    use rand::Rng as _;
    use sqlx::MySqlPool;

    use crate::{config::CONFIG, model::user::UserModel};

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
    async fn test_delete_user_handler() {
        // Create test app state
        let app_state = create_test_app_state().await;

        // Create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .wrap(RequireAuth {})
                .service(web::scope("/admin").configure(routes::admin::user::config)),
        )
        .await;

        // Generate JWT token for authentication
        let jwt = generate_jwt();

        // Insert a test user into the database
        let user_id = uuid::Uuid::new_v4().to_string();
        let create_user_data = CreateUserSchema {
            name: Some(Name().fake()),
            mobile: format!("9665{}", rand::thread_rng().gen_range(10000000..99999999)),
            email: Some(SafeEmail().fake()),
        };

        let _insert_result = sqlx::query(
            "INSERT INTO users (id, name, mobile, email, protected) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(&create_user_data.name.as_deref())
        .bind(&create_user_data.mobile)
        .bind(create_user_data.email.as_deref())
        .bind(0) // Not protected
        .execute(&app_state.pool)
        .await
        .expect("Failed to insert test user");

        // Send DELETE request to delete the user
        let req = test::TestRequest::delete()
            .uri(&format!("/admin/users/delete/{}", user_id))
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        // Verify the response
        assert_eq!(resp.status, 204);
        assert_eq!(resp.message, "User deleted");

        // Verify the user was deleted from the database
        let deleted_user = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
            .fetch_optional(&app_state.pool)
            .await
            .expect("Failed to fetch deleted user");

        assert!(
            deleted_user.is_none(),
            "User should not exist in the database"
        );
    }
}
