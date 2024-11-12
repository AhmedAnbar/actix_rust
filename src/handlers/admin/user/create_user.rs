use crate::{
    core::app_state::AppState,
    model::user::UserModel,
    schema::{
        admin::user::CreateUserSchema,
        response::{
            admin::users::UserModelResponse, api_response::ApiResponse,
            api_response_error::ApiResponseError, api_response_object::ApiResponseObject,
        },
    },
};
use actix_web::{post, web};
use serde_json::json;

// Endpoint handler for creating users
#[utoipa::path(
    post,
    path = "/admin/users/create",
    tag = "Admin: Users Endpoint",
    // Specify request body details
    request_body(content = CreateUserSchema, description = "Credentials to create user", example = json!({"name": "test name", "mobile": "051234567","email": "test@test.test", "gender": "Male", "configurations": json!({"property1": "value", "property2": json!({"sub-property": "value"})})}) ),
    // Specify possible responses
    responses(
        (status = 201, description= "User created", body = ApiResponse),       
        (status = 409, description= "Duplicate entry", body = ApiResponseError),       
        (status = 500, description= "Internal Server Error", body = ApiResponseError),       
    ),
    // Specify security requirements
    security(
       ("auth_token" = [])
   )
)]
#[post("/create")]
pub async fn create_user_handler(
    data: web::Json<CreateUserSchema>, // JSON payload containing user creation data
    app_state: web::Data<AppState>,    // Application state containing database pool
) -> Result<ApiResponse, ApiResponseError> {
    let user_id = uuid::Uuid::new_v4().to_string(); // Generate a new UUID for the user

    // Convert configurations to JSON string if present

    // Execute SQL query to insert user data into the database
    let insert_result =
        sqlx::query("INSERT INTO users (id, name, mobile, email) VALUES (?, ?, ?, ?, ?)")
            .bind(&user_id)
            .bind(&data.name.as_deref())
            .bind(&data.mobile)
            .bind(data.email.as_deref())
            .execute(&app_state.pool)
            .await;

    // Handle the result of the SQL query
    match insert_result {
        Ok(_) => {
            // Retrieve the newly created user from the database
            let mut user = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = ?", user_id)
                .fetch_one(&app_state.pool)
                .await
                .map_err(|err| {
                    ApiResponseError::new(500, format!("Internal Server Error: {:?}", err), None)
                })?;

            // Filter sensitive information from the user response
            let response = UserModelResponse::filter_db(&mut user);

            // Construct the API response object with the user data
            let user_response = ApiResponseObject::new(serde_json::json!({"user": response}))
                .map_err(|err| ApiResponseError::new(500, err.to_string(), None))?;

            // Return a successful API response indicating user creation
            Ok(ApiResponse::new(
                201,
                "User Created".to_string(),
                Some(user_response),
            ))
        }
        Err(err) => {
            // Handle database errors
            if err.to_string().contains("Duplicate entry") {
                // Return a conflict error if the mobile number already exists
                Err(ApiResponseError::new(
                    409,
                    "Mobile already exists".to_string(),
                    None,
                ))
            } else {
                // Return a generic internal server error for other database errors
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
    async fn test_create_user_handler() {
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

        let mobile = format!("9665{}", rand::thread_rng().gen_range(10000000..99999999));
        let create_user_data = CreateUserSchema {
            name: Some(Name().fake()),
            mobile,
            email: Some(SafeEmail().fake()),
        };

        let req = test::TestRequest::post()
            .uri("/admin/users/create")
            .insert_header((
                actix_web::http::header::AUTHORIZATION,
                format!("Bearer {}", jwt),
            ))
            .set_json(&create_user_data)
            .to_request();

        let resp: ApiResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.status, 201);
        assert_eq!(resp.message, "User Created");
    }
}
