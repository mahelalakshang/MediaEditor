//! Manual smoke test for the gRPC unary RPCs. Run core-service, then:
//!   cargo run -p core-service --example grpc_check
use common::proto::media_service_client::MediaServiceClient;
use common::proto::{GetMediaAssetRequest, ListMediaRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = MediaServiceClient::connect("http://127.0.0.1:50051").await?;

    let list = client
        .list_media(ListMediaRequest {
            page_size: 10,
            page_token: String::new(),
            kind_filter: None,
            status_filter: None,
        })
        .await?
        .into_inner();
    println!("ListMedia -> {list:#?}");

    if let Some(first) = list.assets.first() {
        let asset = client
            .get_media_asset(GetMediaAssetRequest {
                asset_id: first.id.clone(),
            })
            .await?
            .into_inner();
        println!("GetMediaAsset({}) -> {asset:#?}", first.id);
    }

    match client
        .get_media_asset(GetMediaAssetRequest {
            asset_id: "does-not-exist".to_string(),
        })
        .await
    {
        Ok(_) => println!("expected NotFound but got Ok"),
        Err(status) => println!("GetMediaAsset(missing) -> {status}"),
    }

    Ok(())
}
