use dashmap::DashMap;
use tokio::sync::{mpsc::{UnboundedSender, unbounded_channel}};
use axum::extract::ws::{Message, WebSocket};
use uuid::Uuid;
use futures_util::StreamExt;
use futures_util::SinkExt;
use serde::Serialize;
use std::collections::HashMap;
use serde_json::Error;
use log::info;

type UserId = Uuid;
type Tx = UnboundedSender<Message>; 

pub struct WebSocketManager {
    user_connections: DashMap<UserId, Vec<Tx>>
}

impl WebSocketManager {
    pub fn new() -> Self {
        WebSocketManager { user_connections: DashMap::new() }
    }

    pub fn register(&self, user_id: UserId, tx: Tx) {
        info!("Registering user id {:?}", user_id);
        self.user_connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(tx);
    }

    pub fn unregister(&self, user_id: UserId, tx: &Tx) {
        if let Some(mut senders) = self.user_connections.get_mut(&user_id) {
            senders.retain(|sender| !sender.same_channel(tx));
            if senders.is_empty() {
                drop(senders);
                self.user_connections.remove(&user_id);
            }
        }
    }

    fn send_to_user_raw(&self, user_id: UserId, message: String) -> Result<usize, Error> {
        let mut sent = 0;
        if let Some(senders) = self.user_connections.get(&user_id) {            
            let msg = Message::Text(message.into());
            for tx in senders.value() {
                if tx.send(msg.clone()).is_ok() {
                    sent += 1;
                }
            }
        }
        Ok(sent)
    }

    pub fn send_to_users<T: Serialize>(&self, user_ids: &[Uuid], message: &T) -> Result<HashMap<Uuid, usize>, Error> {
        let payload = serde_json::to_string(message)?;
        let mut results = HashMap::new();
        for &uid in user_ids {
            let sent = self.send_to_user_raw(uid, payload.clone())?;
            results.insert(uid, sent);
        }
        Ok(results)
    }

    pub async fn handle_connection(&self, user_id: UserId, socket: WebSocket) {
        let (mut sender, mut receiver) = socket.split();
        let (tx, mut rx) = unbounded_channel::<Message>();

        self.register(user_id, tx.clone());

        let mut send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if sender.send(msg).await.is_err() {
                    break;
                }
            }
        });

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Ping(_) => {}
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        });

        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        }

        self.unregister(user_id, &tx);
    }
}