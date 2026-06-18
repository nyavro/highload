use tokio_postgres::{Error};
use dotenv::dotenv;
use std::sync::Arc;
use app_state::AppState;
use application::Application;
use axum::{
    routing::get,
    Router,
};
use http::header::HeaderName;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use crate::{middleware::{CURRENT_CONTEXT, RequestContext}, modules::post::followers::async_handler::post_feed_ws_handler};
use axum::{
    response::Response,
    http::Request,
};
use reqwest::header::{AUTHORIZATION};

mod app_state;
mod migrations;
mod application;
mod load_metric_utils;
mod modules;
mod middleware;

async fn request_id_context_middleware(req: Request<axum::body::Body>, next: axum::middleware::Next) -> Response {
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .map(|id| id.header_value().to_str().unwrap_or("unknown"))
        .unwrap_or("unknown")
        .to_string();
    let jwt_token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())        
        .map(|auth_str| auth_str.to_string());
    let context = RequestContext {
        request_id,
        jwt_token,
    };
    CURRENT_CONTEXT.scope(context, next.run(req)).await
}

fn async_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/post/feed/posted", get(post_feed_ws_handler))
        .with_state(state)
}

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
    migrations::run_migrations(app_state.clone()).await;
    load_metric_utils::generate_load_data(app_state.clone()).await;    
    let sync_routes = openapi::server::new(Application::new(Arc::clone(&app_state)));
    let async_routes = async_routes(Arc::clone(&app_state));
    let x_request_id = HeaderName::from_static("x-request-id");
    let app = sync_routes.merge(async_routes)        
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))        
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(false),
                )
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )  
        .layer(axum::middleware::from_fn(request_id_context_middleware))      
        .layer(SetRequestIdLayer::new(x_request_id.clone(), MakeRequestUuid));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", app_state.port)).await.unwrap();
    tracing::info!("Server is running on port {}", app_state.port);    
    let followers_service = Arc::clone(&app_state.followers_service);    
    tokio::spawn(async move {
        if let Err(e) = followers_service.run_consumer().await {
            tracing::error!("RabbitMQ Consumer error: {:?}", e);
        }
    });
    axum::serve(listener, app).await.unwrap();
    Ok(())
} 