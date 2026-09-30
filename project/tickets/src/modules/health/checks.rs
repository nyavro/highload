use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use fred::interfaces::ClientLike;

use crate::app_state::AppState;

pub async fn redis_health_check(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.redis_pool.next();    
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

pub async fn health_check() -> axum::response::Json<serde_json::Value> {
    Json(
        serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    )
}
