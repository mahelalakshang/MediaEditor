use image::DynamicImage;

/// Scores a candidate video frame for how good a thumbnail it would make;
/// higher is better. Kept behind a trait so a heavier ML-based scorer
/// (e.g. an ONNX/CLIP aesthetic model via the `ort` crate) can be swapped
/// in later without touching the frame-sampling/selection pipeline around
/// it — the honest "AI-era" story here is the pluggable boundary, not a
/// claim that classic CV heuristics are themselves a neural model.
pub trait FrameScorer: Send + Sync {
    fn score(&self, image: &DynamicImage) -> f64;
}

/// Scores frames using three classic, cheap-to-compute signals: sharpness
/// (average gradient magnitude — penalizes motion blur), contrast
/// (luma standard deviation — penalizes flat/washed-out frames), and
/// brightness balance (penalizes near-black or near-white frames, e.g.
/// during a fade). Weights were picked by hand for sane demo behavior,
/// not fit to a dataset.
pub struct HeuristicFrameScorer;

const SCORE_DIM: u32 = 160;
const SHARPNESS_WEIGHT: f64 = 0.55;
const CONTRAST_WEIGHT: f64 = 0.30;
const BRIGHTNESS_WEIGHT: f64 = 0.15;
// Empirical normalization ceilings so each raw signal maps into ~0..1
// before weighting; sharp/high-contrast natural photos land well under
// these, so scores rarely saturate at 1.0 in practice.
const SHARPNESS_NORMALIZER: f64 = 40.0;
const CONTRAST_NORMALIZER: f64 = 70.0;

impl FrameScorer for HeuristicFrameScorer {
    fn score(&self, image: &DynamicImage) -> f64 {
        let small = image.resize(SCORE_DIM, SCORE_DIM, image::imageops::FilterType::Triangle);
        let luma = small.to_luma8();
        let (w, h) = luma.dimensions();
        if w < 2 || h < 2 {
            return 0.0;
        }

        let pixels: Vec<f64> = luma.pixels().map(|p| p[0] as f64).collect();
        let n = pixels.len() as f64;
        let mean = pixels.iter().sum::<f64>() / n;
        let variance = pixels.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / n;
        let contrast = variance.sqrt();

        let mut gradient_sum = 0.0f64;
        let mut gradient_count = 0u32;
        for y in 0..h {
            for x in 0..w - 1 {
                let a = luma.get_pixel(x, y)[0] as f64;
                let b = luma.get_pixel(x + 1, y)[0] as f64;
                gradient_sum += (a - b).abs();
                gradient_count += 1;
            }
        }
        for x in 0..w {
            for y in 0..h - 1 {
                let a = luma.get_pixel(x, y)[0] as f64;
                let b = luma.get_pixel(x, y + 1)[0] as f64;
                gradient_sum += (a - b).abs();
                gradient_count += 1;
            }
        }
        let sharpness = gradient_sum / gradient_count.max(1) as f64;

        let brightness_balance = 1.0 - ((mean - 127.5).abs() / 127.5);

        let norm_sharpness = (sharpness / SHARPNESS_NORMALIZER).min(1.0);
        let norm_contrast = (contrast / CONTRAST_NORMALIZER).min(1.0);

        SHARPNESS_WEIGHT * norm_sharpness
            + CONTRAST_WEIGHT * norm_contrast
            + BRIGHTNESS_WEIGHT * brightness_balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    fn solid_gray_image(dim: u32, gray: u8) -> DynamicImage {
        DynamicImage::ImageRgb8(ImageBuffer::from_pixel(dim, dim, Rgb([gray, gray, gray])))
    }

    fn checkerboard_image(dim: u32) -> DynamicImage {
        let buf = ImageBuffer::from_fn(dim, dim, |x, y| {
            if (x / 4 + y / 4) % 2 == 0 {
                Rgb([20u8, 20, 20])
            } else {
                Rgb([235u8, 235, 235])
            }
        });
        DynamicImage::ImageRgb8(buf)
    }

    #[test]
    fn flat_frame_scores_lower_than_a_sharp_high_contrast_frame() {
        let scorer = HeuristicFrameScorer;
        let flat = scorer.score(&solid_gray_image(200, 128));
        let sharp = scorer.score(&checkerboard_image(200));
        assert!(
            sharp > flat,
            "expected checkerboard ({sharp}) to outscore flat gray ({flat})"
        );
    }

    #[test]
    fn near_black_frame_is_penalized_relative_to_mid_brightness() {
        let scorer = HeuristicFrameScorer;
        let near_black = scorer.score(&solid_gray_image(200, 5));
        let mid = scorer.score(&solid_gray_image(200, 128));
        assert!(
            mid >= near_black,
            "expected mid-brightness ({mid}) >= near-black ({near_black})"
        );
    }

    #[test]
    fn scores_are_finite_and_non_negative() {
        let scorer = HeuristicFrameScorer;
        let score = scorer.score(&checkerboard_image(64));
        assert!(score.is_finite());
        assert!(score >= 0.0);
    }
}
