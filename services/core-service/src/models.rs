use std::str::FromStr;

use common::{MediaKind, MediaStatus, RenditionType};
use serde::Serialize;
use sqlx::FromRow;

/// Single source of truth for the mime-type -> file-extension mapping,
/// shared by the upload handler (validating + naming new files) and the
/// DTO layer (deriving the original file's public download path).
pub fn extension_for_mime(mime_type: &str) -> &'static str {
    match mime_type {
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/jpeg" => "jpg",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        other if other.starts_with("image/") => "jpg",
        other if other.starts_with("video/") => "mp4",
        _ => "bin",
    }
}

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
    /// Portable relative path (e.g. "{id}/original.png") the frontend
    /// resolves to a download URL via `/media/{original_path}` — see
    /// media-worker's `public_rendition_path` for the same pattern
    /// applied to renditions.
    pub original_path: String,
}

impl MediaAssetDto {
    pub fn from_row(row: MediaAssetRow, renditions: Vec<MediaRenditionRow>) -> Self {
        let original_path = format!("{}/original.{}", row.id, extension_for_mime(&row.mime_type));
        MediaAssetDto {
            id: row.id,
            original_filename: row.original_filename,
            mime_type: row.mime_type,
            kind: MediaKind::from_str(&row.kind).expect("kind column holds an invalid enum value"),
            status: MediaStatus::from_str(&row.status)
                .expect("status column holds an invalid enum value"),
            created_at: row.created_at,
            renditions: renditions.into_iter().map(RenditionDto::from).collect(),
            original_path,
        }
    }
}
