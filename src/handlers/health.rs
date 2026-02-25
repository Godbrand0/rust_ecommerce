use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use crate::services::{DbPool, check_database_health};

pub async fn health_check(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match check_database_health(&pool).await {
        Ok(_) => Ok(Json(json!({
            "status": "healthy",
            "database": "connected"
        }))),
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "database": "disconnected",
                    "error": "Database connection failed"
                })),
            ))
        }
    }
}