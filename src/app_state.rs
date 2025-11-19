use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::{NoTls};
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub secret: String,
}

impl AppState {
    pub async fn init() -> Self {
        let mut config = Config::new();
        config.user = env::var("db_postgres_user").ok();        
        config.password = env::var("db_postgres_password").ok();
        config.dbname = env::var("db_postgres_dbname").ok();
        config.host = env::var("db_postgres_host").ok();
        config.port = env::var("db_postgres_port").ok().map(|port| port.parse().unwrap());
        config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });        
        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        pool.resize(10);
        AppState {
            pool,
            secret: env::var("JWT_SECRET").unwrap()
        }
    }
}