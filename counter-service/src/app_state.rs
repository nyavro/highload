use fred::types::config::ReconnectPolicy;
use std::{env, error::Error, time::Duration};
use tracing::info;
use fred::{prelude::*};

#[derive(Clone)]
pub struct AppState {
    pub redis_pool: Pool,
}

async fn init_redis_pool() -> Result<fred::prelude::Pool, fred::prelude::Error> {
    let pool_size = env::var("REDIS_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(8);
    let config = fred::prelude::Config::from_url(&env::var("REDIS_URL").unwrap()).expect("Failed to create redis config from url");
    let pool = fred::prelude::Builder::from_config(config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(10);
        })        
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(pool_size)
        .expect("Failed to create redis pool");            
    pool.init().await.expect("Failed to connect to redis");
    tracing::info!("Connected to Redis");
    Ok(pool)
}

impl AppState {
    pub async fn init() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let redis_pool = init_redis_pool().await?;
        Ok(Self {redis_pool})
    }
}