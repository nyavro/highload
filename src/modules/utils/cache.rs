use log::{error, warn};
use deadpool_redis::{redis::{cmd}};
use serde::{Serialize, Deserialize};

pub async fn get_or_set_cache<T, E, F, Fut>(redis_pool: &deadpool_redis::Pool, cache_key: &str, fetch_func: F) -> Result<T, E> 
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>, {             
    if let Ok(mut conn) = redis_pool.get().await {
        let cached_value: Option<String> = cmd("GET")
            .arg(&cache_key)
            .query_async(&mut conn)
            .await
            .ok();
        if let Some(json) = cached_value {
            match serde_json::from_str(&json) {
                Ok(res) => return Ok(res),
                Err(e) => {
                    error!("Failed to deserialize cache for {}: {}", cache_key, e);
                }
            }
        }
        let data = fetch_func().await?;
        if let Ok(json) = serde_json::to_string(&data) {
            if let Err(e) = cmd("SET")
                .arg(cache_key)
                .arg(json)
                .arg("EX")
                .arg(3600)
                .query_async::<()>(&mut conn)
                .await {
                warn!("Redis caching error {}", e);
            }                    
        }
        Ok(data)                            
    } else {
        warn!("Redis pool error, fetching directly from DB");
        fetch_func().await    
    }        
}