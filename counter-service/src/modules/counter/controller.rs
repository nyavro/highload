use serde::Serialize;
use axum::{Json, extract::{Path, State}, http::StatusCode};
use crate::modules::counter::repository::CounterRepositoryImpl;
use crate::modules::counter::repository::CounterRepository;
use crate::app_state::AppState;

#[derive(Serialize)]
pub struct CounterResponse {
    pub user_id: String,
    pub count: i64
}

pub async fn increment_counter(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<CounterResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = CounterRepositoryImpl::new(state.redis_pool.clone());
    let count = repo.increment(&user_id).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()}))
        ))?;
    Ok(Json(CounterResponse {user_id, count}))
}
