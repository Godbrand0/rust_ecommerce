use crate::handlers::user::{create_user, delete_user, get_user, get_users};
use crate::services::AppState;
use axum::{
    routing::{delete, get},
    Router,
};

pub fn create_user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users).post(create_user))
        .route("/:id", get(get_user))
        .route("/:id", delete(delete_user))
}
