mod db;
mod error;
mod frame_scorer;
mod image_processing;
mod processor;
mod state;
mod video;

use std::path::PathBuf;

use common::{MediaUploaded, KAFKA_CONSUMER_GROUP_MEDIA_WORKER, TOPIC_MEDIA_UPLOADED};
use futures_util::StreamExt;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tracing_subscriber::EnvFilter;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let db_path = PathBuf::from(&data_dir).join("db").join("app.db");

    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_producer = common::kafka::build_producer(&kafka_brokers);

    let state = AppState {
        db: pool,
        kafka_producer,
    };

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &kafka_brokers)
        .set("group.id", KAFKA_CONSUMER_GROUP_MEDIA_WORKER)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("failed to create Kafka consumer");

    consumer
        .subscribe(&[TOPIC_MEDIA_UPLOADED])
        .expect("failed to subscribe to media.uploaded");

    tracing::info!(brokers = %kafka_brokers, "media-worker listening on media.uploaded");

    let mut stream = consumer.stream();
    while let Some(message) = stream.next().await {
        let borrowed = match message {
            Ok(m) => m,
            Err(err) => {
                tracing::error!(error = %err, "Kafka consumer error");
                continue;
            }
        };

        let Some(payload) = borrowed.payload() else {
            continue;
        };

        match serde_json::from_slice::<MediaUploaded>(payload) {
            Ok(event) => {
                tracing::info!(asset_id = %event.asset_id, kind = ?event.kind, "processing upload");
                processor::process(&state, event).await;
            }
            Err(err) => {
                tracing::error!(error = %err, "failed to deserialize MediaUploaded event, skipping");
            }
        }
    }

    Ok(())
}
