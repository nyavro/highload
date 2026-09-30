use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{app_state::AppState, modules::reservation::repository::RedisReservationRepository};
use crate::modules::reservation::service::{ReservationService, ReservationError};

#[derive(Deserialize)]
pub struct ReserveInput {
    pub seat_id: String,
    pub user_id: String,
}

#[derive(Serialize)]
pub struct ReserveOutput {
    pub reservation_id: String,
    pub status: String,
    pub expires_in_seconds: u64,
}

pub async fn reserve_seat_handler(
    Path(event_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReserveInput>,
) -> Result<(StatusCode, Json<ReserveOutput>), (StatusCode, String)> {
    let redis_repo = RedisReservationRepository::new(state.redis_pool.clone());
    let reservation_service = ReservationService::new(redis_repo);
    
    match reservation_service.reserve(event_id, &payload.seat_id, &payload.user_id).await {
        Ok(details) => Ok((
            StatusCode::OK,
            Json(ReserveOutput {
                reservation_id: details.reservation_id,
                status: "reserved".to_string(),
                expires_in_seconds: details.expires_in_seconds,
            }),
        )),
        Err(ReservationError::AlreadyReserved) => Err((
            StatusCode::CONFLICT,
            "This seat is already reserved".to_string(),
        )),
        Err(ReservationError::Internal(err_msg)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("System error: {}", err_msg),
        )),
    }
}
