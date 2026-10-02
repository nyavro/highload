use std::sync::Arc;
use tokio::sync::watch;
use uuid::Uuid;

use crate::app_state::{FlightState, SingleFlightGroup};
use crate::modules::reservation::controller::SeatStatusDto;
use crate::modules::reservation::db_repository::PostgresSeatsRepository;
use crate::modules::reservation::cache_repository::RedisReservationRepository;

pub struct SingleFlightSeatRepository {
    postgres_repo: Arc<PostgresSeatsRepository>,
    redis_repo: Arc<RedisReservationRepository>,
    flight_group: SingleFlightGroup<Vec<SeatStatusDto>>,
}

impl SingleFlightSeatRepository {
    pub fn new(
        postgres_repo: Arc<PostgresSeatsRepository>,
        redis_repo: Arc<RedisReservationRepository>,
        flight_group: SingleFlightGroup<Vec<SeatStatusDto>>,
    ) -> Self {
        Self {
            postgres_repo,
            redis_repo,
            flight_group,
        }
    }

    pub async fn find_all_with_statuses(&self, event_id: Uuid) -> Result<Vec<SeatStatusDto>, String> {
        let rx_or_tx = self.try_register_flight(event_id);
        match rx_or_tx {
            // Slave Mode: Request in progress, wait
            Ok(receiver) => {
                let mut rx = receiver;
                loop {
                    if let Some(ref seats) = *rx.borrow() {
                        return Ok(seats.clone());
                    }
                    if rx.changed().await.is_err() {
                        return Err("SingleFlight leader failed or dropped".to_string());
                    }
                }
            }
            // Leader mode: Do async task
            Err(tx) => {
                let result = self.fetch_and_enrich(event_id).await;                
                self.unregister_flight(event_id);
                match result {
                    Ok(seats) => {
                        let _ = tx.send(Some(seats.clone()));
                        Ok(seats)
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
    
    fn try_register_flight(&self, event_id: Uuid) -> Result<watch::Receiver<Option<Vec<SeatStatusDto>>>, watch::Sender<Option<Vec<SeatStatusDto>>>> {
        let mut lock = self.flight_group.lock().unwrap();
        if let Some(FlightState::Pending(receiver)) = lock.get(&event_id) {
            Ok(receiver.clone())
        } else {
            let (tx, rx) = watch::channel(None);
            lock.insert(event_id, FlightState::Pending(rx));
            Err(tx)
        }
    }

    fn unregister_flight(&self, event_id: Uuid) {
        let mut lock = self.flight_group.lock().unwrap();
        lock.remove(&event_id);
    }

    async fn fetch_and_enrich(&self, event_id: Uuid) -> Result<Vec<SeatStatusDto>, String> {
        let db_seats = self.postgres_repo.find_all_by_event_id(event_id).await
            .map_err(|e| format!("DB Error: {}", e))?;

        let mut seats = Vec::with_capacity(db_seats.len());    
        for db_seat in db_seats {        
            let is_locked: bool = self.redis_repo.is_locked(event_id, db_seat.id.to_string()).await;
            seats.push(SeatStatusDto {
                id: db_seat.id,
                sector: db_seat.sector,
                row_number: db_seat.row_number,
                seat_number: db_seat.seat_number,
                price_cents: db_seat.price_cents,
                is_available: !is_locked,
            });
        }
        Ok(seats)
    }
}
