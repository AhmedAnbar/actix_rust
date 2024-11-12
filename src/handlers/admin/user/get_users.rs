use actix_web::{get, web, HttpResponse, Responder};

use crate::{
    core::{app_state::AppState, utils::export_to_csv::export_to_csv},
    model::user::UserModel,
    schema::{
        admin::user::UsersFilterOptions,
        response::{
            admin::users::UserModelResponse, api_response_collection::ApiResponseCollection,
            api_response_error::ApiResponseError, api_response_object::ApiResponseObject,
            Pagination,
        },
    },
};

// Endpoint metadata using `utoipa` attributes for API documentation
#[utoipa::path(
    get,
    path = "/admin/users",
    tag = "Admin: Users Endpoint",
    params(
        UsersFilterOptions
    ),
    responses(
        (status = 200, description= "Get All Users", body = ApiResponse),
        (status = 401, description= "Unauthorized", body = ApiResponseError),       
        (status = 404, description= "Users Not Found", body = ApiResponseError),
        (status = 500, description= "Internal Server Error", body = ApiResponseError),
    ),
    security(
       ("auth_token" = [])
   )
)]
#[get("")]
pub async fn get_users_handler(
    opts: web::Query<UsersFilterOptions>,
    data: web::Data<AppState>,
) -> Result<impl Responder, ApiResponseError> {
    // sleep(Duration::from_secs(5));
    let limit = opts.limit.unwrap_or(10);
    let page = opts.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    let mut query = "SELECT * FROM users".to_string();
    let mut conditions = Vec::new();

    // Adds SQL condition for `mobile` parameter if provided
    if let Some(ref mobile) = opts.mobile {
        conditions.push(format!("mobile LIKE '%{}%'", mobile));
    }

    // Constructs WHERE clause if conditions exist
    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    // Appends LIMIT and OFFSET to SQL query if export is false
    if !opts.export.unwrap_or(false) {
        query.push_str(" LIMIT ? OFFSET ?");
    }

    // Executes SQL query to fetch users based on conditions, limit, and offset
    let users: Vec<UserModel> = sqlx::query_as::<_, UserModel>(&query)
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(&data.pool)
        .await
        .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {}", e), None))?;

    // Returns 404 error if no users found
    if users.is_empty() {
        return Err(ApiResponseError::new(
            404,
            "No Users Found".to_string(),
            None,
        ));
    }

    // Exports users to CSV if `export` query parameter is true
    if opts.export.unwrap_or(false) {
        let csv_data = export_to_csv(&users)
            .map_err(|e| ApiResponseError::new(500, format!("CSV Export Error: {}", e), None))?;
        return Ok(HttpResponse::Ok().content_type("text/csv").body(csv_data));
    }

    // Maps UserModel instances to UserModelResponse and collects into Vec<UserModelResponse>
    let user_response = users
        .into_iter()
        .map(|mut user| UserModelResponse::filter_db(&mut user))
        .collect::<Vec<UserModelResponse>>();

    // Constructs SQL query to fetch total count of users matching conditions
    let total_count_query = if !conditions.is_empty() {
        format!(
            "SELECT COUNT(*) FROM users WHERE {}",
            conditions.join(" AND ")
        )
    } else {
        "SELECT COUNT(*) FROM users".to_string()
    };

    // Fetches total count of users and handles errors
    let total_count: (i64,) = sqlx::query_as(&total_count_query)
        .fetch_one(&data.pool)
        .await
        .map_err(|e| ApiResponseError::new(500, format!("Internal Server Error: {}", e), None))?;

    // Calculates pagination details
    let total_items = total_count.0;
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
    let pagination = Pagination {
        total_items,
        total_pages,
        current_page: page as i64,
        per_page: limit as i64,
    };

    // Constructs JSON response with users and pagination details
    let json_response = ApiResponseObject::new(serde_json::json!({
        "users": user_response,
    }))
    .map_err(|e| ApiResponseError::new(500, e.to_string(), None))?;

    // Constructs ApiResponseCollection with HTTP status, message, JSON response, and pagination
    Ok(HttpResponse::Ok().json(ApiResponseCollection::new(
        200,
        "Get All Users".to_string(),
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
    async fn test_get_users_handler() {
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

        // Insert test data into the database
        let mobile = format!("9665{}", rand::thread_rng().gen_range(10000000..99999999));
        let create_user_data = CreateUserSchema {
            name: Some(Name().fake()),
            mobile: mobile.clone(),
            email: Some(SafeEmail().fake()),
        };

        let _insert_result =
            sqlx::query("INSERT INTO users (id, name, mobile, email) VALUES (?, ?, ?, ?, ?)")
                .bind(&uuid::Uuid::new_v4().to_string())
                .bind(&create_user_data.name.as_deref())
                .bind(&create_user_data.mobile)
                .bind(create_user_data.email.as_deref())
                .execute(&app_state.pool)
                .await
                .expect("Failed to insert test user");

        let req = test::TestRequest::get()
            .uri("/admin/users?limit=10&page=1")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 200);
        assert_eq!(resp.message, "Get All Users");
    }
}
