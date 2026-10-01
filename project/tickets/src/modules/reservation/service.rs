use uuid::Uuid;
use thiserror::Error;

use crate::modules::reservation::cache_repository::RedisReservationRepository;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("Seat is already reserved or sold")]
    AlreadyReserved,
    #[error("Internal system error: {0}")]
    Internal(String),
}

pub struct ReservationService {
    repo: RedisReservationRepository,
}

pub struct ReservationDetails {
    pub reservation_id: String,
    pub expires_in_seconds: u64,
}

impl ReservationService {
    pub fn new(repo: RedisReservationRepository) -> Self {
        Self { repo }
    }

    pub async fn reserve(
        &self,
        event_id: Uuid,
        seat_id: &str,
        _user_id: &str, 
    ) -> Result<ReservationDetails, ReservationError> {
        let reservation_id = Uuid::new_v4().to_string();
        let ttl_seconds = 600; // 10 минут по ТЗ

        let success = self
            .repo
            .try_reserve_seat(event_id, seat_id, &reservation_id, ttl_seconds)
            .await
            .map_err(|e| ReservationError::Internal(e.to_string()))?;

        if success {
            Ok(ReservationDetails {
                reservation_id,
                expires_in_seconds: ttl_seconds as u64,
            })
        } else {
            Err(ReservationError::AlreadyReserved)
        }
    }
}
