use std::path::PathBuf;

use rdkafka::producer::FutureProducer;
use sqlx::SqlitePool;
use tokio::sync::broadcast;

use crate::status_watcher::StatusEvent;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub media_dir: PathBuf,
    pub kafka_producer: FutureProducer,
    pub status_tx: broadcast::Sender<StatusEvent>,
}
