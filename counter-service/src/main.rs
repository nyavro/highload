mod modules;

use std::error::Error;
use std::net::{SocketAddr};
use tokio::net::TcpListener;
use dotenv::dotenv;

use modules::router;

fn init_env() {
    dotenv::from_filename(".env.secret").ok();    
    dotenv().ok();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    init_env();    
    let app = router::router();
    let port = std::env::var("APPLICATION_PORT")
        .unwrap_or_else(|_| "3004".to_string());
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
