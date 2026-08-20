use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use std::{env, time::Duration};
use crate::modules::{common::tarantool::tarantool_manager::TarantoolManager, dialog::service_provider::{DialogService, create_service}};
use std::sync::Arc;
use deadpool_postgres::{Runtime, Object};
use tokio_postgres::{NoTls};
use deadpool::managed;

#[derive(Clone)]
  pub struct AppState {
    master_pool: Arc<Pool>, 
    replica_pool: Option<Arc<Pool>>,
    tarantool_pool: Arc<managed::Pool<TarantoolManager>>,
    pub secret: String,
    pub port: i32,
    pub dialog_service: Arc<dyn DialogService + Send + Sync>,        
}

fn init_config(port_key: &str) -> Config {
    let mut config = Config::new();
    config.user = env::var("db_postgres_user").ok();        
    config.password = env::var("db_postgres_password").ok();
    config.dbname = env::var("db_postgres_dbname").ok();
    config.host = env::var("db_postgres_host").ok();
    config.port = env::var(port_key).ok().and_then(|port| port.parse().ok());
    config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });        
    config.connect_timeout = Some(Duration::from_secs(10));        
    config
}

impl AppState {    

    pub async fn init() -> anyhow::Result<Self> { 
        let master_pool = Arc::new(
            init_config("db_postgres_master_port")
                .create_pool(Some(Runtime::Tokio1), NoTls)
                .unwrap()
        );
        master_pool.resize(10);        
        let replica_pool: Option<Arc<Pool>> = 
            if let Ok(replica_port) = env::var("db_postgres_replica_port") {
                let mut config = init_config("db_postgres_replica_port");
                config.port = replica_port.parse().ok();
                let pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
                pool.resize(10);
                Some(Arc::new(pool))
            } else {
                tracing::warn!("db_postgres_replica_port not set: all reads to master");
                None
            };
        let manager = TarantoolManager::new(&env::var("TARANTOOL_URL").ok().unwrap());
        let tarantool_pool = Arc::new(
            managed::Pool::builder(manager)
            .max_size(10) 
            .build()
            .unwrap()
        );                
        let port = env::var("APPLICATION_PORT").ok().and_then(|port| port.parse().ok()).unwrap_or(3000);
        let dialog_service = create_service(
            Arc::clone(&tarantool_pool), 
            replica_pool
                .as_ref()
                .map(Arc::clone)
                .unwrap_or_else(|| Arc::clone(&master_pool))
        );
        Ok(
            AppState {
                replica_pool,
                tarantool_pool,                
                master_pool: master_pool,
                port,                
                secret: env::var("JWT_SECRET").unwrap(),                
                dialog_service,                
            }
        )        
    }

    pub async fn get_master_client(&self) -> Object {
        self.master_pool.get().await.unwrap()
    }    
}