use common::{MediaStatus, RenditionType};
use sqlx::SqlitePool;

pub async fn set_status(
    pool: &SqlitePool,
    asset_id: &str,
    status: MediaStatus,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE media_assets SET status = ? WHERE id = ?")
        .bind(status.to_string())
        .bind(asset_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_rendition(
    pool: &SqlitePool,
    id: &str,
    asset_id: &str,
    rendition_type: RenditionType,
    path: &str,
    width: i64,
    height: i64,
    score: Option<f64>,
    is_selected: bool,
    timestamp_seconds: Option<f64>,
    created_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO media_renditions
            (id, asset_id, rendition_type, path, width, height, score, is_selected, timestamp_seconds, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(asset_id)
    .bind(rendition_type.to_string())
    .bind(path)
    .bind(width)
    .bind(height)
    .bind(score)
    .bind(is_selected)
    .bind(timestamp_seconds)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}
