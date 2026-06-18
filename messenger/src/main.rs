use tokio_postgres::{Error};
use dotenv::dotenv;
use std::sync::Arc;
use app_state::AppState;
use application::Application;
use http::header::HeaderName;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tower_http::request_id::RequestId;
use tracing::Level;

mod app_state;
mod migrations;
mod application;
mod modules;

fn init_env() {
    dotenv::from_filename(".env.secret").ok();    
    dotenv().ok();    
    tracing_subscriber::fmt()        
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into())
        )        
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .init();       
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_env();
    let app_state = Arc::new(AppState::init().await.unwrap());
    // migrations::run_migrations(app_state.clone()).await;
    // mock::service::generate_messages(Arc::clone(&app_state)).await;
    let x_request_id = HeaderName::from_static("x-request-id");
    let app = openapi::server::new(Application::new(Arc::clone(&app_state)))
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))        
        .layer(
            TraceLayer::new_for_http()                
                .make_span_with(|request: &http::Request<_>| {
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .map(|id| id.header_value().to_str().unwrap_or("unknown"))
                        .unwrap_or("unknown");
                    tracing::info_span!(
                        "request",
                        request_id = %request_id,
                        method = %request.method(),
                        uri = %request.uri().path(),
                        version = ?request.version(),
                    )
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )        
        .layer(SetRequestIdLayer::new(x_request_id.clone(), MakeRequestUuid));;        
    let port = app_state.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    tracing::info!("Server is running on port {}", port);
    axum::serve(listener, app).await.unwrap();
    Ok(())
} 