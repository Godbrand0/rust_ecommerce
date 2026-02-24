use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::models::{User, UserResponse, CreateUser};
use crate::services::DbPool;

pub async fn create_user(
    State(pool): State<DbPool>,
    Json(user_data): Json<CreateUser>,
) -> Result<Json<UserResponse>, (StatusCode, Json<Value>)> {
    let new_user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let result = sqlx::query(
        r#"
        INSERT INTO users (id, email, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(new_user_id)
    .bind(&user_data.email)
    .bind(&user_data.name)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(new_user_id)
                .fetch_one(&pool)
                .await;

            match user {
                Ok(user) => {
                    let user_response = UserResponse::from(user);
                    Ok(Json(user_response))
                }
                Err(e) => {
                    tracing::error!("Error fetching created user: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to create user"})),
                    ))
                }
            }
        }
        Err(e) => {
            tracing::error!("Error creating user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create user"})),
            ))
        }
    }
}

pub async fn get_user(
    Path(user_id): Path<Uuid>,
    State(pool): State<DbPool>,
) -> Result<Json<UserResponse>, (StatusCode, Json<Value>)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await;

    match user {
        Ok(Some(user)) => {
            let user_response = UserResponse::from(user);
            Ok(Json(user_response))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"})),
        )),
        Err(e) => {
            tracing::error!("Error fetching user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch user"})),
            ))
        }
    }
}

pub async fn delete_user(
    Path(user_id): Path<Uuid>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(res) => {
            if res.rows_affected() > 0 {
                Ok(Json(json!({"message": "User deleted successfully"})))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "User not found"})),
                ))
            }
        }
        Err(e) => {
            tracing::error!("Error deleting user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete user"})),
            ))
        }
    }
}
