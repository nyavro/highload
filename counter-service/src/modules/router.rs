use axum::extract::State;
use axum::{Router};
use axum::routing::{get};
use axum::http::StatusCode;
use axum::response::Json;
use fred::interfaces::ClientLike;

use crate::app_state::AppState;

async fn redis_health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut client = state.redis_pool.next();    
    let result = client.ping::<String>(None).await;
    match result {
        Ok(_) => Ok(Json(
                serde_json::json!({
                    "status": "ok",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            )),
        Err(_) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Redis unavailable","timestamp": chrono::Utc::now().to_rfc3339()}))
        ))
    }
}

async fn health_check() -> axum::response::Json<serde_json::Value> {
    Json(
        serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    )
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/redis/health", get(redis_health_check))
        .with_state(state)
}