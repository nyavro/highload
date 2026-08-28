use axum::Router;
use axum::routing::{get};

async fn health_check() -> axum::response::Json<serde_json::Value> {
    axum::response::Json(
        serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    )
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health_check))
}