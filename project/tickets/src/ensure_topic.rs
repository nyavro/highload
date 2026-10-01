use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use tracing::{info, warn,error};

pub async fn ensure_kafka_topic_exists(brokers: &str, topic_name: &str, partitions: i32) {    
    let admin_client: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .expect("Failed to create Kafka AdminClient");
    let new_topic = NewTopic::new(topic_name, partitions, TopicReplication::Fixed(1));

    info!("Checking if Kafka topic '{}' exists...", topic_name);
    
    match admin_client.create_topics(&[new_topic], &AdminOptions::new()).await {
        Ok(results) => {
            for result in results {
                match result {
                    Ok(name) => info!("Kafka topic '{}' created successfully automatically!", name),
                    Err((name, err)) => {
                        if err.to_string().contains("already exists") || err.to_string().contains("TopicAlreadyExists") {
                            info!("Kafka topic '{}' already exists, creation is not required.", name);
                        } else {
                            warn!("Warning while creating topic '{}': {}", name, err);
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to execute request for creating topics: {}", e);
        }
    }
}
