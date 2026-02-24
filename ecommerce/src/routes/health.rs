use crate::handlers::health::health_check;
use crate::services::DbPool;
use axum::{routing::get, Router};

pub fn create_health_routes() -> Router<DbPool> {
    Router::new().route("/health", get(health_check))
}
