use std::str::FromStr;

use common::{MediaKind, MediaStatus, RenditionType};
use serde::Serialize;
use sqlx::FromRow;

/// Raw row shapes as stored in SQLite (enums as TEXT columns).
#[derive(Debug, FromRow)]
pub struct MediaAssetRow {
    pub id: String,
    pub original_filename: String,
    pub mime_type: String,
    pub kind: String,
    pub status: String,
    pub storage_path: String,
    pub created_at: String,
}

#[derive(Debug, FromRow)]
pub struct MediaRenditionRow {
    pub id: String,
    pub asset_id: String,
    pub rendition_type: String,
    pub path: String,
    pub width: i64,
    pub height: i64,
    pub score: Option<f64>,
    pub is_selected: bool,
    pub timestamp_seconds: Option<f64>,
    pub created_at: String,
}

/// JSON API shapes returned to clients. camelCase to match the shape the
/// BFF's gRPC path already produces (proto-loader's default JS field
/// naming) — the BFF's upload proxy forwards this response body through
/// the same DTO mapper it uses for gRPC responses, so the two must agree.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenditionDto {
    pub id: String,
    pub rendition_type: RenditionType,
    pub path: String,
    pub width: i64,
    pub height: i64,
    pub score: Option<f64>,
    pub is_selected: bool,
    pub timestamp_seconds: Option<f64>,
}

impl From<MediaRenditionRow> for RenditionDto {
    fn from(row: MediaRenditionRow) -> Self {
        RenditionDto {
            id: row.id,
            rendition_type: RenditionType::from_str(&row.rendition_type)
                .expect("rendition_type column holds an invalid enum value"),
            path: row.path,
            width: row.width,
            height: row.height,
            score: row.score,
            is_selected: row.is_selected,
            timestamp_seconds: row.timestamp_seconds,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAssetDto {
    pub id: String,
    pub original_filename: String,
    pub mime_type: String,
    pub kind: MediaKind,
    pub status: MediaStatus,
    pub created_at: String,
    pub renditions: Vec<RenditionDto>,
}

impl MediaAssetDto {
    pub fn from_row(row: MediaAssetRow, renditions: Vec<MediaRenditionRow>) -> Self {
        MediaAssetDto {
            id: row.id,
            original_filename: row.original_filename,
            mime_type: row.mime_type,
            kind: MediaKind::from_str(&row.kind).expect("kind column holds an invalid enum value"),
            status: MediaStatus::from_str(&row.status)
                .expect("status column holds an invalid enum value"),
            created_at: row.created_at,
            renditions: renditions.into_iter().map(RenditionDto::from).collect(),
        }
    }
}
