mod modules;
mod app_state;
mod migrations;

use std::{error::Error, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use dotenv::dotenv;
use tracing::info;

use app_state::AppState;
use modules::router;

use crate::modules::reconciliation::reconciler;

fn init_env() {
    dotenv::from_filename(".env.secret").ok();    
    dotenv().ok();
}

fn init_tracing() {
    tracing_subscriber::fmt()        
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into())
        )        
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .init(); 
}

async fn reconcile(state: Arc<AppState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let interval = std::env::var("RECONCILIATION_INTERVAL")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(60));
    tokio::spawn(reconciler::run_reconciliation_loop(Arc::clone(&state), interval));
    info!("Reconciliation: interval = {:?}", interval);
    Ok(())
}

async fn serve(state: Arc<AppState>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let port = std::env::var("APPLICATION_PORT").unwrap_or_else(|_| "3004".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Started at {:?}", port);
    axum::serve(
        listener, 
        router::router(state)
    ).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_env();  
    init_tracing();      
    let state = Arc::new(AppState::init().await?);
    migrations::run_migrations(Arc::clone(&state.postgres_pool)).await;
    reconcile(Arc::clone(&state)).await?;
    serve(Arc::clone(&state)).await?;
    Ok(())
}