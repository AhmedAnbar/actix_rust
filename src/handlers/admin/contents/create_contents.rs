use crate::{
    core::app_state::AppState,
    model::{
        content::{ContentModel, ContentModelResponse},
        user::UserModel,
    },
    schema::{
        admin::content::CreateContentSchema,
        response::{
            admin::users::AuthUser, api_response::ApiResponse,
            api_response_error::ApiResponseError, api_response_object::ApiResponseObject,
        },
    },
};
use actix_web::{post, web};
use serde_json::json;

// Endpoint metadata using `utoipa` attributes for API documentation
#[utoipa::path(
    post,
    path = "/admin/contents/create",
    tag = "Admin: Contents Endpoint",
    request_body(content = CreateContentSchema, description = "Credentials to create content", example = json!({"content_type": "page", "title": "test page","summary": "test summary", "details": "test details", "configurations": json!({"property1": "value", "property2": json!({"sub-property": "value"})})})),
    responses(
        (status = 201, description= "Content created", body = ApiResponse),       
        (status = 409, description= "Duplicate entry", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    ),
    security(
       ("auth_token" = [])
   )
)]
#[post("/create")]
pub async fn create_contents_handler(
    data: web::Json<CreateContentSchema>, // JSON request body as `CreateContentSchema`
    auth: AuthUser,                       // JWT claims extracted from authorization token
    app_state: web::Data<AppState>, // Shared application state containing database connection pool
) -> Result<ApiResponse, ApiResponseError> {
    // Generate a new UUID for content ID
    let content_id = uuid::Uuid::new_v4().to_string();

    // Execute SQL query to insert new content into database
    let insert_result = sqlx::query(
        "INSERT INTO contents (id, content_type, title, summary, details, created_by) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&content_id) // Binds content ID
    .bind(&data.content_type) // Binds content type
    .bind(&data.title) // Binds title
    .bind(data.summary.as_deref()) // Binds summary if present
    .bind(data.details.as_deref()) // Binds details if present
    .bind(&auth.id) // Binds creator's ID from JWT claims
    .execute(&app_state.pool) // Executes query against database pool
    .await;

    // Handle insert result
    match insert_result {
        Ok(_) => {
            // Fetch newly created content from database
            let mut content = sqlx::query_as!(
                ContentModel,
                "SELECT * FROM contents WHERE id = ?",
                content_id
            )
            .fetch_one(&app_state.pool)
            .await
            .map_err(|err| {
                ApiResponseError::new(500, format!("Internal Server Error: {:?}", err), None)
            })?;

            // Fetch creator details from database
            let created_user =
                sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", auth.id)
                    .fetch_one(&app_state.pool)
                    .await
                    .map_err(|err| {
                        ApiResponseError::new(
                            500,
                            format!("Internal Server Error: {:?}", err),
                            None,
                        )
                    })?;

            // Generate response with filtered content details
            let response = ContentModelResponse::filter_db(&mut content, &created_user);
            let content_response = ApiResponseObject::new(serde_json::json!({"content": response}))
                .map_err(|err| ApiResponseError::new(500, err.to_string(), None))?;

            // Return success response with created content details
            Ok(ApiResponse::new(
                201,
                "Content Created".to_string(),
                Some(content_response),
            ))
        }
        Err(err) => {
            // Handle specific error case for duplicate entry
            if err.to_string().contains("Duplicate entry") {
                Err(ApiResponseError::new(
                    409,
                    "Title already exists".to_string(),
                    None,
                ))
            } else {
                // Return general internal server error for other errors
                Err(ApiResponseError::new(
                    500,
                    format!("Internal Server Error: {:?}", err),
                    None,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::CONFIG, routes};

    use super::*;
    use actix_web::{test, App};
    use fake::{
        faker::lorem::en::{Sentence, Word},
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
    async fn test_create_contents_handler() {
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

        // Prepare request body
        let create_content_data = CreateContentSchema {
            content_type: "page".to_string(),
            title: Word().fake(),
            summary: Some(Sentence(5..10).fake()),
            details: Some(Sentence(10..15).fake()),
        };

        // Send POST request to create content
        let req = test::TestRequest::post()
            .uri("/admin/contents/create")
            .set_json(&create_content_data)
            .insert_header(("Authorization", format!("Bearer {}", jwt)))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        // Verify the response
        assert_eq!(resp.status, 201);
        assert_eq!(resp.message, "Content Created");
    }
}
