use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::error::WorkerError;

pub struct ProbeInfo {
    pub duration_seconds: f64,
}

pub async fn probe_duration(path: &Path) -> Result<ProbeInfo, WorkerError> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| WorkerError::Ffmpeg(format!("failed to spawn ffprobe: {e}")))?;

    if !output.status.success() {
        return Err(WorkerError::Ffmpeg(format!(
            "ffprobe exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let duration_seconds: f64 = stdout.trim().parse().map_err(|_| {
        WorkerError::Ffmpeg(format!("could not parse ffprobe duration output: {stdout:?}"))
    })?;

    Ok(ProbeInfo { duration_seconds })
}

pub async fn extract_frame_at(
    video_path: &Path,
    timestamp_seconds: f64,
    out_path: &Path,
) -> Result<(), WorkerError> {
    let output = Command::new("ffmpeg")
        .args(["-y", "-ss"])
        .arg(format!("{timestamp_seconds:.3}"))
        .arg("-i")
        .arg(video_path)
        .args(["-frames:v", "1", "-q:v", "2"])
        .arg(out_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| WorkerError::Ffmpeg(format!("failed to spawn ffmpeg: {e}")))?;

    if !output.status.success() {
        return Err(WorkerError::Ffmpeg(format!(
            "ffmpeg exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Spreads `count` sample points across the middle 80% of the clip,
/// avoiding the very first/last frames where fades and black frames
/// tend to live.
pub fn candidate_timestamps(duration_seconds: f64, count: usize) -> Vec<f64> {
    if duration_seconds <= 0.0 || count == 0 {
        return vec![0.0];
    }
    if count == 1 {
        return vec![duration_seconds * 0.5];
    }
    (0..count)
        .map(|i| {
            let frac = 0.1 + 0.8 * (i as f64) / ((count - 1) as f64);
            duration_seconds * frac
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreads_timestamps_across_middle_of_clip() {
        let ts = candidate_timestamps(100.0, 5);
        assert_eq!(ts.len(), 5);
        assert!((ts[0] - 10.0).abs() < 1e-9);
        assert!((ts[4] - 90.0).abs() < 1e-9);
    }

    #[test]
    fn falls_back_to_midpoint_for_a_single_candidate() {
        assert_eq!(candidate_timestamps(50.0, 1), vec![25.0]);
    }

    #[test]
    fn handles_zero_duration_without_panicking() {
        assert_eq!(candidate_timestamps(0.0, 5), vec![0.0]);
    }
}
