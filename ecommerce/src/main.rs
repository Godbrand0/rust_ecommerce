use axum::{routing::get, Router};
use std::env;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;
mod routes;
mod services;

use routes::create_routes;
use services::{create_connection_pool, AppState, PaystackService};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "rust_ecommerce=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let server_address = format!("{}:{}", host, port);

    let db_pool = match create_connection_pool().await {
        Ok(pool) => {
            info!("Database connection pool created successfully");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to create database connection pool: {}", e);
            tracing::error!("Please ensure PostgreSQL is running and accessible");
            tracing::error!("See DATABASE_SETUP.md for setup instructions");
            std::process::exit(1);
        }
    };

    let paystack_service = PaystackService::new();

    let app = Router::new()
        .route(
            "/",
            get(|| async { axum::Json(serde_json::json!({"message": "Rust E-commerce API"})) }),
        )
        .nest("/api", create_routes())
        .with_state(AppState::new(db_pool, paystack_service))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .into_inner(),
        );

    info!("Starting server at http://{}", server_address);

    let listener = tokio::net::TcpListener::bind(&server_address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
