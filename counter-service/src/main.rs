mod modules;
mod app_state;

use std::error::Error;
use tokio::net::TcpListener;
use dotenv::dotenv;
use tracing::info;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_env();  
    init_tracing();      

    let state = AppState::init().await?;

    let port = std::env::var("APPLICATION_PORT").unwrap_or_else(|_| "3004".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Started at {:?}", port);
    axum::serve(
        listener, 
        router::router(state)
    ).await?;
    Ok(())
}
