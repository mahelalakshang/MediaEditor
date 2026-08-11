#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error("ffmpeg/ffprobe failed: {0}")]
    Ffmpeg(String),
}
