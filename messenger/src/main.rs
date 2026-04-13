use log::info;
use tokio_postgres::{Error};
use dotenv::dotenv;
use std::sync::Arc;
use app_state::AppState;
use application::Application;

mod app_state;
mod migrations;
mod application;
mod modules;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::from_filename(".env.secret").ok();    
    dotenv().ok();    
    env_logger::init();        
    let app_state = Arc::new(AppState::init().await.unwrap());
    // migrations::run_migrations(app_state.clone()).await;
    // mock::service::generate_messages(Arc::clone(&app_state)).await;
    let app = openapi::server::new(Application::new(Arc::clone(&app_state)));        
    let port = app_state.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    info!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
    Ok(())
} 