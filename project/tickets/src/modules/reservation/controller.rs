use std::sync::Arc;

use axum::{
    Json, extract::{Path, State}, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{app_state::AppState, modules::reservation::{cache_repository::RedisReservationRepository, db_repository::PostgresSeatsRepository, single_flight::SingleFlightSeatRepository}};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SeatStatusDto {
    pub id: Uuid,
    pub sector: String,
    pub row_number: i32,
    pub seat_number: i32,
    pub price_cents: i32,
    pub is_available: bool,
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

pub async fn get_event_seats_handler(
    Path(event_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Vec<SeatStatusDto>>), (StatusCode, String)> {    
    let redis_repo = Arc::new(RedisReservationRepository::new(state.redis_pool.clone()));
    // 1. Cache hit
    if let Some(cached_json) = redis_repo.get_cached_seats(event_id).await {
        if let Ok(seats) = serde_json::from_str::<Vec<SeatStatusDto>>(&cached_json) {
            return Ok((StatusCode::OK, Json(seats)));
        }
    }
    // 2. Cache miss + SingleFlight
    let postgres_repo = Arc::new(PostgresSeatsRepository::new(state.postgres_pool.clone()));
    let sf_repo = SingleFlightSeatRepository::new(
        postgres_repo, 
        redis_repo.clone(), 
        state.seats_single_flight.clone()
    );    
    let seats = match sf_repo.find_all_with_statuses(event_id).await {
        Ok(s) => s,
        Err(e) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR, 
            format!("Failed to load seats: {}", e)
        )),
    };
    let json_string = match serde_json::to_string(&seats) {
        Ok(js) => js,
        Err(e) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR, 
            format!("Serialization error: {}", e)
        )),
    };
    let _ = redis_repo.set_cached_seats(event_id, &json_string).await;
    Ok((StatusCode::OK, Json(seats)))
}
