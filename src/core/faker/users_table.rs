use crate::core::app_state::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use fake::{faker::internet::en::SafeEmail, faker::name::en::Name, Fake};
use rand::Rng;
use uuid::Uuid;

#[get("/users/{count}")]
pub async fn create_fake_users(
    app_state: web::Data<AppState>,
    count: web::Path<usize>,
) -> impl Responder {
    let count = count.into_inner();
    let mut unique_names = std::collections::HashSet::new();
    let mut unique_mobiles = std::collections::HashSet::new();
    let mut unique_emails = std::collections::HashSet::new();

    for _ in 0..count {
        let name: String = loop {
            let name: String = Name().fake();
            if !unique_names.contains(&name) {
                let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE name = ?")
                    .bind(&name)
                    .fetch_one(&app_state.pool)
                    .await
                    .expect("Failed to query the database");

                if row.0 == 0 {
                    unique_names.insert(name.clone());
                    break name;
                }
            }
        };

        let mobile: String = loop {
            let mobile = format!("9665{}", rand::thread_rng().gen_range(10000000..99999999));
            if !unique_mobiles.contains(&mobile) {
                let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE mobile = ?")
                    .bind(&mobile)
                    .fetch_one(&app_state.pool)
                    .await
                    .expect("Failed to query the database");

                if row.0 == 0 {
                    unique_mobiles.insert(mobile.clone());
                    break mobile;
                }
            }
        };

        let email: String = loop {
            let email: String = SafeEmail().fake();
            if !unique_emails.contains(&email) {
                let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ?")
                    .bind(&email)
                    .fetch_one(&app_state.pool)
                    .await
                    .expect("Failed to query the database");

                if row.0 == 0 {
                    unique_emails.insert(email.clone());
                    break email;
                }
            }
        };

        let user_id = Uuid::new_v4().to_string();
        let insert_result = sqlx::query(
            "INSERT INTO users (id, name, mobile, email, configurations) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(&name)
        .bind(&mobile)
        .bind(&email)
        .bind("{}")
        .execute(&app_state.pool)
        .await;

        match insert_result {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error seeding user: {:?}", e);
                return HttpResponse::InternalServerError()
                    .body(format!("Error seeding user: {:?}", e));
            }
        }
    }
    HttpResponse::Ok().body("Users data seeded")
}

