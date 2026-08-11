//! Manual smoke test for the StreamMediaStatus server-streaming RPC.
//! Run core-service + media-worker + Kafka, upload an asset, then:
//!   cargo run -p core-service --example grpc_stream_check -- <asset_id>
use common::proto::media_service_client::MediaServiceClient;
use common::proto::StreamMediaStatusRequest;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let asset_id = std::env::args()
        .nth(1)
        .expect("usage: grpc_stream_check <asset_id>");

    let mut client = MediaServiceClient::connect("http://127.0.0.1:50051").await?;
    let mut stream = client
        .stream_media_status(StreamMediaStatusRequest {
            asset_id: asset_id.clone(),
        })
        .await?
        .into_inner();

    println!("streaming status for {asset_id}...");
    while let Some(update) = stream.next().await {
        match update {
            Ok(update) => println!("update -> {update:?}"),
            Err(status) => {
                println!("stream error -> {status}");
                break;
            }
        }
    }
    println!("stream closed");

    Ok(())
}
