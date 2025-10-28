use log::{info, error};
use tokio_postgres::{NoTls, Error};
use dotenv::dotenv;
use axum::{routing::get, Router, extract::State};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use std::sync::Arc;

mod embedded {
    refinery::embed_migrations!("migrations");
}

#[derive(Clone)]
struct AppState {
    pool: Pool
}

impl AppState {
    async fn init_pool() -> Self {
        let mut config = Config::new();
        config.user = Some("pguser".to_string());
        config.password = Some("pgpassword".to_string());
        config.dbname = Some("highload".to_string());
        config.host = Some("localhost".to_string());
        config.port = Some(5432);
        config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
        let pool = config.create_pool(None, NoTls).unwrap();
        pool.resize(10);
        AppState {
            pool,
        }
    }
}

async fn handle_root(State(state): State<Arc<AppState>>) -> String {
    let client = state.pool.get().await.unwrap();
    let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
    let rows = client.query(&stmt, &[&21]).await.unwrap();
    let value: i32 = rows[0].get(0);
    "Basics)".to_string() + &value.to_string()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();    
    let (mut client, connection) = tokio_postgres::connect(
        "host=localhost user=pguser password=pgpassword dbname=highload",
        NoTls
    ).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Db connection error: {}", e);
        }
    });    
    info!("Running DB migrations...");
    let report = embedded::migrations::runner().run_async(&mut client).await.unwrap();
    for migration in report.applied_migrations() {
        info!("Migration Applied - Name: {}, Version: {}", migration.name(), migration.version());
    }
    info!("Applied {} migrations", report.applied_migrations().len());  

    let app_state = AppState::init_pool().await;

    let app = Router::new()
        .route("/", get(handle_root))
        .with_state(Arc::new(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
