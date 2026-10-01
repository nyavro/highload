use std::sync::Arc;
use axum::{Router, middleware::from_fn_with_state};
use axum::routing::{get, post};
use crate::app_state::AppState;
use crate::modules::health::checks;
use crate::modules::queue::controller::get_queue_status_handler;
use crate::modules::reservation::controller::reserve_seat_handler;
use crate::modules::order::controller::checkout_handler;
use crate::modules::queue::middleware::queue_protection_middleware;

pub fn router(state: Arc<AppState>) -> Router {
    let protected_reservation_route = Router::new()
        .route("/events/{id}/reserve", post(reserve_seat_handler))
        .route_layer(from_fn_with_state(state.clone(), queue_protection_middleware));

    Router::new()
        .route("/health", get(checks::health_check))
        .route("/redis/health", get(checks::redis_health_check))        
        .route("/orders/checkout", post(checkout_handler))        
        .route("/events/{id}/queue/status", get(get_queue_status_handler))
        .merge(protected_reservation_route)        
        .with_state(state)
}
