use core::{
    api_doc::ApiDoc, app_state::AppState, mail::email_queue::EmailQueue, sms::sms_queue::SmsQueue,
};
use std::{error::Error, fmt::Display, sync::Arc};

use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use config::CONFIG;
use dotenv::dotenv;
use env_logger::Env;
use log::{error, info};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sqlx::mysql::MySqlPoolOptions;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod core;
mod handlers;
mod middlewares;
mod model;
mod routes;
mod schema;

#[actix_web::main]
async fn main() -> Result<(), MainError> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info,actix_web=info")).init();

    let openapi = ApiDoc::openapi();

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&CONFIG.database.url)
        .await
        .map_err(|e| {
            error!("🔥 Failed to connect to the database: {:?}", e);
            std::process::exit(1);
        })
        .expect("Database connection pool creation failed");

    info!("✅ Connection to the database is successful!");

    // Create application state shared across all Actix Web handlers
    let app_state = Arc::new(AppState { pool });
    info!("🚀 Server started successfully");

    // Initialize MAIL processing queue and spawn processing task
    let (email_queue, email_receiver) = EmailQueue::new();
    tokio::spawn(async move {
        info!("Starting email processing task");
        EmailQueue::process_queue(email_receiver).await;
    });

    // Initialize SMS processing queue and spawn processing task
    let (sms_queue, sms_receiver) = SmsQueue::new();
    tokio::spawn(async move {
        info!("Starting SMS processing task");
        SmsQueue::process_queue(sms_receiver).await;
    });

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("certs/key.pem", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("certs/cert.pem")
        .unwrap();

    let host = format!("{}:{}", CONFIG.host, CONFIG.port);

    // Configure Actix Web server with application routes and middleware
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&CONFIG.cors)
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();

        App::new()
            .app_data(web::Data::from(app_state.clone()))
            .app_data(web::Data::new(email_queue.clone()))
            .app_data(web::Data::new(sms_queue.clone()))
            .service(web::scope("/seed").configure(core::faker::config))
            .service(
                web::scope("/api")
                    .configure(routes::health_checker::config)
                    .configure(routes::project::profile::config)
                    .configure(routes::auth::config),
            )
            .service(
                web::scope("/admin")
                    .configure(routes::admin::user::config)
                    .configure(routes::admin::content::config),
            )
            .service(Redoc::with_url("/redoc", openapi.clone()))
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind_openssl(host, builder)
    .map_err(|e| MainError::new(e.to_string()))?
    .run()
    .await
    .map_err(|e| MainError::new(e.to_string()))
}

// Custom error type for main function
#[derive(Debug)]
struct MainError {
    message: String,
}

impl MainError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for MainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl Error for MainError {}
