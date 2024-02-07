use crate::bitmap::{Bitmap, BitmapFormat};
use crate::error::Error;
use std::borrow::Cow;
use std::io::Read;
use swf::Color;

/// The format of image data in a DefineBitsJpeg2/3 tag.
/// Generally this will be JPEG, but according to SWF19, these tags can also contain PNG and GIF data.
/// SWF19 pp.138-139
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JpegTagFormat {
    Jpeg,
    Png,
    Gif,
    Unknown,
}

/// Determines the format of the image data in `data` from a DefineBitsJPEG2/3 tag.
pub fn determine_jpeg_tag_format(data: &[u8]) -> JpegTagFormat {
    match data {
        [0xff, 0xd8, ..] => JpegTagFormat::Jpeg,
        [0xff, 0xd9, 0xff, 0xd8, ..] => JpegTagFormat::Jpeg, // erroneous header in SWF
        [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, ..] => JpegTagFormat::Png,
        [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, ..] => JpegTagFormat::Gif,
        _ => JpegTagFormat::Unknown,
    }
}

/// Decodes bitmap data from a DefineBitsJPEG2/3 tag.
/// The data is returned with pre-multiplied alpha.
pub fn decode_define_bits_jpeg(data: &[u8], alpha_data: Option<&[u8]>) -> Result<Bitmap, Error> {
    let format = determine_jpeg_tag_format(data);
    if format != JpegTagFormat::Jpeg && alpha_data.is_some() {
        // Only DefineBitsJPEG3 with true JPEG data should have separate alpha data.
        tracing::warn!("DefineBitsJPEG contains non-JPEG data with alpha; probably incorrect")
    }
    match format {
        JpegTagFormat::Jpeg => decode_jpeg(data, alpha_data),
        JpegTagFormat::Png => decode_png(data),
        JpegTagFormat::Gif => decode_gif(data),
        JpegTagFormat::Unknown => Err(Error::UnknownType),
    }
}

pub fn decode_define_bits_jpeg_dimensions(data: &[u8]) -> Result<(u16, u16), Error> {
    let format = determine_jpeg_tag_format(data);
    match format {
        JpegTagFormat::Jpeg => decode_jpeg_dimensions(data),
        JpegTagFormat::Png => decode_png_dimensions(data),
        JpegTagFormat::Gif => decode_gif_dimensions(data),
        JpegTagFormat::Unknown => Err(Error::UnknownType),
    }
}

/// Glues the JPEG encoding tables from a JPEGTables SWF tag to the JPEG data
/// in a DefineBits tag, producing complete JPEG data suitable for a decoder.
pub fn glue_tables_to_jpeg<'a>(
    jpeg_data: &'a [u8],
    jpeg_tables: Option<&'a [u8]>,
) -> Cow<'a, [u8]> {
    if let Some(jpeg_tables) = jpeg_tables {
        if jpeg_tables.len() >= 2 {
            let mut full_jpeg = Vec::with_capacity(jpeg_tables.len() + jpeg_data.len());
            full_jpeg.extend_from_slice(&jpeg_tables[..jpeg_tables.len() - 2]);
            if jpeg_data.len() >= 2 {
                full_jpeg.extend_from_slice(&jpeg_data[2..]);
            }

            return full_jpeg.into();
        }
    }

    // No JPEG tables or not enough data; return JPEG data as is
    jpeg_data.into()
}

/// Removes potential invalid JPEG data from SWF DefineBitsJPEG tags.
/// These bytes need to be removed for the JPEG to decode properly.
pub fn remove_invalid_jpeg_data(data: &[u8]) -> Cow<[u8]> {
    // SWF19 errata p.138:
    // "Before version 8 of the SWF file format, SWF files could contain an erroneous header of 0xFF, 0xD9, 0xFF, 0xD8
    // before the JPEG SOI marker."
    // 0xFFD9FFD8 is a JPEG EOI+SOI marker pair. Contrary to the spec, this invalid marker sequence can actually appear
    // at any time before the 0xFFC0 SOF marker, not only at the beginning of the data. I believe this is a relic from
    // the SWF JPEGTables tag, which stores encoding tables separately from the DefineBits image data, encased in its
    // own SOI+EOI pair. When these data are glued together, an interior EOI+SOI sequence is produced. The Flash JPEG
    // decoder expects this pair and ignores it, despite standard JPEG decoders stopping at the EOI.
    // When DefineBitsJPEG2 etc. were introduced, the Flash encoders/decoders weren't properly adjusted, resulting in
    // this sequence persisting. Also, despite what the spec says, this doesn't appear to be version checked (e.g., a
    // v9 SWF can contain one of these malformed JPEGs and display correctly).
    // See https://github.com/ruffle-rs/ruffle/issues/8775 for various examples.

    // JPEG markers
    const SOF0: u8 = 0xC0; // Start of frame
    const RST0: u8 = 0xD0; // Restart (we shouldn't see this before SOS, but just in case)
    const RST7: u8 = 0xD7;
    const SOI: u8 = 0xD8; // Start of image
    const EOI: u8 = 0xD9; // End of image

    let mut data: Cow<[u8]> = if let Some(stripped) = data.strip_prefix(&[0xFF, EOI, 0xFF, SOI]) {
        // Common case: usually the sequence is at the beginning as the spec says, so adjust the slice to avoid a copy.
        stripped.into()
    } else {
        // Parse the JPEG markers searching for the 0xFFD9FFD8 marker sequence to splice out.
        // We only have to search up to the SOF0 marker.
        // This might be another case where eventually we want to write our own full JPEG decoder to match Flash's decoder.
        let mut jpeg_data = data;
        let mut pos = 0;
        loop {
            if jpeg_data.len() < 4 {
                // No invalid sequence found before SOF marker, return data as-is.
                break data.into();
            }
            let payload_len: usize = match &jpeg_data[..4] {
                [0xFF, EOI, 0xFF, SOI] => {
                    // Invalid EOI+SOI sequence found, splice it out.
                    let mut out_data = Vec::with_capacity(data.len() - 4);
                    out_data.extend_from_slice(&data[..pos]);
                    out_data.extend_from_slice(&data[pos + 4..]);
                    break out_data.into();
                }
                // EOI, SOI, RST markers do not include a size.
                [0xFF, EOI | SOI | RST0..=RST7, _, _] => 0,
                [0xFF, SOF0, _, _] => {
                    // No invalid sequence found before SOF marker, return data as-is.
                    break data.into();
                }
                // Other tags include a length.
                [0xFF, _, a, b] => u16::from_be_bytes([*a, *b]).into(),
                _ => {
                    // All JPEG markers should start with 0xFF.
                    // So this is either not a JPEG, or we screwed up parsing the markers. Bail out.
                    break data.into();
                }
            };
            // Advance to next JPEG marker.
            jpeg_data = jpeg_data.get(payload_len + 2..).unwrap_or_default();
            pos += payload_len + 2;
        }
    };

    // Some JPEGs are missing the final EOI marker (JPEG optimizers truncate it?)
    // Flash and most image decoders will still display these images, but jpeg-decoder errors.
    // Glue on an EOI marker if its not already there and hope for the best.
    if data.ends_with(&[0xFF, EOI]) {
        data
    } else {
        tracing::warn!("JPEG is missing EOI marker and may not decode properly");
        data.to_mut().extend_from_slice(&[0xFF, EOI]);
        data
    }
}

/// Some SWFs report unreasonable bitmap dimensions (#1191).
/// Fail before decoding such bitmaps to avoid panics.
fn validate_size(width: u16, height: u16) -> Result<(), Error> {
    const INVALID_SIZE: usize = 0x8000000; // 128MB

    let size = (width as usize).saturating_mul(height as usize);
    if size >= INVALID_SIZE {
        return Err(Error::TooLarge);
    }
    Ok(())
}

fn decode_jpeg_dimensions(jpeg_data: &[u8]) -> Result<(u16, u16), Error> {
    let jpeg_data = remove_invalid_jpeg_data(jpeg_data);

    let mut decoder = jpeg_decoder::Decoder::new(&jpeg_data[..]);
    decoder.read_info()?;
    let metadata = decoder
        .info()
        .expect("info() should always return Some if read_info returned Ok");
    validate_size(metadata.width, metadata.height)?;
    Ok((metadata.width, metadata.height))
}

/// Decodes a JPEG with optional alpha data.
/// The decoded bitmap will have pre-multiplied alpha.
fn decode_jpeg(jpeg_data: &[u8], alpha_data: Option<&[u8]>) -> Result<Bitmap, Error> {
    let jpeg_data = remove_invalid_jpeg_data(jpeg_data);

    let mut decoder = jpeg_decoder::Decoder::new(&jpeg_data[..]);
    decoder.read_info()?;
    let metadata = decoder
        .info()
        .expect("info() should always return Some if read_info returned Ok");
    validate_size(metadata.width, metadata.height)?;
    let decoded_data = decoder.decode()?;

    let decoded_data = match metadata.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => decoded_data,
        jpeg_decoder::PixelFormat::CMYK32 => decoded_data
            .chunks_exact(4)
            .flat_map(|cmyk| {
                let c = 255 - u16::from(cmyk[0]);
                let m = 255 - u16::from(cmyk[1]);
                let y = 255 - u16::from(cmyk[2]);
                let k = 255 - u16::from(cmyk[3]);

                let r = c * k / 255;
                let g = m * k / 255;
                let b = y * k / 255;
                [r as u8, g as u8, b as u8]
            })
            .collect(),
        jpeg_decoder::PixelFormat::L8 => decoded_data.iter().flat_map(|&c| [c, c, c]).collect(),
        jpeg_decoder::PixelFormat::L16 => {
            tracing::warn!("Unimplemented L16 JPEG pixel format");
            decoded_data
        }
    };

    // Decompress the alpha data (DEFLATE compression).
    if let Some(alpha_data) = alpha_data {
        let alpha_data = decompress_zlib(alpha_data)?;

        if alpha_data.len() == decoded_data.len() / 3 {
            let rgba = decoded_data
                .chunks_exact(3)
                .zip(alpha_data)
                .flat_map(|(rgb, a)| {
                    // The JPEG data should be premultiplied alpha, but it isn't in some incorrect
                    // SWFs (see #6893).
                    // This means 0% alpha pixels may have color and incorrectly show as visible.
                    // Flash Player clamps color to the alpha value to fix this case.
                    // Only applies to DefineBitsJPEG3; DefineBitsLossless does not seem to clamp.
                    let r = rgb[0].min(a);
                    let g = rgb[1].min(a);
                    let b = rgb[2].min(a);
                    [r, g, b, a]
                })
                .collect();
            return Ok(Bitmap::new(
                metadata.width.into(),
                metadata.height.into(),
                BitmapFormat::Rgba,
                rgba,
            ));
        } else {
            // Size isn't correct; fallback to RGB?
            tracing::error!("Size mismatch in DefineBitsJPEG3 alpha data");
        }
    }

    // No alpha.
    Ok(Bitmap::new(
        metadata.width.into(),
        metadata.height.into(),
        BitmapFormat::Rgb,
        decoded_data,
    ))
}

/// Decodes the bitmap data in DefineBitsLossless tag into RGBA.
/// DefineBitsLossless is Zlib encoded pixel data (similar to PNG), possibly
/// palletized.
pub fn decode_define_bits_lossless(swf_tag: &swf::DefineBitsLossless) -> Result<Bitmap, Error> {
    // Decompress the image data (DEFLATE compression).
    let mut decoded_data = decompress_zlib(&swf_tag.data)?;

    let has_alpha = swf_tag.version == 2;

    // Swizzle/de-palettize the bitmap.
    let out_data = match (swf_tag.version, swf_tag.format) {
        (1, swf::BitmapFormat::Rgb15) => {
            let padded_width = (swf_tag.width + 0b1) & !0b1;
            validate_size(swf_tag.width, swf_tag.height)?;
            let mut out_data =
                Vec::with_capacity(swf_tag.width as usize * swf_tag.height as usize * 4);
            let mut i = 0;
            for _ in 0..swf_tag.height {
                for _ in 0..swf_tag.width {
                    let compressed = u16::from_be_bytes([decoded_data[i], decoded_data[i + 1]]);
                    let rgb5_component = |shift: u16| {
                        let component = (compressed >> shift) & 0x1F;
                        ((component * 255 + 15) / 31) as u8
                    };
                    out_data.extend([
                        rgb5_component(10),
                        rgb5_component(5),
                        rgb5_component(0),
                        u8::MAX,
                    ]);
                    i += 2;
                }
                i += (padded_width - swf_tag.width) as usize * 2;
            }
            out_data
        }
        (1 | 2, swf::BitmapFormat::Rgb32) => {
            for rgba in decoded_data.chunks_exact_mut(4) {
                rgba.rotate_left(1);
                if !has_alpha {
                    rgba[3] = u8::MAX;
                }
            }
            decoded_data
        }
        (1 | 2, swf::BitmapFormat::ColorMap8 { num_colors }) => {
            let mut i = 0;
            let padded_width = (swf_tag.width + 0b11) & !0b11;

            let mut palette = Vec::with_capacity(num_colors as usize + 1);
            for _ in 0..=num_colors {
                let a = if has_alpha {
                    decoded_data[i + 3]
                } else {
                    u8::MAX
                };
                palette.push(Color {
                    r: decoded_data[i],
                    g: decoded_data[i + 1],
                    b: decoded_data[i + 2],
                    a,
                });
                i += if has_alpha { 4 } else { 3 };
            }

            validate_size(swf_tag.width, swf_tag.height)?;
            let mut out_data =
                Vec::with_capacity(swf_tag.width as usize * swf_tag.height as usize * 4);
            for _ in 0..swf_tag.height {
                for _ in 0..swf_tag.width {
                    let entry = decoded_data[i] as usize;
                    let color = palette.get(entry).unwrap_or(if has_alpha {
                        &Color::TRANSPARENT
                    } else {
                        &Color::BLACK
                    });
                    out_data.extend([color.r, color.g, color.b, color.a]);
                    i += 1;
                }
                i += (padded_width - swf_tag.width) as usize;
            }
            out_data
        }
        _ => {
            return Err(Error::UnsupportedLosslessFormat(
                swf_tag.version,
                swf_tag.format,
            ));
        }
    };

    Ok(Bitmap::new(
        swf_tag.width.into(),
        swf_tag.height.into(),
        BitmapFormat::Rgba,
        out_data,
    ))
}

fn decode_png_dimensions(data: &[u8]) -> Result<(u16, u16), Error> {
    use png::Transformations;

    let mut decoder = png::Decoder::new(data);
    // Normalize output to 8-bit grayscale or RGB.
    // Ideally we'd want to normalize to 8-bit RGB only, but seems like the `png` crate provides no such a feature.
    decoder.set_transformations(Transformations::normalize_to_color8());
    let reader = decoder.read_info()?;
    Ok((
        reader.info().width.try_into().expect("Invalid PNG width"),
        reader.info().height.try_into().expect("Invalid PNG height"),
    ))
}

/// Decodes the bitmap data in DefineBitsLossless tag into RGBA.
/// DefineBitsLossless is Zlib encoded pixel data (similar to PNG), possibly
/// palletized.
fn decode_png(data: &[u8]) -> Result<Bitmap, Error> {
    use png::{ColorType, Transformations};

    let mut decoder = png::Decoder::new(data);
    // Normalize output to 8-bit grayscale or RGB.
    // Ideally we'd want to normalize to 8-bit RGB only, but seems like the `png` crate provides no such a feature.
    decoder.set_transformations(Transformations::normalize_to_color8());
    let mut reader = decoder.read_info()?;

    let mut data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut data)?;

    let (format, data) = match info.color_type {
        ColorType::Rgb => (BitmapFormat::Rgb, data),
        ColorType::Rgba => {
            // In contrast to DefineBitsLossless tags, PNGs embedded in a DefineBitsJPEG tag will not have
            // premultiplied alpha and need to be converted before sending to the renderer.
            premultiply_alpha_rgba(&mut data);
            (BitmapFormat::Rgba, data)
        }
        ColorType::Grayscale => (
            BitmapFormat::Rgb,
            data.into_iter().flat_map(|v| [v, v, v]).collect(),
        ),
        ColorType::GrayscaleAlpha => {
            (
                BitmapFormat::Rgba,
                data.chunks_exact(2)
                    .flat_map(|pixel| {
                        // Pre-multiply alpha.
                        let a = pixel[1];
                        let v = (u16::from(pixel[0]) * u16::from(a) / 255) as u8;
                        [v, v, v, a]
                    })
                    .collect(),
            )
        }
        ColorType::Indexed => {
            // Shouldn't get here because of `normalize_to_color8` transformation above.
            unreachable!("Unexpected PNG ColorType::Indexed");
        }
    };

    Ok(Bitmap::new(info.width, info.height, format, data))
}

fn decode_gif_dimensions(data: &[u8]) -> Result<(u16, u16), Error> {
    let mut decode_options = gif::DecodeOptions::new();
    decode_options.set_color_output(gif::ColorOutput::RGBA);
    let reader = decode_options.read_info(data)?;
    Ok((reader.width(), reader.height()))
}

/// Decodes the bitmap data in DefineBitsLossless tag into RGBA.
/// DefineBitsLossless is Zlib encoded pixel data (similar to PNG), possibly
/// palletized.
fn decode_gif(data: &[u8]) -> Result<Bitmap, Error> {
    let mut decode_options = gif::DecodeOptions::new();
    decode_options.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decode_options.read_info(data)?;
    let frame = reader.read_next_frame()?.ok_or(Error::EmptyGif)?;
    // GIFs embedded in a DefineBitsJPEG tag will not have premultiplied alpha and need to be converted before sending to the renderer.
    let mut data = frame.buffer.to_vec();
    premultiply_alpha_rgba(&mut data);

    Ok(Bitmap::new(
        frame.width.into(),
        frame.height.into(),
        BitmapFormat::Rgba,
        data,
    ))
}

/// Converts standard RBGA to premultiplied alpha.
fn premultiply_alpha_rgba(rgba: &mut [u8]) {
    rgba.chunks_exact_mut(4).for_each(|rgba| {
        let a = f32::from(rgba[3]) / 255.0;
        rgba[0] = (f32::from(rgba[0]) * a) as u8;
        rgba[1] = (f32::from(rgba[1]) * a) as u8;
        rgba[2] = (f32::from(rgba[2]) * a) as u8;
    })
}

/// Converts premultiplied RBGA to unmultipled RGBA.
pub fn unmultiply_alpha_rgba(rgba: &mut [u8]) {
    rgba.chunks_exact_mut(4).for_each(|rgba| {
        let a = rgba[3];
        if a > 0 {
            let a = f32::from(a) / 255.0;
            rgba[0] = (f32::from(rgba[0]) / a) as u8;
            rgba[1] = (f32::from(rgba[1]) / a) as u8;
            rgba[2] = (f32::from(rgba[2]) / a) as u8;
        }
    })
}

/// Decodes zlib-compressed data.
fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut out_data = Vec::new();
    let mut decoder = flate2::bufread::ZlibDecoder::new(data);
    decoder
        .read_to_end(&mut out_data)
        .map_err(|_| Error::InvalidZlibCompression)?;
    out_data.shrink_to_fit();
    Ok(out_data)
}
