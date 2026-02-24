use crate::handlers::order::{create_order, get_order, get_user_orders};
use crate::services::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_order_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_order))
        .route("/:user_id", get(get_user_orders))
        .route("/:user_id/:order_id", get(get_order))
}
