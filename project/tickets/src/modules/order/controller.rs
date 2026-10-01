use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;
use std::sync::Arc;
use crate::app_state::AppState;
use crate::modules::reservation::repository::RedisReservationRepository;
use crate::modules::order::repository::KafkaOrderRepository;
use crate::modules::order::service::{OrderService, OrderError};
use crate::modules::order::dto::CheckoutInput;

#[derive(Serialize)]
pub struct CheckoutOutput {
    pub order_id: Uuid,
    pub status: String,
    pub message: String,
}

pub async fn checkout_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CheckoutInput>,
) -> Result<(StatusCode, Json<CheckoutOutput>), (StatusCode, String)> {    
    let redis_repo = RedisReservationRepository::new(state.redis_pool.clone());
    let kafka_repo = KafkaOrderRepository::new(state.kafka_producer.clone(), state.kafka_topic.clone());
    let order_service = OrderService::new(redis_repo, kafka_repo);

    match order_service.checkout(payload).await {
        Ok(order_id) => Ok((
            StatusCode::ACCEPTED,
            Json(CheckoutOutput {
                order_id,
                status: "processing".to_string(),
                message: "Order received and is being processed asynchronously".to_string(),
            }),
        )),
        Err(OrderError::InvalidReservation) => Err((
            StatusCode::BAD_REQUEST,
            "Your reservation has expired or is invalid. Please try reserving the seat again.".to_string(),
        )),
        Err(OrderError::Internal(err_msg)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Payment pipeline error: {}", err_msg),
        )),
    }
}
