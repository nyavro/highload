use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime, Object};
use tokio_postgres::{NoTls};
use std::{env, time::Duration};
use log::info;

#[derive(Clone)]
pub struct AppState {
    master_pool: Pool,
    replica_pools: Vec<Pool>,
    pub secret: String,
    pub jwt_token_ttl_minutes: i64,    
    pub redis_pool: deadpool_redis::Pool
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

fn init_redis() -> deadpool_redis::Pool {
    let cfg = deadpool_redis::Config::from_url("redis://127.0.0.1:6379/");
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)).unwrap()
}

impl AppState {    

    pub async fn init() -> Self {        
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
        AppState {
            master_pool,
            replica_pools: vec!(replica_pool1, replica_pool2),
            secret: env::var("JWT_SECRET").unwrap(),
            jwt_token_ttl_minutes: env::var("jwt_token_ttl_minutes").unwrap().parse().unwrap(),
            redis_pool: init_redis()
        }
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