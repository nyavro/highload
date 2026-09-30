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

        // Строго указываем индексы KEYS[1], ARGV[1], ARGV[2] для корректной работы парсера Redis
        let lua_script = r#"
            if redis.call("EXISTS", KEYS[1]) == 0 then
                redis.call("SET", KEYS[1], ARGV[1])
                redis.call("EXPIRE", KEYS[1], ARGV[2])
                return 1
            else
                return 0
            end
        "#;

        // 1. Ключ передаем как тип Key
        let keys: Key = lock_key.into();

        // 2. ИСПРАВЛЕНО: Явно упаковываем аргументы в Value::String. 
        // Это гарантирует, что Redis получит чистые строки и Lua-скрипт не упадет с ошибкой типов.
        let args: Vec<Value> = vec![
            Value::String(reservation_id.to_string().into()),
            Value::String(ttl_seconds.to_string().into()),
        ];

        // 3. Вызываем .eval()
        let result: i32 = self.pool
            .eval(lua_script, keys, args)
            .await?;

        Ok(result == 1)
    }
}
