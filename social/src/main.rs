use log::info;
use tokio_postgres::{Error};
use dotenv::dotenv;
use std::sync::Arc;
use app_state::AppState;
use application::Application;
use axum::{
    routing::get,
    Router,
};
use modules::post::async_handler::post_feed_ws_handler;

mod app_state;
mod migrations;
mod application;
mod load_metric_utils;
mod modules;

fn async_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/post/feed/posted", get(post_feed_ws_handler))
        .with_state(state)
}

fn init_env() {
    dotenv::from_filename(".env.secret").ok();    
    dotenv().ok();    
    env_logger::init();        
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_env();
    let app_state = Arc::new(AppState::init().await.unwrap());
    migrations::run_migrations(app_state.clone()).await;
    load_metric_utils::generate_load_data(app_state.clone()).await;    
    let sync_routes = openapi::server::new(Application::new(Arc::clone(&app_state)));
    let async_routes = async_routes(Arc::clone(&app_state));
    let app = sync_routes.merge(async_routes);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", app_state.port)).await.unwrap();
    info!("Server is running on port {}", app_state.port);
    axum::serve(listener, app).await.unwrap();
    Ok(())
} 