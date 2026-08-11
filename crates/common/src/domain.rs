use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Shared domain enums. Stored in SQLite as TEXT (via Display/FromStr) and
/// exchanged over JSON/gRPC as SCREAMING_SNAKE_CASE strings, so all three
/// representations stay in lockstep with a single source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaKind {
    Image,
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaStatus {
    Uploaded,
    Processing,
    Processed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RenditionType {
    Thumbnail,
    CandidateFrame,
    PreviewClip,
}

#[derive(Debug)]
pub struct ParseEnumError(pub String);

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unrecognized value: {}", self.0)
    }
}

impl std::error::Error for ParseEnumError {}

impl fmt::Display for MediaKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MediaKind::Image => "IMAGE",
            MediaKind::Video => "VIDEO",
        })
    }
}

impl FromStr for MediaKind {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IMAGE" => Ok(MediaKind::Image),
            "VIDEO" => Ok(MediaKind::Video),
            other => Err(ParseEnumError(other.to_string())),
        }
    }
}

impl fmt::Display for MediaStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MediaStatus::Uploaded => "UPLOADED",
            MediaStatus::Processing => "PROCESSING",
            MediaStatus::Processed => "PROCESSED",
            MediaStatus::Failed => "FAILED",
        })
    }
}

impl FromStr for MediaStatus {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UPLOADED" => Ok(MediaStatus::Uploaded),
            "PROCESSING" => Ok(MediaStatus::Processing),
            "PROCESSED" => Ok(MediaStatus::Processed),
            "FAILED" => Ok(MediaStatus::Failed),
            other => Err(ParseEnumError(other.to_string())),
        }
    }
}

impl fmt::Display for RenditionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RenditionType::Thumbnail => "THUMBNAIL",
            RenditionType::CandidateFrame => "CANDIDATE_FRAME",
            RenditionType::PreviewClip => "PREVIEW_CLIP",
        })
    }
}

impl FromStr for RenditionType {
    type Err = ParseEnumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "THUMBNAIL" => Ok(RenditionType::Thumbnail),
            "CANDIDATE_FRAME" => Ok(RenditionType::CandidateFrame),
            "PREVIEW_CLIP" => Ok(RenditionType::PreviewClip),
            other => Err(ParseEnumError(other.to_string())),
        }
    }
}
