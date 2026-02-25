use crate::handlers::cart::{add_to_cart, get_cart, remove_from_cart, update_cart_item};
use crate::services::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn create_cart_routes() -> Router<AppState> {
    Router::new()
        .route("/:user_id", get(get_cart))
        .route("/:user_id", post(add_to_cart))
        .route("/:user_id/:item_id", put(update_cart_item))
        .route("/:user_id/:item_id", delete(remove_from_cart))
}
