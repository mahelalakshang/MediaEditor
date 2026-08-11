mod db;
mod error;
mod grpc;
mod handlers;
mod models;
mod state;
mod status_watcher;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use common::proto::media_service_server::MediaServiceServer;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use grpc::MediaGrpcService;
use state::AppState;

const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let media_dir = PathBuf::from(&data_dir).join("media");
    let db_dir = PathBuf::from(&data_dir).join("db");
    tokio::fs::create_dir_all(&media_dir).await?;
    tokio::fs::create_dir_all(&db_dir).await?;

    let db_path = db_dir.join("app.db");
    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_producer = common::kafka::build_producer(&kafka_brokers);

    let (status_tx, _) = tokio::sync::broadcast::channel(1024);
    tokio::spawn(status_watcher::run(kafka_brokers.clone(), status_tx.clone()));

    let state = Arc::new(AppState {
        db: pool,
        media_dir: media_dir.clone(),
        kafka_producer,
        status_tx,
    });

    // Renditions are stored with portable relative paths (e.g.
    // "{asset_id}/thumbnail.jpg") precisely so they can be served here
    // and fetched directly by the frontend — see models.rs / processor.rs.
    let app = Router::new()
        .route("/healthz", get(handlers::health))
        .route("/uploads", post(handlers::upload))
        .route("/assets", get(handlers::list_assets))
        .route("/assets/:id", get(handlers::get_asset))
        .nest_service("/media", ServeDir::new(media_dir))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    let http_server = async {
        tracing::info!("core-service REST listening on 0.0.0.0:{port}");
        axum::serve(listener, app).await.map_err(anyhow::Error::from)
    };

    let grpc_port: u16 = std::env::var("GRPC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50051);
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], grpc_port).into();
    let grpc_service = MediaGrpcService { state };
    let grpc_server = async {
        tracing::info!("core-service gRPC listening on {grpc_addr}");
        tonic::transport::Server::builder()
            .add_service(MediaServiceServer::new(grpc_service))
            .serve(grpc_addr)
            .await
            .map_err(anyhow::Error::from)
    };

    tokio::try_join!(http_server, grpc_server)?;

    Ok(())
}
