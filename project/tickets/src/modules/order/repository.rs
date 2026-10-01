use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use crate::modules::order::dto::OrderPaidEvent;

pub struct KafkaOrderRepository {
    producer: Arc<FutureProducer>,
    topic_name: String,
}

impl KafkaOrderRepository {
    pub fn new(producer: Arc<FutureProducer>, topic_name: String) -> Self {
        Self { producer, topic_name }
    }

    pub async fn send_order_paid_event(&self, event: &OrderPaidEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        let payload = serde_json::to_string(event)?;
        // Using event_id as Kafka key.         
        let key_str = event.event_id.to_string();
        let record = FutureRecord::to(&self.topic_name)
            .key(&key_str)
            .payload(&payload);
        // Async producing message with 5 seconds timeout
        self.producer
            .send(record, Timeout::After(Duration::from_secs(5)))
            .await
            .map_err(|(err, _)| err)?;
        Ok(())
    }
}
