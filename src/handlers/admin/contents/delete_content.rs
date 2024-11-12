use actix_web::{delete, web};
use uuid::Uuid;

use crate::{
    core::app_state::AppState, // Import application state AppState
    schema::response::{api_response::ApiResponse, api_response_error::ApiResponseError}, // Import ApiResponse and ApiResponseError from response module
};

// Endpoint metadata using `utoipa` attributes for API documentation
#[utoipa::path(
    delete,
    path = "/admin/contetns/delete/{id}",
    tag = "Admin: Contents Endpoint",
    params(
        ("id" = Uuid, Path, description = "UUID of the content"),
    ),
    responses(
        (status = 204, description= "Content Deleted", body = ApiResponse),       
        (status = 404, description= "Content Not Found", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    ),
    security(
       ("auth_token" = []) 
   )
)]
#[delete("/delete/{id}")] // HTTP DELETE method endpoint
pub async fn delete_contents_handler(
    path: web::Path<Uuid>,
    app_state: web::Data<AppState>,
) -> Result<ApiResponse, ApiResponseError> {
    let content_id = path.into_inner().to_string();

    // Execute SQL DELETE query for content ID
    let query_result = sqlx::query!("DELETE FROM contents WHERE id = ?", content_id)
        .execute(&app_state.pool)
        .await;

    // Match query result for handling success or error cases
    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                // If no rows affected by the DELETE query
                return Err(ApiResponseError::new(
                    404,
                    format!("No data found with id {}", content_id),
                    None,
                ));
            } else {
                // If rows affected, indicating successful deletion
                return Ok(ApiResponse::new(204, format!("Content deleted"), None));
            }
        }
        Err(e) => {
            // Handle SQL execution errors
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
        config::CONFIG, model::content::ContentModel, routes,
        schema::admin::content::CreateContentSchema,
    };

    use super::*;
    use actix_web::{test, App};
    use fake::{
        faker::lorem::en::{Paragraph, Sentence},
        Fake,
    };
    use sqlx::MySqlPool;

    // Function to generate JWT token for authentication
    fn generate_jwt(user_id: &str) -> String {
        let expire = chrono::Duration::minutes(60);
        let now = chrono::Utc::now();
        let claims = crate::core::utils::jwt::Claims {
            exp: (now + expire).timestamp() as usize,
            iat: now.timestamp() as usize,
            id: user_id.to_owned(),
        };
        let encoding_key = jsonwebtoken::EncodingKey::from_secret(CONFIG.jwt.secret.as_ref());
        jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &encoding_key).unwrap()
    }

    // Function to create test AppState
    async fn create_test_app_state() -> web::Data<AppState> {
        // Setup database connection pool
        let database_url = CONFIG.clone().database.url;
        let pool = MySqlPool::connect(&database_url)
            .await
            .expect("Failed to create pool");

        // Create app state
        web::Data::new(AppState { pool })
    }

    #[actix_web::test]
    async fn test_delete_contents_handler() {
        // Create test app state
        let app_state = create_test_app_state().await;

        // Create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .service(web::scope("/admin").configure(routes::admin::content::config)),
        )
        .await;

        // Generate JWT token for authentication
        let user_id = "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b";
        let jwt = generate_jwt(user_id);

        let create_content_data = CreateContentSchema {
            content_type: "page".to_string(),
            title: Paragraph(1..3).fake(),
            summary: Some(Sentence(5..10).fake()),
            details: Some(Sentence(10..15).fake()),
        };
        let content_id = uuid::Uuid::new_v4().to_string();
        let _insert_result = sqlx::query(
            "INSERT INTO contents (id, title, content_type, summary, details, created_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&content_id)
        .bind(&create_content_data.title)
        .bind(&create_content_data.content_type)
        .bind(create_content_data.summary.as_deref())
        .bind(create_content_data.details.as_deref())
        .bind("a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b".to_string())
        .execute(&app_state.pool)
        .await
        .expect("Failed to insert test user");

        let req = test::TestRequest::delete()
            .uri(&format!("/admin/contents/delete/{}", content_id))
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        // Verify the response
        assert_eq!(resp.status, 204);
        assert_eq!(resp.message, "Content deleted");

        // Verify the content was deleted from the database
        let deleted_content = sqlx::query_as!(
            ContentModel,
            "SELECT * FROM contents WHERE id = ?",
            content_id
        )
        .fetch_optional(&app_state.pool)
        .await
        .expect("Failed to fetch deleted user");

        assert!(
            deleted_content.is_none(),
            "Content should not exist in the database"
        );
    }
}
