use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime, Object};
use tokio_postgres::{NoTls};
use std::{env, time::Duration};
use log::info;
use fred::{prelude::{Error, ReconnectPolicy}, prelude::*};

#[derive(Clone)]
pub struct AppState {
    master_pool: Pool,
    replica_pools: Vec<Pool>,
    pub secret: String,
    pub jwt_token_ttl_minutes: i64,
    pub redis: fred::prelude::Pool,
}

fn init_config(port_key: &str) -> Config {
    let mut config = Config::new();
    config.user = env::var("db_postgres_user").ok();        
    config.password = env::var("db_postgres_password").ok();
    config.dbname = env::var("db_postgres_dbname").ok();
    config.host = env::var("db_postgres_host").ok();
    config.port = env::var(port_key).ok().map(|port| port.parse().unwrap());
    config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });        
    config.connect_timeout = Some(Duration::from_secs(10));        
    config
}

async fn init_redis_pool() -> Result<fred::prelude::Pool, Error> {
    let pool_size = env::var("REDIS_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(8);
    let config = fred::prelude::Config::from_url("redis://127.0.0.1:6379/").expect("Failed to create redis config from url");
    let pool = fred::prelude::Builder::from_config(config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(10);
        })        
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(pool_size)
        .expect("Failed to create redis pool");            
    pool.init().await.expect("Failed to connect to redis");
    info!("Connected to Redis");
    Ok(pool)
}

impl AppState {    

    pub async fn init() -> Result<Self, String> {        
        let master_pool = init_config(
                "db_postgres_master_port"
            )
            .create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        master_pool.resize(10);        
        let replica_pool1 = init_config(
            "db_postgres_replica1_port"
            )
            .create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        replica_pool1.resize(10);            
        let replica_pool2 = init_config(
                "db_postgres_replica2_port"
            )
            .create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        replica_pool2.resize(10);        
        Ok(
            AppState {
                master_pool,
                replica_pools: vec!(replica_pool1, replica_pool2),
                secret: env::var("JWT_SECRET").unwrap(),
                jwt_token_ttl_minutes: env::var("jwt_token_ttl_minutes").unwrap().parse().unwrap(),                
                redis: init_redis_pool().await.unwrap()
            }
        )
    }

    pub async fn get_master_client(&self) -> Object {
        self.master_pool.get().await.unwrap()
    }

    pub async fn get_replica_client(&self) -> Object {        
        use rand::Rng;
        let idx = rand::rng().random_range(0..self.replica_pools.len());
        info!("{:?}", idx);
        self.replica_pools[idx].get().await.unwrap()
        // self.get_master_client().await
    }        
}