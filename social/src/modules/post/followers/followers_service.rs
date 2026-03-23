use std::sync::Arc;
use async_trait::async_trait;
use deadpool_lapin::{Pool, lapin::{Consumer, ExchangeKind, options::{BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, types::{AMQPValue, FieldTable, ShortString}}};
use log::{info, warn};
use uuid::Uuid;
use crate::modules::{friend::repository::{FriendRepository, FriendRepositoryError}, post::{event::DomainEvent, followers::follower_event_bus::{EventBus, FollowerEvent, FollowerEventListener}}};
use tokio_stream::StreamExt;

#[async_trait]
pub trait FollowersService {
    async fn run_consumer(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct FollowersServiceImpl<F> 
where 
    F: FriendRepository {            
    repository: F,    
    event_bus: EventBus,
    pool: Arc<Pool>,
    exchange: String
}

impl <F> FollowersServiceImpl<F>
where 
    F: FriendRepository + Send + Sync {
    pub fn new(repository: F, listeners: Vec<Arc<dyn FollowerEventListener + Send + Sync>>, pool: Arc<Pool>, exchange: String) -> Self {
        FollowersServiceImpl { 
            repository,
            event_bus: EventBus::new(100, listeners),
            pool,
            exchange
        }
    }

    async fn fetch_followers(&self, user_id: Uuid) -> Result<Vec<Uuid>, FriendRepositoryError> {
        Ok(self.repository.get_followers_ids(user_id).await?)
    }

    async fn init_consumer(&self) -> Result<Consumer, Box<dyn std::error::Error>> {
        let conn = self.pool.get().await?;
        let channel = conn.create_channel().await?;
        channel.exchange_declare(
            "dlx_exchange", 
            ExchangeKind::Direct, 
            ExchangeDeclareOptions::default(), 
            FieldTable::default()
        ).await?;
        channel.exchange_declare(
            &self.exchange, 
            ExchangeKind::Topic, 
            ExchangeDeclareOptions::default(), 
            FieldTable::default()
        ).await?;
        channel.queue_declare("failed_posts", QueueDeclareOptions::default(), FieldTable::default()).await?;
        channel.queue_bind("failed_posts", "dlx_exchange", "failed", QueueBindOptions::default(), FieldTable::default()).await?;
        let mut args = FieldTable::default();
        args.insert("x-dead-letter-exchange".into(), AMQPValue::LongString("dlx_exchange".into()));
        args.insert("x-dead-letter-routing-key".into(), AMQPValue::LongString("failed".into()));
        channel.queue_declare("post_events", QueueDeclareOptions::default(), args).await?;
        channel.queue_bind("post_events", &self.exchange, "post.*", QueueBindOptions::default(), FieldTable::default()).await?;
        Ok(channel.basic_consume("post_events", "worker", BasicConsumeOptions::default(), FieldTable::default()).await?)
    }    
}

#[async_trait]
impl <F> FollowersService for FollowersServiceImpl<F>
where 
    F: FriendRepository + Send + Sync {
    async fn run_consumer(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut consumer = self.init_consumer().await?;
        info!("Consumer started...");
        while let Some(delivery) = consumer.next().await {            
            let delivery = delivery?;
            let event: DomainEvent = serde_json::from_slice(&delivery.data)?;
            info!("Incoming event: {:?}", event);
            let user_id = event.user_id();
            match self.fetch_followers(*user_id).await {
                Ok(followers) => {
                    self.event_bus.publish(FollowerEvent {                        
                        domain_event: event,
                        followers,
                    }).await;
                },
                Err(e) => warn!("Failed to fetch followers {:?}", e)
            };    
            delivery.ack(BasicAckOptions::default()).await?;
        }
        Ok(())
    }
}