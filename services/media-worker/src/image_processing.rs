use std::io::Cursor;

pub struct Thumbnail {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Decodes an image and produces a JPEG thumbnail that fits within
/// `max_dim` x `max_dim`, preserving aspect ratio.
pub fn generate_thumbnail(
    original_bytes: &[u8],
    max_dim: u32,
) -> Result<Thumbnail, image::ImageError> {
    let img = image::load_from_memory(original_bytes)?;
    let thumb = img.thumbnail(max_dim, max_dim);

    let mut buf = Cursor::new(Vec::new());
    thumb.to_rgb8().write_to(&mut buf, image::ImageFormat::Jpeg)?;

    Ok(Thumbnail {
        bytes: buf.into_inner(),
        width: thumb.width(),
        height: thumb.height(),
    })
}
