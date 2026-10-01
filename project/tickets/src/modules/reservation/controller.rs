use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{app_state::AppState, modules::reservation::{cache_repository::RedisReservationRepository, db_repository::PostgresSeatsRepository}};
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
    let redis_repo = RedisReservationRepository::new(state.redis_pool.clone());        
    if let Some(cached_json) = redis_repo.get_cached_seats(event_id).await {
        //Cache hit
        if let Ok(seats) = serde_json::from_str::<Vec<SeatStatusDto>>(&cached_json) {            
            return Ok((StatusCode::OK, Json(seats)));
        }
    }
    // Cache miss
    // TODO: SingleFlight     
    let postgres_repo = PostgresSeatsRepository::new(state.postgres_pool.clone());
    let db_seats = postgres_repo.find_all_by_event_id(event_id).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch seats from DB: {}", e))
    })?;

    let mut seats = Vec::with_capacity(db_seats.len());    
    for db_seat in db_seats {        
        let is_locked: bool = redis_repo.is_locked(event_id, db_seat.id.to_string()).await;
        seats.push(SeatStatusDto {
            id: db_seat.id,
            sector: db_seat.sector,
            row_number: db_seat.row_number,
            seat_number: db_seat.seat_number,
            price_cents: db_seat.price_cents,
            is_available: !is_locked,
        });
    }

    if let Ok(json_string) = serde_json::to_string(&seats) {
        let _ = redis_repo.set_cached_seats(event_id, &json_string).await;
    }
    Ok((StatusCode::OK, Json(seats)))
}
