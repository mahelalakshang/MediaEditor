use std::path::{Path, PathBuf};
use std::time::Instant;

use common::{
    MediaKind, MediaProcessed, MediaProcessingFailed, MediaStatus, MediaThumbnailReady,
    MediaUploaded, ProcessingStage, RenditionRef, RenditionType, TOPIC_MEDIA_PROCESSED,
    TOPIC_MEDIA_PROCESSING_DLQ, TOPIC_MEDIA_THUMBNAIL_READY,
};
use image::DynamicImage;
use uuid::Uuid;

use crate::db;
use crate::error::WorkerError;
use crate::frame_scorer::{FrameScorer, HeuristicFrameScorer};
use crate::image_processing::generate_thumbnail;
use crate::state::AppState;
use crate::video;

const CANDIDATE_FRAME_COUNT: usize = 5;
const MAX_THUMBNAIL_DIM: u32 = 320;

type StageResult<T> = Result<T, (ProcessingStage, WorkerError)>;

pub async fn process(state: &AppState, event: MediaUploaded) {
    let started = Instant::now();

    if let Err(err) = db::set_status(&state.db, &event.asset_id, MediaStatus::Processing).await {
        tracing::error!(asset_id = %event.asset_id, error = %err, "failed to mark asset as processing");
        return;
    }

    let result = match event.kind {
        MediaKind::Image => process_image(state, &event).await,
        MediaKind::Video => process_video(state, &event).await,
    };

    match result {
        Ok(renditions) => {
            if let Err(err) =
                db::set_status(&state.db, &event.asset_id, MediaStatus::Processed).await
            {
                tracing::error!(asset_id = %event.asset_id, error = %err, "failed to mark asset as processed");
                return;
            }
            if let Err(err) = common::kafka::publish(
                &state.kafka_producer,
                TOPIC_MEDIA_PROCESSED,
                &event.asset_id,
                &MediaProcessed {
                    asset_id: event.asset_id.clone(),
                    renditions,
                    processing_duration_ms: started.elapsed().as_millis() as i64,
                },
            )
            .await
            {
                tracing::error!(asset_id = %event.asset_id, error = %err, "failed to publish media.processed");
            }
            tracing::info!(asset_id = %event.asset_id, "processing complete");
        }
        Err((stage, err)) => {
            tracing::error!(asset_id = %event.asset_id, stage = ?stage, error = %err, "processing failed");
            let _ = db::set_status(&state.db, &event.asset_id, MediaStatus::Failed).await;
            if let Err(publish_err) = common::kafka::publish(
                &state.kafka_producer,
                TOPIC_MEDIA_PROCESSING_DLQ,
                &event.asset_id,
                &MediaProcessingFailed {
                    asset_id: event.asset_id.clone(),
                    stage,
                    error_message: err.to_string(),
                    retry_count: 0,
                },
            )
            .await
            {
                tracing::error!(asset_id = %event.asset_id, error = %publish_err, "failed to publish to DLQ");
            }
        }
    }
}

fn asset_dir_for(storage_path: &str) -> PathBuf {
    Path::new(storage_path)
        .parent()
        .expect("storage_path always has a parent asset dir")
        .to_path_buf()
}

/// Renditions are persisted with a portable, forward-slash relative path
/// (e.g. "{asset_id}/thumbnail.jpg") rather than the OS-specific
/// filesystem path used for actual file IO — core-service serves
/// `media_dir` at `/media`, so this string doubles as the URL path the
/// frontend fetches the image from.
fn public_rendition_path(asset_id: &str, filename: &str) -> String {
    format!("{asset_id}/{filename}")
}

async fn process_image(state: &AppState, event: &MediaUploaded) -> StageResult<Vec<RenditionRef>> {
    let asset_dir = asset_dir_for(&event.storage_path);

    let original_bytes = tokio::fs::read(&event.storage_path)
        .await
        .map_err(|e| (ProcessingStage::Probe, WorkerError::from(e)))?;

    let thumb = generate_thumbnail(&original_bytes, MAX_THUMBNAIL_DIM)
        .map_err(|e| (ProcessingStage::Encode, WorkerError::from(e)))?;

    let thumb_path = asset_dir.join("thumbnail.jpg");
    tokio::fs::write(&thumb_path, &thumb.bytes)
        .await
        .map_err(|e| (ProcessingStage::Write, WorkerError::from(e)))?;

    let rendition_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let public_path = public_rendition_path(&event.asset_id, "thumbnail.jpg");
    db::insert_rendition(
        &state.db,
        &rendition_id,
        &event.asset_id,
        RenditionType::Thumbnail,
        &public_path,
        thumb.width as i64,
        thumb.height as i64,
        None,
        true,
        None,
        &now,
    )
    .await
    .map_err(|e| (ProcessingStage::Write, WorkerError::from(e)))?;

    if let Err(err) = common::kafka::publish(
        &state.kafka_producer,
        TOPIC_MEDIA_THUMBNAIL_READY,
        &event.asset_id,
        &MediaThumbnailReady {
            asset_id: event.asset_id.clone(),
            rendition_id: rendition_id.clone(),
            thumbnail_path: public_path.clone(),
            width: thumb.width,
            height: thumb.height,
            selected_frame_timestamp_seconds: None,
            score: None,
            candidate_count: None,
        },
    )
    .await
    {
        tracing::error!(asset_id = %event.asset_id, error = %err, "failed to publish media.thumbnail.ready");
    }

    Ok(vec![RenditionRef {
        id: rendition_id,
        rendition_type: RenditionType::Thumbnail,
        path: public_path,
    }])
}

struct Candidate {
    timestamp: f64,
    score: f64,
    image: DynamicImage,
}

async fn process_video(state: &AppState, event: &MediaUploaded) -> StageResult<Vec<RenditionRef>> {
    let asset_dir = asset_dir_for(&event.storage_path);
    let video_path = Path::new(&event.storage_path);

    let probe = video::probe_duration(video_path)
        .await
        .map_err(|e| (ProcessingStage::Probe, e))?;

    let timestamps = video::candidate_timestamps(probe.duration_seconds, CANDIDATE_FRAME_COUNT);

    let candidates_dir = asset_dir.join("candidates");
    tokio::fs::create_dir_all(&candidates_dir)
        .await
        .map_err(|e| (ProcessingStage::ExtractFrames, WorkerError::from(e)))?;

    let scorer = HeuristicFrameScorer;
    let mut candidates = Vec::with_capacity(timestamps.len());

    for (i, timestamp) in timestamps.into_iter().enumerate() {
        let frame_path = candidates_dir.join(format!("frame_{i}.jpg"));
        video::extract_frame_at(video_path, timestamp, &frame_path)
            .await
            .map_err(|e| (ProcessingStage::ExtractFrames, e))?;

        let bytes = tokio::fs::read(&frame_path)
            .await
            .map_err(|e| (ProcessingStage::Score, WorkerError::from(e)))?;
        let image = image::load_from_memory(&bytes)
            .map_err(|e| (ProcessingStage::Score, WorkerError::from(e)))?;
        let score = scorer.score(&image);

        candidates.push(Candidate {
            timestamp,
            score,
            image,
        });
    }

    let best_idx = candidates
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.score.total_cmp(&b.score))
        .map(|(i, _)| i)
        .expect("at least one candidate frame was sampled");

    let now = chrono::Utc::now().to_rfc3339();
    let mut renditions = Vec::with_capacity(candidates.len() + 1);

    for (i, candidate) in candidates.iter().enumerate() {
        let rendition_id = Uuid::new_v4().to_string();
        let public_path = public_rendition_path(&event.asset_id, &format!("candidates/frame_{i}.jpg"));
        db::insert_rendition(
            &state.db,
            &rendition_id,
            &event.asset_id,
            RenditionType::CandidateFrame,
            &public_path,
            candidate.image.width() as i64,
            candidate.image.height() as i64,
            Some(candidate.score),
            i == best_idx,
            Some(candidate.timestamp),
            &now,
        )
        .await
        .map_err(|e| (ProcessingStage::Write, WorkerError::from(e)))?;
        renditions.push(RenditionRef {
            id: rendition_id,
            rendition_type: RenditionType::CandidateFrame,
            path: public_path,
        });
    }

    let best = &candidates[best_idx];
    let thumb_path = asset_dir.join("thumbnail.jpg");
    let thumb_public_path = public_rendition_path(&event.asset_id, "thumbnail.jpg");
    let mut buf = std::io::Cursor::new(Vec::new());
    best.image
        .to_rgb8()
        .write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| (ProcessingStage::Encode, WorkerError::from(e)))?;
    tokio::fs::write(&thumb_path, buf.into_inner())
        .await
        .map_err(|e| (ProcessingStage::Write, WorkerError::from(e)))?;

    let thumb_rendition_id = Uuid::new_v4().to_string();
    db::insert_rendition(
        &state.db,
        &thumb_rendition_id,
        &event.asset_id,
        RenditionType::Thumbnail,
        &thumb_public_path,
        best.image.width() as i64,
        best.image.height() as i64,
        Some(best.score),
        true,
        Some(best.timestamp),
        &now,
    )
    .await
    .map_err(|e| (ProcessingStage::Write, WorkerError::from(e)))?;

    if let Err(err) = common::kafka::publish(
        &state.kafka_producer,
        TOPIC_MEDIA_THUMBNAIL_READY,
        &event.asset_id,
        &MediaThumbnailReady {
            asset_id: event.asset_id.clone(),
            rendition_id: thumb_rendition_id.clone(),
            thumbnail_path: thumb_public_path.clone(),
            width: best.image.width(),
            height: best.image.height(),
            selected_frame_timestamp_seconds: Some(best.timestamp),
            score: Some(best.score),
            candidate_count: Some(candidates.len() as u32),
        },
    )
    .await
    {
        tracing::error!(asset_id = %event.asset_id, error = %err, "failed to publish media.thumbnail.ready");
    }

    renditions.push(RenditionRef {
        id: thumb_rendition_id,
        rendition_type: RenditionType::Thumbnail,
        path: thumb_public_path,
    });

    Ok(renditions)
}
