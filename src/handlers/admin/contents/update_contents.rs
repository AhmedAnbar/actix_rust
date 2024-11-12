use crate::{
    core::app_state::AppState,
    model::{
        content::{ContentModel, ContentModelResponse},
        user::UserModel,
    },
    schema::admin::content::UpdateContentSchema,
    schema::response::{
        api_response::ApiResponse, api_response_error::ApiResponseError,
        api_response_object::ApiResponseObject,
    },
};
use actix_web::{put, web};
use serde_json::json;
use uuid::Uuid;

// Endpoint metadata using `utoipa` attributes for API documentation
#[utoipa::path(
    put,
    path = "/admin/contents/update/{id}",
    tag = "Admin: Contents Endpoint",
    params(
        ("id" = Uuid, Path, description = "UUID of the content"),
    ),
    request_body(content = UpdateContentSchema, description = "Content data to update", example = json!({"title": "Updated title", "configurations": json!({"property1": "updated_value", "property2": json!({"sub-property": "updated_value"})})})),
    responses(
        (status = 200, description= "Content updated", body = ApiResponse),
        (status = 404, description= "Content not found", body = ApiResponseError),
        (status = 409, description= "Duplicate entry", body = ApiResponseError),
        (status = 500, description= "Internal Server Error", body = ApiResponseError),
    ),
    security(
       ("auth_token" = [])
   )
)]
#[put("/update/{id}")]
pub async fn update_contents_handler(
    id: web::Path<Uuid>,
    data: web::Json<UpdateContentSchema>,
    app_state: web::Data<AppState>,
) -> Result<ApiResponse, ApiResponseError> {
    let content_id = id.into_inner().to_string();

    // Fetch the existing content to merge configurations
    let existing_content = sqlx::query_as!(
        ContentModel,
        "SELECT * FROM contents WHERE id = ?",
        content_id
    )
    .fetch_one(&app_state.pool)
    .await
    .map_err(|_| ApiResponseError::new(404, "Content not found".to_string(), None))?;

    let created_user = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE id = ?",
        existing_content.created_by
    )
    .fetch_one(&app_state.pool)
    .await
    .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {:?}", e), None))?; // Fetch user who created the existing content

    // Merge existing configurations with new configurations

    // Update the contents table
    let query_result = sqlx::query!(
        "UPDATE contents SET content_type = COALESCE(?, content_type), title = COALESCE(?, title), summary = COALESCE(?, summary), details = COALESCE(?, details), updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        data.content_type,
        data.title,
        data.summary,
        data.details,
        content_id
    )
    .execute(&app_state.pool)
    .await; // Execute SQL update query

    if let Err(err) = query_result {
        // Handle SQL update query error
        let error_message = err.to_string();
        if error_message.contains("Duplicate entry") {
            // If duplicate entry error
            return Err(ApiResponseError::new(
                409,
                "Title already exists".to_string(),
                None,
            ));
        }
        return Err(ApiResponseError::new(
            500,
            format!("Internal Server Error: {:?}", err),
            None,
        ));
    }

    // Fetch updated content from database
    let mut updated_content = sqlx::query_as!(
        ContentModel,
        "SELECT * FROM contents WHERE id = ?",
        content_id
    )
    .fetch_one(&app_state.pool)
    .await
    .map_err(|err| ApiResponseError::new(500, format!("Internal Server Error: {:?}", err), None))?; // Handle fetch updated content error

    let response = ContentModelResponse::filter_db(&mut updated_content, &created_user); // Filter updated content and creator user details

    let content_response = ApiResponseObject::new(json!({"content": response})) // Create JSON response object
        .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?; // Handle JSON response object creation error

    Ok(ApiResponse::new(
        // Return HTTP response with ApiResponse object
        200,
        "Content Updated".to_string(),
        Some(content_response),
    ))
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
    async fn test_update_contents_handler() {
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

        let update_content_data = UpdateContentSchema {
            content_type: Some("page".to_string()),
            title: Some(Word().fake()),
            summary: Some("Updated Summary".to_string()),
            details: Some("Updated Details".to_string()),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/admin/contents/update/{}", content_id))
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .set_json(&update_content_data)
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        // Verify the response
        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Content Updated");

        // Verify the content was deleted from the database
        let updated_content = sqlx::query_as!(
            ContentModel,
            "SELECT * FROM contents WHERE id = ?",
            content_id
        )
        .fetch_one(&app_state.pool)
        .await
        .expect("Failed to fetch updated content");

        assert_eq!(updated_content.title, update_content_data.title.unwrap());
    }
}
