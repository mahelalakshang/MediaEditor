use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

/// Shared by core-service and media-worker — both need the exact same
/// producer setup, so it lives here instead of being copy-pasted twice.
pub fn build_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("failed to create Kafka producer")
}

/// Serializes `payload` as JSON and publishes it, keyed by `key` (the
/// asset id) so all events for one asset land on the same partition and
/// stay ordered relative to each other.
pub async fn publish<T: serde::Serialize>(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    payload: &T,
) -> Result<(), rdkafka::error::KafkaError> {
    let json = serde_json::to_vec(payload).expect("event type must serialize to JSON");
    let record = FutureRecord::to(topic).key(key).payload(&json);
    producer
        .send(record, Duration::from_secs(5))
        .await
        .map(|_| ())
        .map_err(|(err, _)| err)
}
