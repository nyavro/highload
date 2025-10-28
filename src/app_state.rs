use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::{NoTls};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool
}

impl AppState {
    pub async fn init() -> Self {
        let mut config = Config::new();
        config.user = Some("pguser".to_string());
        config.password = Some("pgpassword".to_string());
        config.dbname = Some("highload".to_string());
        config.host = Some("localhost".to_string());
        config.port = Some(5432);
        config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
        pool.resize(10);
        AppState {
            pool,
        }
    }
}