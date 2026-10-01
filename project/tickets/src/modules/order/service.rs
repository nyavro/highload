use uuid::Uuid;
use thiserror::Error;
use chrono::Utc;
use crate::modules::reservation::repository::RedisReservationRepository;
use crate::modules::order::repository::KafkaOrderRepository;
use crate::modules::order::dto::{CheckoutInput, OrderPaidEvent};

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Reservation is invalid or has expired")]
    InvalidReservation,
    #[error("Internal system error: {0}")]
    Internal(String),
}

pub struct OrderService {
    redis_repo: RedisReservationRepository,
    kafka_repo: KafkaOrderRepository,
}

impl OrderService {
    pub fn new(redis_repo: RedisReservationRepository, kafka_repo: KafkaOrderRepository) -> Self {
        Self { redis_repo, kafka_repo }
    }

    pub async fn checkout(&self, input: CheckoutInput) -> Result<Uuid, OrderError> {
        // 1. Атомарно проверяем и удаляем временную бронь в Redis
        let is_valid = self
            .redis_repo
            .confirm_and_free_reservation(input.event_id, &input.seat_id, &input.reservation_id)
            .await
            .map_err(|e| OrderError::Internal(e.to_string()))?;

        if !is_valid {
            return Err(OrderError::InvalidReservation);
        }
        let order_id = Uuid::new_v4();        
        let ticket_code = format!("TKT-{}-{}", input.seat_id, Uuid::new_v4().to_string().split('-').next().unwrap_or(""));

        let event = OrderPaidEvent {
            order_id,
            user_id: input.user_id,
            event_id: input.event_id,
            seat_id: input.seat_id,
            reservation_id: input.reservation_id,
            amount_cents: input.amount_cents,
            ticket_code,
            timestamp_ms: Utc::now().timestamp_millis(),
        };
        
        self.kafka_repo
            .send_order_paid_event(&event)
            .await
            .map_err(|e| OrderError::Internal(e.to_string()))?;

        Ok(order_id)
    }
}
