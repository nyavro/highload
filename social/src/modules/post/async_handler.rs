use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::Arc;
use crate::app_state::AppState;
use futures_util::StreamExt;
use futures_util::SinkExt;
use log::info;

pub async fn post_feed_ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, _) = socket.split();
    let mut rx = state.tx.subscribe();
    tokio::spawn(async move {
        while let Ok(post) = rx.recv().await {
            info!("Sending post {:?}", post);
            if let Ok(msg) = serde_json::to_string(&post) {            
                if let Err(e) = sender.send(Message::Text(msg.into())).await {
                    info!("Client disconnected: {:?}", e);
                    break;
                }
            }
        }
    });
}