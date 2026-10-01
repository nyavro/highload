use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderPaidEvent {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub seat_id: String,
    pub reservation_id: String,
    pub amount_cents: i32,
    pub ticket_code: String,
    pub timestamp_ms: i64,
}

#[derive(Deserialize)]
pub struct CheckoutInput {
    pub event_id: Uuid,
    pub seat_id: String,
    pub reservation_id: String,
    pub user_id: Uuid,
    pub amount_cents: i32,
}
