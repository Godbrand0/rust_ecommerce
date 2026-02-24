use crate::handlers::product::{create_product, get_product, get_products};
use crate::services::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_product_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_products))
        .route("/", post(create_product))
        .route("/:id", get(get_product))
}
