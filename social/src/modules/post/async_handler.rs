use axum::{
    extract::ws::WebSocketUpgrade,
    response::IntoResponse,
};
use axum::extract::Query;
use std::sync::Arc;
use crate::app_state::AppState;
use serde::Deserialize;
use uuid::Uuid;


#[derive(Deserialize)]
pub struct WebSocketQuery {
    user_id: Uuid,
}

pub async fn post_feed_ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {        
    ws.on_upgrade(move |socket| async move {
        state.ws_manager.handle_connection(params.user_id, socket).await;}
    )
}