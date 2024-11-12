use actix_web::web;
use sqlx::MySqlPool;

use crate::{config::CONFIG, core::app_state::AppState};

#[allow(dead_code)]
pub const USER_ID: &str = "a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b";

#[allow(dead_code)]
pub fn generate_jwt(user_id: &str) -> String {
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

#[allow(dead_code)]
pub fn generate_test_jwt() -> String {
    let expire = chrono::Duration::minutes(60);
    let now = chrono::Utc::now();
    let claims = crate::core::utils::jwt::Claims {
        exp: (now + expire).timestamp() as usize,
        iat: now.timestamp() as usize,
        id: USER_ID.to_owned(),
    };
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(CONFIG.jwt.secret.as_ref());
    jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &encoding_key).unwrap()
}

#[allow(dead_code)]
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
