use fred::prelude::*;
use std::env;
use std::time::Duration;
use uuid::Uuid;

// Test data generator

//copypasted from app_state.rs
async fn init_redis_pool() -> Result<fred::prelude::Pool, fred::prelude::Error> {
    let pool_size = env::var("REDIS_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(8);
    
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in env");
    let config = fred::prelude::Config::from_url(&redis_url)
        .expect("Failed to create redis config from url");
        
    let pool = fred::prelude::Builder::from_config(config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(10);
        })        
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(pool_size)
        .expect("Failed to create redis pool");            
    
    pool.init().await.expect("Failed to connect to redis");
    // Используем стандартный println, если глобальный логгер tracing еще не проинициализирован в main
    println!("Connected to Redis");
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    // Загружаем .env файл, чтобы env::var("REDIS_URL") сработал при локальном запуске
    dotenv::dotenv().ok();

    let pool = init_redis_pool().await?;
    pool.connect();
    pool.wait_for_connect().await?;

    // Фиксированный UUID для тестов
    let event_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    println!("Инициализация мест для event_id: {}", event_id);

    // ИСПРАВЛЕНО: Собираем Vec<Key> вместо Vec<Value>. 
    // Тип Key представляет именно ключи базы данных и отлично подходит для метода .del()
    let mut keys_to_del: Vec<Key> = vec![];
    for seat_id in 1..=100 {
        let key_str = format!("event:{}:seat:seat_{}:lock", event_id, seat_id);
        keys_to_del.push(key_str.into());
    }
    
    // Теперь Vec<Key> успешно преобразуется в MultipleKeys под капотом драйвера fred
    let _: i32 = pool.del(keys_to_del).await.unwrap_or(0);
    
    println!("Redis успешно очищен и готов к нагрузочным тестам!");
    Ok(())
}
