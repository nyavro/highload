use fred::types::config::ReconnectPolicy;
use std::{env, error::Error, sync::Arc, time::Duration};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime, Object};
use fred::{prelude::*};
use tokio_postgres::{NoTls};

#[derive(Clone)]
pub struct AppState {
    pub redis_pool: Arc<fred::prelude::Pool>,
    pub postgres_pool: Arc<deadpool_postgres::Pool>,
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


fn init_config(port_key: &str) -> Config {
    let mut config = Config::new();
    config.user = env::var("POSTGRES_USER").ok();     
    config.password = env::var("POSGTRES_PASSWORD").ok();    
    config.dbname = env::var("POSTGRES_DB_NAME").ok();        
    config.host = env::var("POSTGRES_HOST").ok();
    config.port = env::var(port_key).ok().map(|port| port.parse().unwrap());
    config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });        
    config.connect_timeout = Some(Duration::from_secs(10));        
    config
}

impl AppState {
    pub async fn init() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let redis_pool = init_redis_pool().await?;
        let postgres_pool = init_config(
                "POSTGRES_PORT"
            )
            .create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        postgres_pool.resize(10);        
        Ok(Self {redis_pool: Arc::new(redis_pool), postgres_pool: Arc::new(postgres_pool)})
    }
}