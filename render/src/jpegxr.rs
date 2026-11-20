//! JPEG-XR decoding, backed by the pure-Rust `jxr` crate.
//!
//! Shared by `DefineBitsJPEG`/`Loader` image decoding and by ATF texture
//! uploads in Stage3D.

use crate::error::Error;
use jxr::{
    AlphaMode, ChannelLayout, ColorFormat, DecodeRequest, DecodedImage, DecodedSamples, JxrView,
    PixelFormat, PlaneDescriptor, SampleFormat,
};

/// A decoded JPEG-XR image.
pub struct JpegXrImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Returns the dimensions of a JPEG-XR image without decoding its pixels.
pub fn dimensions(data: &[u8]) -> Result<(u32, u32), Error> {
    Ok(JxrView::parse(data)?.info().dimensions())
}

/// Returns `true` if the image is coded as a single 8-bit grayscale plane
/// with no alpha.
pub fn is_gray8(data: &[u8]) -> Result<bool, Error> {
    let view = JxrView::parse(data)?;
    let info = view.info();
    Ok(info.primary.color_format == ColorFormat::Luma
        && info.primary.sample_format == SampleFormat::Unsigned { bits: 8 }
        && info.alpha_mode == AlphaMode::None)
}

/// Decodes any grayscale or RGB JPEG-XR image (with or without alpha) to
/// straight (non-premultiplied) 8-bit RGBA.
///
/// Images without alpha get an opaque alpha channel, and grayscale is
/// replicated into the R, G and B channels. Samples wider than 8 bits are
/// rounded down to 8 bits.
pub fn decode_rgba8(data: &[u8]) -> Result<JpegXrImage, Error> {
    let view = JxrView::parse(data)?;
    let info = view.info();
    let (width, height) = info.dimensions();

    let gray = info.primary.color_format == ColorFormat::Luma;
    let alpha = info.alpha_mode != AlphaMode::None;
    let (layout, channels) = match (gray, alpha) {
        (true, false) => (ChannelLayout::Luma, 1),
        (true, true) => (ChannelLayout::LumaAlpha, 2),
        (false, false) => (ChannelLayout::Rgb, 3),
        (false, true) => (ChannelLayout::Rgba, 4),
    };
    let wide = matches!(info.primary.sample_format, SampleFormat::Unsigned { bits } if bits > 8);
    let format = if wide {
        PixelFormat::U16(layout)
    } else {
        PixelFormat::U8(layout)
    };

    let image = view.decoder().decode(&DecodeRequest::new(format))?;
    let plane = single_plane(&image)?;
    let samples = match &image.samples {
        DecodedSamples::U8(samples) => tightly_packed(samples, plane, 1, channels)?,
        DecodedSamples::U16(samples) => tightly_packed(samples, plane, 2, channels)?
            .into_iter()
            // Same rounding as `image`'s 16-bit to 8-bit conversion.
            .map(|sample| ((u32::from(sample) + 128) / 257) as u8)
            .collect(),
        _ => return Err(unexpected_output(&image)),
    };

    let data = match channels {
        1 => samples.iter().flat_map(|&l| [l, l, l, 255]).collect(),
        2 => samples
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&[l, a]| [l, l, l, a])
            .collect(),
        3 => samples
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|&[r, g, b]| [r, g, b, 255])
            .collect(),
        _ => samples,
    };

    Ok(JpegXrImage {
        width,
        height,
        data,
    })
}

/// Decodes a JPEG-XR image to packed 16-bit RGB565 pixels, as little-endian
/// bytes (two bytes per pixel).
pub fn decode_rgb565(data: &[u8]) -> Result<JpegXrImage, Error> {
    let view = JxrView::parse(data)?;
    let (width, height) = view.info().dimensions();
    let image = view
        .decoder()
        .decode(&DecodeRequest::new(PixelFormat::Rgb565))?;
    let plane = single_plane(&image)?;
    let DecodedSamples::Rgb565(samples) = &image.samples else {
        return Err(unexpected_output(&image));
    };
    let data = tightly_packed(samples, plane, 2, 1)?
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect();

    Ok(JpegXrImage {
        width,
        height,
        data,
    })
}

fn single_plane(image: &DecodedImage) -> Result<&PlaneDescriptor, Error> {
    match image.planes.as_slice() {
        [plane] => Ok(plane),
        _ => Err(unexpected_output(image)),
    }
}

/// Copies the rows of an interleaved plane into a buffer without row padding.
fn tightly_packed<T: Copy>(
    samples: &[T],
    plane: &PlaneDescriptor,
    sample_size: usize,
    samples_per_pixel: usize,
) -> Result<Vec<T>, Error> {
    let row_len = plane.width as usize * samples_per_pixel;
    let stride = plane.row_stride_bytes / sample_size;
    let offset = plane.byte_offset / sample_size;
    let mut packed = Vec::with_capacity(row_len * plane.height as usize);
    for y in 0..plane.height as usize {
        let start = offset + y * stride;
        let row = samples
            .get(start..start + row_len)
            .ok_or_else(|| Error::Unimplemented("JPEG-XR plane out of bounds".into()))?;
        packed.extend_from_slice(row);
    }
    Ok(packed)
}

fn unexpected_output(image: &DecodedImage) -> Error {
    Error::Unimplemented(format!("Unexpected JPEG-XR decoder output: {:?}", image.format).into())
}
