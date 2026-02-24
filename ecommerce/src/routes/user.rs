use crate::handlers::user::{create_user, delete_user, get_user};
use crate::services::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn create_user_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_user))
        .route("/:id", get(get_user))
        .route("/:id", delete(delete_user))
}
