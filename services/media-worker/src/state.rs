use rdkafka::producer::FutureProducer;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub kafka_producer: FutureProducer,
}
