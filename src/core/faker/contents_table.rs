use crate::core::app_state::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use fake::{faker::lorem::en::Paragraph, Fake};
use rand::Rng;
use uuid::Uuid;

#[get("/contents/{count}")]
pub async fn create_fake_contents(
    app_state: web::Data<AppState>,
    count: web::Path<usize>,
) -> impl Responder {
    let count = count.into_inner();
    let mut unique_titles = std::collections::HashSet::new();

    for _ in 0..count {
        let title: String = loop {
            let title: String = Paragraph(1..3).fake();
            if !unique_titles.contains(&title) {
                let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contents WHERE title = ?")
                    .bind(&title)
                    .fetch_one(&app_state.pool)
                    .await
                    .unwrap_or((0,)); // Handle error appropriately

                if row.0 == 0 {
                    unique_titles.insert(title.clone());
                    break title;
                }
            }
        };

        let content_id = Uuid::new_v4().to_string();
        let summary: Option<String> = if rand::thread_rng().gen_range(0..2) == 1 {
            Some(Paragraph(1..5).fake())
        } else {
            None
        };
        let details: Option<String> = if rand::thread_rng().gen_range(0..2) == 1 {
            Some(Paragraph(5..10).fake())
        } else {
            None
        };
        let content_type = "page";

        let insert_result = sqlx::query(
            "INSERT INTO contents (id, title, content_type, summary, details, created_by) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&content_id)
        .bind(&title)
        .bind(&content_type)
        .bind(&summary)
        .bind(&details)
        .bind("a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b") // Replace with actual creator ID logic
        .execute(&app_state.pool)
        .await;

        if let Err(e) = insert_result {
            log::error!("Error seeding content: {:?}", e);
            return HttpResponse::InternalServerError().body("Error seeding content");
        }
    }

    HttpResponse::Ok().body("Contents data seeded")
}

