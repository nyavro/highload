use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use crate::app_state::AppState;
use crate::modules::queue::repository::RedisQueueRepository;

#[derive(Deserialize)]
pub struct StatusQueryParams {
    pub user_id: Uuid,
}

#[derive(Serialize)]
pub struct QueueStatusOutput {
    pub status: String, // "allowed" / "waiting" / "not_in_queue"
    pub queue_position: i64,
}

pub async fn get_queue_status_handler(
    Path(event_id): Path<Uuid>,
    Query(params): Query<StatusQueryParams>,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<QueueStatusOutput>), (StatusCode, String)> {
    let queue_repo = RedisQueueRepository::new(state.redis_pool.clone());
    match queue_repo.get_queue_status(event_id, params.user_id).await {
        Ok(None) => {                
            Ok((
                StatusCode::OK,
                Json(QueueStatusOutput {
                    status: "allowed".to_string(),
                    queue_position: 0,
                }),
            ))
        }
        Ok(Some(position)) if position > 0 => {                
            Ok((
                StatusCode::OK,
                Json(QueueStatusOutput {
                    status: "waiting".to_string(),
                    queue_position: position,
                }),
            ))
        }
        Ok(Some(0)) | Ok(Some(_)) => {                
            Ok((
                StatusCode::OK,
                Json(QueueStatusOutput {
                    status: "not_in_queue".to_string(),
                    queue_position: 0,
                }),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch queue status: {}", e),
        )),
    }
}
