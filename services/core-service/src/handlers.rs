use std::sync::Arc;

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use common::{MediaKind, MediaStatus, MediaUploaded, TOPIC_MEDIA_UPLOADED};
use uuid::Uuid;

use crate::db;
use crate::error::AppError;
use crate::models::MediaAssetDto;
use crate::state::AppState;

pub async fn health(State(state): State<Arc<AppState>>) -> Result<StatusCode, AppError> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(StatusCode::OK)
}

fn kind_and_extension_for_mime(mime_type: &str) -> Option<(MediaKind, &'static str)> {
    match mime_type {
        "image/png" => Some((MediaKind::Image, "png")),
        "image/gif" => Some((MediaKind::Image, "gif")),
        "image/webp" => Some((MediaKind::Image, "webp")),
        "image/jpeg" => Some((MediaKind::Image, "jpg")),
        "video/mp4" => Some((MediaKind::Video, "mp4")),
        "video/webm" => Some((MediaKind::Video, "webm")),
        "video/quicktime" => Some((MediaKind::Video, "mov")),
        other if other.starts_with("image/") => Some((MediaKind::Image, "jpg")),
        other if other.starts_with("video/") => Some((MediaKind::Video, "mp4")),
        _ => None,
    }
}

pub async fn upload(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MediaAssetDto>), AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut original_filename = "upload".to_string();
    let mut mime_type = "application/octet-stream".to_string();

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            if let Some(name) = field.file_name() {
                original_filename = name.to_string();
            }
            if let Some(ct) = field.content_type() {
                mime_type = ct.to_string();
            }
            file_bytes = Some(field.bytes().await?.to_vec());
        }
    }

    let bytes = file_bytes.ok_or_else(|| AppError::BadRequest("missing 'file' field".into()))?;

    let (kind, ext) = kind_and_extension_for_mime(&mime_type)
        .ok_or_else(|| AppError::UnsupportedMediaType(mime_type.clone()))?;

    let asset_id = Uuid::new_v4().to_string();
    let asset_dir = state.media_dir.join(&asset_id);
    tokio::fs::create_dir_all(&asset_dir).await?;

    let original_path = asset_dir.join(format!("original.{ext}"));
    tokio::fs::write(&original_path, &bytes).await?;

    let now = chrono::Utc::now();

    db::insert_asset(
        &state.db,
        &asset_id,
        &original_filename,
        &mime_type,
        kind,
        MediaStatus::Uploaded,
        &original_path.to_string_lossy(),
        &now.to_rfc3339(),
    )
    .await?;

    common::kafka::publish(
        &state.kafka_producer,
        TOPIC_MEDIA_UPLOADED,
        &asset_id,
        &MediaUploaded {
            asset_id: asset_id.clone(),
            kind,
            original_filename,
            mime_type,
            storage_path: original_path.to_string_lossy().to_string(),
            uploaded_at_ms: now.timestamp_millis(),
        },
    )
    .await?;

    let asset_row = db::fetch_asset_row(&state.db, &asset_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let renditions = db::fetch_renditions(&state.db, &asset_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(MediaAssetDto::from_row(asset_row, renditions)),
    ))
}

pub async fn list_assets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<MediaAssetDto>>, AppError> {
    let rows = db::list_asset_rows(&state.db).await?;
    let mut dtos = Vec::with_capacity(rows.len());
    for row in rows {
        let renditions = db::fetch_renditions(&state.db, &row.id).await?;
        dtos.push(MediaAssetDto::from_row(row, renditions));
    }
    Ok(Json(dtos))
}

pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<MediaAssetDto>, AppError> {
    let row = db::fetch_asset_row(&state.db, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    let renditions = db::fetch_renditions(&state.db, &id).await?;
    Ok(Json(MediaAssetDto::from_row(row, renditions)))
}
