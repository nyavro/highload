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
    config.port = env::var(port_key).ok().map(|port| port.parse().unwrap());
    config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });        
    config.connect_timeout = Some(Duration::from_secs(10));        
    config
}

impl AppState {    

    pub async fn init() -> anyhow::Result<Self> { 
        let master_pool = init_config(
                "db_postgres_master_port"
            )
            .create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        master_pool.resize(10);        
        let manager = TarantoolManager::new(&env::var("TARANTOOL_URL").ok().unwrap());
        let pool = managed::Pool::builder(manager)
            .max_size(10) 
            .build()
            .unwrap();                
        let port = env::var("APPLICATION_PORT").ok().map(|port| port.parse().unwrap()).unwrap();
        Ok(
            AppState {                
                master_pool: Arc::new(master_pool),
                port,                
                secret: env::var("JWT_SECRET").unwrap(),                
                dialog_service: create_service(Arc::new(pool)),                
            }
        )        
    }

    pub async fn get_master_client(&self) -> Object {
        self.master_pool.get().await.unwrap()
    }    
}