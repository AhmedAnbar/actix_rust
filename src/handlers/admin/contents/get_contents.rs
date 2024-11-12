use crate::{
    core::{app_state::AppState, utils::export_to_csv::export_to_csv},
    model::{
        content::{ContentModel, ContentModelResponse},
        user::UserModel,
    },
    schema::admin::content::ContentsFilterOptions,
    schema::response::{
        api_response_collection::ApiResponseCollection, api_response_error::ApiResponseError,
        api_response_object::ApiResponseObject, Pagination,
    },
};
use actix_web::{get, web, HttpResponse, Responder};

//COMM: Endpoint metadata using `utoipa` attributes for API documentation
#[utoipa::path(
    get,
    path = "/admin/contents", // HTTP GET method endpoint path
    tag = "Admin: Contents Endpoint", // Endpoint tag for documentation
    params(
        ContentsFilterOptions // Parameter type for filtering contents
    ),
    responses(
        (status = 200, description= "Get All Contentes", body = ApiResponse), // Response metadata for successful retrieval
        (status = 401, description= "Unauthorized", body = ApiResponseError), // Response metadata for unauthorized access
        (status = 404, description= "No Data Found", body = ApiResponseError), // Response metadata for no data found
        (status = 500, description= "Internal Server Error", body = ApiResponseError), // Response metadata for internal server error
    ),
    security(
       ("auth_token" = []) // Security requirement: auth_token is required
   )
)]
#[get("")] // HTTP GET method endpoint
pub async fn get_contents_handler(
    opts: web::Query<ContentsFilterOptions>, // Query parameter: ContentsFilterOptions for filtering contents
    app_state: web::Data<AppState>, // Shared application state containing database connection pool
) -> Result<impl Responder, ApiResponseError> {
    let limit = opts.limit.unwrap_or(10); // Extract limit parameter from query or default to 10
    let page = opts.page.unwrap_or(1); // Extract page parameter from query or default to 1
    let offset = (page - 1) * limit; // Calculate offset based on page and limit

    let mut query = "SELECT * FROM contents".to_string(); // Initialize SQL query string to fetch contents
    let mut conditions = Vec::new(); // Initialize vector to store query conditions

    if let Some(ref content_type) = opts.content_type {
        // Check if content_type filter is provided
        conditions.push(format!("content_type = '{}'", content_type)); // Add content_type filter condition to vector
    }
    if let Some(ref title) = opts.title {
        // Check if title filter is provided
        conditions.push(format!("title LIKE '%{}%'", title)); // Add title filter condition to vector
    }

    if !conditions.is_empty() {
        // If there are any conditions in the vector
        query.push_str(" WHERE "); // Append WHERE clause to query string
        query.push_str(&conditions.join(" AND ")); // Append conditions joined by AND to query string
    }

    //COMM: Appends LIMIT and OFFSET to SQL query if export is false
    if !opts.export.unwrap_or(false) {
        query.push_str(" LIMIT ? OFFSET ?");
    }

    let contents: Vec<ContentModel> = sqlx::query_as::<_, ContentModel>(&query) // Execute SQL query to fetch contents
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(&app_state.pool)
        .await
        .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {}", e), None))?; // Handle query execution error

    if contents.is_empty() {
        // If no contents are fetched
        return Err(ApiResponseError::new(
            404,
            "No Data Found".to_string(),
            None,
        ));
    }

    if opts.export.unwrap_or(false) {
        // If export option is enabled in query parameters
        let csv_data =
            export_to_csv(&contents) // Export contents to CSV format
                .map_err(|e| {
                    ApiResponseError::new(500, format!("CSV Export Error: {}", e), None)
                })?; // Handle export error
        return Ok(HttpResponse::Ok().content_type("text/csv").body(csv_data)); // Return CSV data as HTTP response
    }

    let total_count_query = if !conditions.is_empty() {
        // Construct total count query based on conditions
        format!(
            "SELECT COUNT(*) FROM contents WHERE {}",
            conditions.join(" AND ")
        )
    } else {
        "SELECT COUNT(*) FROM contents".to_string()
    };

    let total_count: (i64,) = sqlx::query_as(&total_count_query) // Execute total count query
        .fetch_one(&app_state.pool)
        .await
        .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {}", e), None))?; // Handle total count query error

    let mut content_response = Vec::new(); // Initialize vector to store content responses

    for mut content in contents {
        // Iterate through fetched contents
        let created_user = sqlx::query_as!(
            // Fetch user who created each content
            UserModel,
            "SELECT * FROM users WHERE id = ?",
            content.created_by
        )
        .fetch_one(&app_state.pool)
        .await
        .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {:?}", e), None))?; // Handle user fetch error

        let response = ContentModelResponse::filter_db(&mut content, &created_user); // Filter content and creator user details
        content_response.push(response); // Push filtered response to content_response vector
    }

    let total_items = total_count.0; // Extract total count of items
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64; // Calculate total pages
    let pagination = Pagination {
        // Construct Pagination object
        total_items,
        total_pages,
        current_page: page as i64,
        per_page: limit as i64,
    };

    let json_response = ApiResponseObject::new(serde_json::json!({ // Create JSON response object
        "contents": content_response,
    }))
    .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?; // Handle JSON response object creation error

    Ok(HttpResponse::Ok().json(ApiResponseCollection::new(
        // Return HTTP response with JSON payload
        200,
        "Get All Contents".to_string(),
        Some(json_response),
        Some(pagination),
    )))
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
    async fn test_get_contents_handler() {
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
            .uri("/admin/contents?limit=10&page=1")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Get All Contents");
    }
}
