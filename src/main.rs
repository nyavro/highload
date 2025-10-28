use log::{info};
use tokio_postgres::{Error};
use dotenv::dotenv;
use axum::{routing::get, Router, extract::State};
use std::sync::Arc;
use app_state::AppState;

mod app_state;
mod migrations;

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
    let app_state = Arc::new(AppState::init().await);
    migrations::run_migrations(app_state.clone()).await;
    let app = Router::new()
        .route("/", get(handle_root))
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
