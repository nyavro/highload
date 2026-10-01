use fred::clients::Pool;
use fred::prelude::*;
use uuid::Uuid;
use std::{error::Error, sync::Arc};

pub struct RedisReservationRepository {
    pool: Arc<Pool>,
}

impl RedisReservationRepository {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }

    pub async fn try_reserve_seat(
        &self,
        event_id: Uuid,
        seat_id: &str,
        reservation_id: &str,
        ttl_seconds: usize,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        
        let lock_key = format!("event:{}:seat:{}:lock", event_id, seat_id);
        
        let lua_script = r#"
            if redis.call("EXISTS", KEYS[1]) == 0 then
                redis.call("SET", KEYS[1], ARGV[1])
                redis.call("EXPIRE", KEYS[1], ARGV[2])
                return 1
            else
                return 0
            end
        "#;
        
        let keys: Key = lock_key.into();
        let args: Vec<Value> = vec![
            Value::String(reservation_id.to_string().into()),
            Value::String(ttl_seconds.to_string().into()),
        ];

        let result: i32 = self.pool
            .eval(lua_script, keys, args)
            .await?;

        Ok(result == 1)
    }

    // Checks if reservation exists by UUID and deletes if valid.
    // Ok(true) if reservation confirmed and deleted, Ok(false) if reservation is not valid.
        pub async fn confirm_and_free_reservation(
        &self,
        event_id: Uuid,
        seat_id: &str,
        reservation_id: &str,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let lock_key = format!("event:{}:seat:{}:lock", event_id, seat_id);
        let lua_script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                redis.call("DEL", KEYS[1])
                return 1
            else
                return 0
            end
        "#;
        let keys: Key = lock_key.into();
        let args: Vec<Value> = vec![
            Value::String(reservation_id.to_string().into())
        ];
        let result: i32 = self.pool
            .eval(lua_script, keys, args)
            .await?;

        Ok(result == 1)
    }

}
