use std::sync::Arc;
use axum::{Router};
use axum::routing::{get};
use crate::app_state::AppState;
use crate::modules::health::checks;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(checks::health_check))
        .route("/redis/health", get(checks::redis_health_check))        
        .with_state(state)
}