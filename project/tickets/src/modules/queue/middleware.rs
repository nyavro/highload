use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::app_state::AppState;
use crate::modules::queue::repository::RedisQueueRepository;

// Структура для парсинга тела запроса (нужен user_id)
#[derive(Deserialize, Clone)]
struct UserPayload {
    user_id: Uuid,
}

#[derive(Serialize)]
pub struct QueueRejectResponse {
    pub status: String,
    pub message: String,
    pub queue_position: i64,
}

async fn read_payload(request: Request<Body>) -> Result<(UserPayload, Request<Body>), (StatusCode, Json<QueueRejectResponse>)> {    
    let (parts, body) = request.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(QueueRejectResponse {
            status: "error".to_string(),
            message: "Invalid request body".to_string(),
            queue_position: 0,
        }))
    })?;
    let payload = serde_json::from_slice(&bytes).map_err(|_| {
        (StatusCode::BAD_REQUEST, Json(QueueRejectResponse {
            status: "error".to_string(),
            message: "Missing user_id in payload".to_string(),
            queue_position: 0,
        }))
    })?;

    Ok((payload, Request::from_parts(parts, Body::from(bytes))))
}

pub async fn queue_protection_middleware(
    Path(event_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<QueueRejectResponse>)> {
    let (payload, request) = read_payload(request).await?;
    let queue_repo = RedisQueueRepository::new(state.redis_pool.clone());
    if queue_repo.is_session_active(event_id, payload.user_id).await.unwrap_or(false) {    
        return Ok(next.run(request).await);
    }
    
    // Fast Path check if under heavy load
    // TODO: move to AppState or env variable
    let system_limit = std::env::var("QUEUE_SYSTEM_LIMIT")
        .unwrap_or_else(|_| "50".to_string())
        .parse::<i64>()
        .unwrap_or(50);     
    let current_active_chunks: i64 = queue_repo.get_active_sessions(event_id).await;
    if current_active_chunks < system_limit { 
        // Fast Path: system is not overloaded, allow user to proceed                
        queue_repo.move_user_to_active(event_id, payload.user_id).await;        
        return Ok(next.run(request).await);
    }

    let current_time = Utc::now().timestamp_millis();
    let position = queue_repo.join_queue(event_id, payload.user_id, current_time).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(QueueRejectResponse {
            status: "error".to_string(),
            message: format!("Queue system failure: {}", e),
            queue_position: 0,
        }))
    })?;
    
    Err((
        StatusCode::TOO_MANY_REQUESTS,
        Json(QueueRejectResponse {
            status: "in_queue".to_string(),
            message: "System is at capacity. You have been placed in a virtual waiting room.".to_string(),
            queue_position: position,
        }),
    ))
}
