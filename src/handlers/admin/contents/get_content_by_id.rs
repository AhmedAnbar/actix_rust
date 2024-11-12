use crate::{
    model::content::{ContentModel, ContentModelResponse}, // Import ContentModel and ContentModelResponse from content module
    schema::response::{
        api_response::ApiResponse, api_response_error::ApiResponseError,
        api_response_object::ApiResponseObject,
    }, // Import ApiResponse and ApiResponseError from response module
};
use actix_web::{get, web}; // Import get macro and web module from Actix Web
use uuid::Uuid; // Import Uuid type

use crate::{core::app_state::AppState, model::user::UserModel}; // Import AppState and UserModel from core and model modules

// Endpoint metadata using `utoipa` attributes for API documentation
#[utoipa::path(
    get,
    path = "/admin/contents/{id}", // HTTP GET method endpoint path
    tag = "Admin: Contents Endpoint", // Endpoint tag for documentation
    params(
        ("id" = Uuid, Path, description = "UUID of the content", example = "1f34e48a-d5b1-4bfa-9f10-9345d0a66a1d") // Parameter metadata for content ID
    ),
    responses(
        (status = 200, description= "Get Content By ID", body = ApiResponse), // Response metadata for successful retrieval
        (status = 404, description= "Data Not Found", body = ApiResponseError), // Response metadata for not found scenario
        (status = 500, description= "Internal Server Error", body = ApiResponseError), // Response metadata for internal server error
    ),
    security(
       ("auth_token" = []) // Security requirement: auth_token is required
   )
)]
#[get("/{id}")] // HTTP GET method endpoint
pub async fn get_content_by_id_handler(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> Result<ApiResponse, ApiResponseError> {
    let content_id = path.into_inner().to_string();

    // Execute SQL query to fetch content by ID
    let query_result = sqlx::query_as!(
        ContentModel,
        "SELECT * FROM contents WHERE id = ?",
        content_id
    )
    .fetch_one(&data.pool)
    .await;

    // Match query result for handling success or error cases
    match query_result {
        Ok(mut content) => {
            // If content successfully fetched
            // Fetch user who created the content
            let created_user = sqlx::query_as!(
                UserModel,
                "SELECT * FROM users WHERE id = ?", // SQL query to select user by ID
                content.created_by                  // Use content's created_by field as user ID
            )
            .fetch_one(&data.pool) // Fetch single user row from database using app_state pool
            .await
            .map_err(|e| {
                // Handle potential error fetching user
                ApiResponseError::new(500, format!("Internal Server Error: {:?}", e), None)
                // Return internal server error if user fetch fails
            })?;

            // Prepare response object with content and creator user details
            let content_response = ApiResponseObject::new(
                serde_json::json!({"content": ContentModelResponse::filter_db(&mut content, &created_user)}),
            )
            .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;

            // Return success response with content details
            return Ok(ApiResponse::new(
                200,
                "Get Content By Id".to_string(),
                Some(content_response),
            ));
        }
        Err(sqlx::Error::RowNotFound) => {
            // If content not found
            return Err(ApiResponseError::new(
                404,
                format!("Content with ID: {} not found", content_id),
                None,
            ));
        }
        Err(e) => {
            // Handle other SQL query errors
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
        schema::{admin::content::CreateContentSchema, response::api_response::ApiResponse},
    };
    use actix_web::{test, web, App};
    use fake::{
        faker::lorem::en::{Sentence, Word},
        Fake,
    };
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
    async fn test_get_contetn_by_id_handler() {
        let app_state = create_test_app_state().await;

        // Create and configure the test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .wrap(RequireAuth {})
                .service(web::scope("/admin").configure(routes::admin::content::config)),
        )
        .await;

        let jwt = generate_jwt();

        // Insert test data into the database
        let create_content_data = CreateContentSchema {
            content_type: "page".to_string(),
            title: Word().fake(),
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

        let req = test::TestRequest::get()
            .uri(&format!("/admin/contents/{}", content_id))
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Get Content By Id");
    }
}
