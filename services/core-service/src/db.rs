use common::{MediaKind, MediaStatus};
use sqlx::SqlitePool;

use crate::models::{MediaAssetRow, MediaRenditionRow};

#[allow(clippy::too_many_arguments)]
pub async fn insert_asset(
    pool: &SqlitePool,
    id: &str,
    original_filename: &str,
    mime_type: &str,
    kind: MediaKind,
    status: MediaStatus,
    storage_path: &str,
    created_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO media_assets (id, original_filename, mime_type, kind, status, storage_path, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(original_filename)
    .bind(mime_type)
    .bind(kind.to_string())
    .bind(status.to_string())
    .bind(storage_path)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_asset_row(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<MediaAssetRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaAssetRow>("SELECT * FROM media_assets WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn fetch_renditions(
    pool: &SqlitePool,
    asset_id: &str,
) -> Result<Vec<MediaRenditionRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRenditionRow>(
        "SELECT * FROM media_renditions WHERE asset_id = ? ORDER BY created_at ASC",
    )
    .bind(asset_id)
    .fetch_all(pool)
    .await
}

pub async fn list_asset_rows(pool: &SqlitePool) -> Result<Vec<MediaAssetRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaAssetRow>("SELECT * FROM media_assets ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}
