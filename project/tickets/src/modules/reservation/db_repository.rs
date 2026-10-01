use std::sync::Arc;
use deadpool_postgres::Pool;
use uuid::Uuid;
use std::error::Error;

pub struct DBSeat {
    pub id: Uuid,
    pub sector: String,
    pub row_number: i32,
    pub seat_number: i32,
    pub price_cents: i32,
}

pub struct PostgresSeatsRepository {
    pool: Arc<Pool>,
}

impl PostgresSeatsRepository {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }
    
    pub async fn find_all_by_event_id(&self, event_id: Uuid) -> Result<Vec<DBSeat>, Box<dyn Error + Send + Sync>> {        
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, sector, row_number, seat_number, price_cents FROM seats WHERE event_id = $1;",
                &[&event_id],
            )
            .await?;

        let mut seats = Vec::with_capacity(rows.len());
        for row in rows {
            seats.push(DBSeat {
                id: row.get(0),
                sector: row.get(1),
                row_number: row.get(2),
                seat_number: row.get(3),
                price_cents: row.get(4),
            });
        }
        Ok(seats)
    }
}
