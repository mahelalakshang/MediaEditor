use common::{
    MediaProcessed, MediaProcessingFailed, MediaStatus, MediaThumbnailReady,
    TOPIC_MEDIA_PROCESSED, TOPIC_MEDIA_PROCESSING_DLQ, TOPIC_MEDIA_THUMBNAIL_READY,
};
use futures_util::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct StatusEvent {
    pub asset_id: String,
    pub status: MediaStatus,
    pub message: Option<String>,
    pub updated_at_ms: i64,
    pub progress_hint: Option<String>,
}

/// Consumes the worker's result topics and republishes them into an
/// in-process broadcast channel, which is what actually powers the
/// StreamMediaStatus gRPC RPC. This is the "Kafka -> in-process pub/sub
/// -> push to clients" leg described in the plan doc — a distinct
/// consumer group from media-worker's, and `auto.offset.reset=latest`
/// since this is a live feed, not a replay: a new subscriber always gets
/// the current status from SQLite first (see grpc.rs), so history here
/// doesn't matter.
pub async fn run(kafka_brokers: String, tx: broadcast::Sender<StatusEvent>) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("group.id", "core-service-status-watcher")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "latest")
        .create()
        .expect("failed to create Kafka consumer for status watcher");

    consumer
        .subscribe(&[
            TOPIC_MEDIA_THUMBNAIL_READY,
            TOPIC_MEDIA_PROCESSED,
            TOPIC_MEDIA_PROCESSING_DLQ,
        ])
        .expect("failed to subscribe to status topics");

    tracing::info!("status watcher subscribed to worker result topics");

    let mut stream = consumer.stream();
    while let Some(message) = stream.next().await {
        let borrowed = match message {
            Ok(m) => m,
            Err(err) => {
                tracing::error!(error = %err, "status watcher Kafka consumer error");
                continue;
            }
        };

        let Some(topic) = Some(borrowed.topic()) else {
            continue;
        };
        let Some(payload) = borrowed.payload() else {
            continue;
        };

        let event = match topic {
            TOPIC_MEDIA_THUMBNAIL_READY => {
                serde_json::from_slice::<MediaThumbnailReady>(payload)
                    .ok()
                    .map(|e| StatusEvent {
                        asset_id: e.asset_id,
                        status: MediaStatus::Processing,
                        message: Some("thumbnail selected, finishing up".to_string()),
                        updated_at_ms: chrono::Utc::now().timestamp_millis(),
                        progress_hint: Some("thumbnail_ready".to_string()),
                    })
            }
            TOPIC_MEDIA_PROCESSED => serde_json::from_slice::<MediaProcessed>(payload)
                .ok()
                .map(|e| StatusEvent {
                    asset_id: e.asset_id,
                    status: MediaStatus::Processed,
                    message: Some("processing complete".to_string()),
                    updated_at_ms: chrono::Utc::now().timestamp_millis(),
                    progress_hint: None,
                }),
            TOPIC_MEDIA_PROCESSING_DLQ => serde_json::from_slice::<MediaProcessingFailed>(payload)
                .ok()
                .map(|e| StatusEvent {
                    asset_id: e.asset_id,
                    status: MediaStatus::Failed,
                    message: Some(e.error_message),
                    updated_at_ms: chrono::Utc::now().timestamp_millis(),
                    progress_hint: None,
                }),
            _ => None,
        };

        if let Some(event) = event {
            // Errors here just mean nobody is currently subscribed for
            // this asset — not a problem, the DB remains the source of
            // truth for anyone who queries afterwards.
            let _ = tx.send(event);
        }
    }
}
