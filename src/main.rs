use log::{info};
use tokio_postgres::{Error};
use dotenv::dotenv;
use axum::{routing::get, Router, extract::State, extract::Path};
use std::sync::Arc;
use app_state::AppState;

mod app_state;
mod migrations;

async fn handle_get_user(Path(id): Path<String>, State(state): State<Arc<AppState>>) -> String {
    let client = state.pool.get().await.unwrap();
    let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
    let rows = client.query(&stmt, &[&2]).await.unwrap();
    let value: i32 = rows[0].get(0);    
    "Basics)".to_string() + &value.to_string() + &id
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();    
    let app_state = Arc::new(AppState::init().await);
    migrations::run_migrations(app_state.clone()).await;
    let app = Router::new()
        .route("/user/get/{user_id}", get(handle_get_user))
        .with_state(app_state);    
    let port = 3000;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    info!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
