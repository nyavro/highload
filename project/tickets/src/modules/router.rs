use std::sync::Arc;
use axum::{Router};
use axum::routing::{get, post};
use crate::app_state::AppState;
use crate::modules::health::checks;
use crate::modules::order::controller::checkout_handler;
use crate::modules::reservation::controller::reserve_seat_handler;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(checks::health_check))
        .route("/redis/health", get(checks::redis_health_check))        
        .route("/events/{id}/reserve", post(reserve_seat_handler))
        .route("/orders/checkout", post(checkout_handler))
        .with_state(state)
}