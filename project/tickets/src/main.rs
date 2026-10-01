mod modules;
mod app_state;
mod ensure_topic;
mod migrations;

use std::{error::Error, sync::Arc};
use tokio::net::TcpListener;
use dotenv::dotenv;
use tracing::{error, info};

use app_state::AppState;
use modules::router;

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
    let queue_state = state.clone();     
    let queue_limit = std::env::var("QUEUE_SYSTEM_LIMIT")
        .unwrap_or_else(|_| "50".to_string())
        .parse::<i64>()
        .unwrap_or(50);
    tokio::spawn(async move {
        let queue_repo = crate::modules::queue::repository::RedisQueueRepository::new(queue_state.redis_pool.clone());
        // TODO: process all active events from DB instead of hardcoding event_id
        let event_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap();        
        info!("Running queue promotion loop for event_id: {}", event_id);        
        loop {                        
            match queue_repo.promote_queue(event_id, queue_limit, 120).await {
                Ok(count) if count > 0 => {
                    info!("The queue has been promoted! {} users have been allowed to purchase.", count);
                }
                Err(e) => error!("Error in queue promotion thread: {}", e),
                _ => {} // Queue is empty or full, sleep and try again
            }            
            // Check every 1 second
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });
    serve(Arc::clone(&state)).await?;
    Ok(())
}