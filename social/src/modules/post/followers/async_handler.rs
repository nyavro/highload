use axum::http::StatusCode;
use axum::{
    extract::ws::WebSocketUpgrade,
    response::IntoResponse,
};
use axum::extract::Query;
use std::sync::Arc;
use crate::app_state::AppState;
use crate::modules::auth::auth::verify_token;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct WebSocketQuery {
    token: String
}

pub async fn post_feed_ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {      
    let claims = match verify_token(&params.token, &state.secret.as_bytes()) {
        Ok(uid) => uid,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
        }
    };      
    ws.on_upgrade(move |socket| async move {
        state.ws_manager.handle_connection(claims.user_id, socket).await;}
    )
}