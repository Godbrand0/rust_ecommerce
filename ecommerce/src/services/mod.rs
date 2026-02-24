pub mod database;
pub mod payment;

pub use database::{check_database_health, create_connection_pool, DbPool};
pub use payment::PaystackService;

use axum::extract::FromRef;

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub paystack_service: PaystackService,
}

impl AppState {
    pub fn new(db_pool: DbPool, paystack_service: PaystackService) -> Self {
        Self { db_pool, paystack_service }
    }
}

impl FromRef<AppState> for DbPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

impl FromRef<AppState> for PaystackService {
    fn from_ref(state: &AppState) -> Self {
        state.paystack_service.clone()
    }
}
