use std::pin::Pin;
use std::sync::Arc;

use common::proto::media_service_server::MediaService;
use common::proto::{
    GetMediaAssetRequest, ListMediaRequest, ListMediaResponse, MediaAsset as ProtoMediaAsset,
    MediaKind as ProtoMediaKind, MediaStatus as ProtoMediaStatus, MediaStatusUpdate,
    Rendition as ProtoRendition, RenditionType as ProtoRenditionType, StreamMediaStatusRequest,
};
use futures_util::Stream;
use tokio::sync::broadcast;
use tonic::{Request, Response, Status};

use crate::db;
use crate::models::{MediaAssetRow, MediaRenditionRow};
use crate::state::AppState;
use crate::status_watcher::StatusEvent;

pub struct MediaGrpcService {
    pub state: Arc<AppState>,
}

fn proto_kind(kind: common::MediaKind) -> i32 {
    match kind {
        common::MediaKind::Image => ProtoMediaKind::Image as i32,
        common::MediaKind::Video => ProtoMediaKind::Video as i32,
    }
}

fn proto_status(status: common::MediaStatus) -> i32 {
    match status {
        common::MediaStatus::Uploaded => ProtoMediaStatus::Uploaded as i32,
        common::MediaStatus::Processing => ProtoMediaStatus::Processing as i32,
        common::MediaStatus::Processed => ProtoMediaStatus::Processed as i32,
        common::MediaStatus::Failed => ProtoMediaStatus::Failed as i32,
    }
}

fn proto_rendition_type(rendition_type: common::RenditionType) -> i32 {
    match rendition_type {
        common::RenditionType::Thumbnail => ProtoRenditionType::Thumbnail as i32,
        common::RenditionType::CandidateFrame => ProtoRenditionType::CandidateFrame as i32,
        common::RenditionType::PreviewClip => ProtoRenditionType::PreviewClip as i32,
    }
}

fn to_proto_rendition(row: MediaRenditionRow) -> ProtoRendition {
    let rendition_type = row
        .rendition_type
        .parse::<common::RenditionType>()
        .expect("rendition_type column holds an invalid enum value");
    ProtoRendition {
        id: row.id,
        rendition_type: proto_rendition_type(rendition_type),
        path: row.path,
        width: row.width as u32,
        height: row.height as u32,
        score: row.score,
        is_selected: row.is_selected,
        timestamp_seconds: row.timestamp_seconds,
    }
}

fn to_proto_asset(row: MediaAssetRow, renditions: Vec<MediaRenditionRow>) -> ProtoMediaAsset {
    let kind = row
        .kind
        .parse::<common::MediaKind>()
        .expect("kind column holds an invalid enum value");
    let status = row
        .status
        .parse::<common::MediaStatus>()
        .expect("status column holds an invalid enum value");
    ProtoMediaAsset {
        id: row.id,
        original_filename: row.original_filename,
        mime_type: row.mime_type,
        kind: proto_kind(kind),
        status: proto_status(status),
        created_at: row.created_at,
        renditions: renditions.into_iter().map(to_proto_rendition).collect(),
    }
}

fn to_proto_status_update(event: StatusEvent) -> MediaStatusUpdate {
    MediaStatusUpdate {
        asset_id: event.asset_id,
        status: proto_status(event.status),
        message: event.message,
        updated_at_ms: event.updated_at_ms,
        progress_hint: event.progress_hint,
    }
}

fn is_terminal(status: common::MediaStatus) -> bool {
    matches!(
        status,
        common::MediaStatus::Processed | common::MediaStatus::Failed
    )
}

const DEFAULT_PAGE_SIZE: usize = 20;

#[tonic::async_trait]
impl MediaService for MediaGrpcService {
    async fn get_media_asset(
        &self,
        request: Request<GetMediaAssetRequest>,
    ) -> Result<Response<ProtoMediaAsset>, Status> {
        let asset_id = request.into_inner().asset_id;
        let row = db::fetch_asset_row(&self.state.db, &asset_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("asset not found"))?;
        let renditions = db::fetch_renditions(&self.state.db, &asset_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(to_proto_asset(row, renditions)))
    }

    async fn list_media(
        &self,
        request: Request<ListMediaRequest>,
    ) -> Result<Response<ListMediaResponse>, Status> {
        let req = request.into_inner();
        let page_size = if req.page_size == 0 {
            DEFAULT_PAGE_SIZE
        } else {
            req.page_size as usize
        };
        // Offset-encoded page token — see plan's "Explicit Cuts": simple
        // by design, kept behind a cursor-shaped wire contract so it can
        // be swapped for a real cursor later without a client-facing change.
        let offset: usize = req.page_token.parse().unwrap_or(0);

        let all_rows = db::list_asset_rows(&self.state.db)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let page: Vec<_> = all_rows.into_iter().skip(offset).take(page_size).collect();
        let is_last_page = page.len() < page_size;

        let mut assets = Vec::with_capacity(page.len());
        for row in page {
            let renditions = db::fetch_renditions(&self.state.db, &row.id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            assets.push(to_proto_asset(row, renditions));
        }

        let next_page_token = if is_last_page {
            String::new()
        } else {
            (offset + assets.len()).to_string()
        };

        Ok(Response::new(ListMediaResponse {
            assets,
            next_page_token,
        }))
    }

    type StreamMediaStatusStream =
        Pin<Box<dyn Stream<Item = Result<MediaStatusUpdate, Status>> + Send + 'static>>;

    async fn stream_media_status(
        &self,
        request: Request<StreamMediaStatusRequest>,
    ) -> Result<Response<Self::StreamMediaStatusStream>, Status> {
        let asset_id = request.into_inner().asset_id;

        let row = db::fetch_asset_row(&self.state.db, &asset_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("asset not found"))?;

        let current_status = row
            .status
            .parse::<common::MediaStatus>()
            .expect("status column holds an invalid enum value");

        let initial = MediaStatusUpdate {
            asset_id: asset_id.clone(),
            status: proto_status(current_status),
            message: None,
            updated_at_ms: chrono::Utc::now().timestamp_millis(),
            progress_hint: None,
        };
        let already_terminal = is_terminal(current_status);

        let mut rx = self.state.status_tx.subscribe();

        let stream = async_stream::stream! {
            yield Ok(initial);

            if already_terminal {
                return;
            }

            loop {
                match rx.recv().await {
                    Ok(event) if event.asset_id == asset_id => {
                        let terminal = is_terminal(event.status);
                        yield Ok(to_proto_status_update(event));
                        if terminal {
                            break;
                        }
                    }
                    Ok(_) => continue,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }
}
