use std::sync::Arc;

use deadpool_lapin::{Pool, lapin::{BasicProperties, options::BasicPublishOptions}};
use log::info;

use crate::modules::post::event::DomainEvent;

pub struct RabbitPublisher {
    pool: Arc<Pool>,
    exchange: String,
}

impl RabbitPublisher {
    pub fn new(pool: Arc<Pool>, exchange: String) -> Self {        
        Self { pool, exchange }
    }

    pub async fn publish(&self, event: &DomainEvent) -> Result<(), Box<dyn std::error::Error>> {
        info!("Publishing event...");
        let conn = self.pool.get().await?;
        let channel = conn.create_channel().await?;
        let payload = serde_json::to_vec(event)?;        
        let routing_key = match event {
            DomainEvent::PostCreated { .. } => "post.created",
            DomainEvent::PostUpdated { .. } => "post.updated",
            DomainEvent::PostDeleted { .. } => "post.deleted",
        };
        info!("Publishing event: {:?} by key: {:?}", event, routing_key);
        channel.basic_publish(
            &self.exchange,
            routing_key,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        ).await?;
        info!("Event published: {:?} by key: {:?}", event, routing_key);
        Ok(())
    }
}