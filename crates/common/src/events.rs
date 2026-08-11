use serde::{Deserialize, Serialize};

use crate::{MediaKind, RenditionType};

pub const TOPIC_MEDIA_UPLOADED: &str = "media.uploaded";
pub const TOPIC_MEDIA_THUMBNAIL_READY: &str = "media.thumbnail.ready";
pub const TOPIC_MEDIA_PROCESSED: &str = "media.processed";
pub const TOPIC_MEDIA_PROCESSING_DLQ: &str = "media.processing.dlq";

pub const KAFKA_CONSUMER_GROUP_MEDIA_WORKER: &str = "media-worker";

/// Published by core-service right after an upload is persisted to disk/DB.
/// This is the handoff point: core-service's job ends here, media-worker
/// picks up all further processing asynchronously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploaded {
    pub asset_id: String,
    pub kind: MediaKind,
    pub original_filename: String,
    pub mime_type: String,
    pub storage_path: String,
    pub uploaded_at_ms: i64,
}

/// Published by media-worker once the best thumbnail has been selected
/// (immediately, for images; after frame sampling + scoring, for video).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaThumbnailReady {
    pub asset_id: String,
    pub rendition_id: String,
    pub thumbnail_path: String,
    pub width: u32,
    pub height: u32,
    pub selected_frame_timestamp_seconds: Option<f64>,
    pub score: Option<f64>,
    pub candidate_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenditionRef {
    pub id: String,
    pub rendition_type: RenditionType,
    pub path: String,
}

/// Published by media-worker once an asset has reached a terminal
/// PROCESSED state, listing every rendition produced along the way.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProcessed {
    pub asset_id: String,
    pub renditions: Vec<RenditionRef>,
    pub processing_duration_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessingStage {
    Probe,
    ExtractFrames,
    Score,
    Encode,
    Write,
}

/// Published to the DLQ topic when processing fails at any stage. There is
/// no automatic reprocessing consumer in this build (see plan's "Explicit
/// Cuts") — this is the documented extension point for one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProcessingFailed {
    pub asset_id: String,
    pub stage: ProcessingStage,
    pub error_message: String,
    pub retry_count: u32,
}
