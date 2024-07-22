use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::TextureObject;
use crate::avm2::Activation;
use crate::avm2::Error;
use crate::avm2::Object;
use crate::avm2::TObject;
use crate::avm2_stub_method;
use ruffle_render::atf::ATFTexture;
use ruffle_render::atf::ATFTextureData;
use std::io::Cursor;

use jpegxr::PixelFormat;
use std::io::Read;
use std::io::Seek;

pub fn do_compressed_upload<'gc>(
    activation: &mut Activation<'_, 'gc>,
    texture: TextureObject<'gc>,
    data: Object<'gc>,
    byte_array_offset: usize,
    is_cube: bool,
) -> Result<(), Error<'gc>> {
    let bytes = data.as_bytearray().unwrap();
    let raw_atf = &ByteArrayStorage::bytes(&bytes)[byte_array_offset..];

    let atf_texture = ATFTexture::from_bytes(raw_atf).expect("Failed to parse ATF texture");

    if is_cube != atf_texture.cubemap {
        return Err("Stage3D Texture and ATF Texture must both be cube/non-cube".into());
    }

    if atf_texture.width != texture.handle().width()
        || atf_texture.height != texture.handle().height()
    {
        return Err("ATF texture dimensions do not match Texture dimensions".into());
    }

    // Just use the first mip level for now. We ignore the builtin format - the JPEG-XR format
    // appears to override it
    let bitmap = match &atf_texture.face_mip_data[0][0] {
        ATFTextureData::JpegXR(bytes) => jpegxr_to_tiff(
            atf_texture.width,
            atf_texture.height,
            &mut Cursor::new(bytes),
        )
        .0
        .to_rgba8()
        .pixels()
        .flat_map(|p| p.0)
        .collect(),
        ATFTextureData::CompressedAlpha {
            dxt1_alpha_compressed,
            jpegxr_alpha: orig_jpegxr_alpha,
            dxt5_rgb_compressed,
            jpegxr_bgr: orig_jpegxr_bgr,
        } => {
            // See https://github.com/adobe/dds2atf/issues/5
            // The ATF format uses a weird version of LZMA that doesn't store the uncompressed length.
            // Compute it ourselves, and insert it where it should be
            let mut dxt1_alpha_compressed = dxt1_alpha_compressed.clone();
            let mut dxt5_rgb_compressed = dxt5_rgb_compressed.clone();

            let dxt1_uncompressed_length = u64::MAX;
            let dxt5_uncompressed_length = u64::MAX;

            dxt1_alpha_compressed.splice(5..5, dxt1_uncompressed_length.to_le_bytes());
            dxt5_rgb_compressed.splice(5..5, dxt5_uncompressed_length.to_le_bytes());

            let mut dxt1_alpha = Vec::with_capacity(dxt1_alpha_compressed.len());
            lzma_rs::lzma_decompress(&mut dxt1_alpha_compressed.as_slice(), &mut dxt1_alpha)
                .expect("Failed to decompress DXT1 alpha");

            let mut dxt5_rgb = Vec::with_capacity(dxt5_rgb_compressed.len());
            lzma_rs::lzma_decompress(&mut dxt5_rgb_compressed.as_slice(), &mut dxt5_rgb)
                .expect("Failed to decompress DXT5 RGB");

            // 'COMPRESSED_ALPHA' images are encoded in a very strange way. The LZMA-compressed DXT1/DXT5
            // sections just hold the DXT lookup table block. The associated values for each block
            // are stored in JPEG-XR images, which are optionally compressed
            let (jpegxr_alpha, pixel_format) = jpegxr_to_tiff(
                atf_texture.width / 4,
                atf_texture.height / 2,
                &mut Cursor::new(orig_jpegxr_alpha),
            );

            let jpegxr_alpha = jpegxr_alpha.to_rgba8();
            assert_eq!(
                pixel_format,
                PixelFormat::PixelFormat8bppGray,
                "Unexpected JPEG-XR alpha format"
            );

            let jpegxr_bgr = jpegxr_to_raw_pixels(
                atf_texture.width / 4,
                atf_texture.height / 2,
                &mut Cursor::new(orig_jpegxr_bgr),
            );

            let mut reconstructed_dxt = vec![];

            assert_eq!(dxt1_alpha.len() / 6, dxt5_rgb.len() / 4, "Bad DXT data");
            assert_eq!(
                jpegxr_alpha.as_raw().len() / 8,
                jpegxr_bgr.len() / 4,
                "Bad JPEG-XR data"
            );

            assert_eq!(
                dxt1_alpha.len() / 6,
                jpegxr_alpha.as_raw().len() / 8,
                "Dxt data doesn't match jpegxr data"
            );

            let second_half = ((atf_texture.width / 4) * (atf_texture.height / 4)) as usize;

            // The two values for each pixels are split across the upper and lower halves of the image.
            // See https://github.com/adobe/dds2atf/blob/cbc479be2e77daa273306161af571f8255aec78d/pvr2atfcore.cpp#L884
            for i in 0..(dxt1_alpha.len() / 6) {
                let alpha_lookup_table = &dxt1_alpha[i * 6..(i + 1) * 6];
                let jpegxr_alpha_first = &jpegxr_alpha.as_raw()[i * 4..(i + 1) * 4];
                let jpegxr_alpha_second =
                    &jpegxr_alpha.as_raw()[(second_half + (i * 4))..(second_half + ((i + 1) * 4))];

                let rgb_lookup_table = &dxt5_rgb[i * 4..(i + 1) * 4];
                let jpegxr_bgr_first = &jpegxr_bgr[(i * 2)..((i + 1) * 2)];
                // Each pixel is 2 bytes, so multiply second_half by 2 to get the correct
                // start of the second half of the image
                let jpegxr_bgr_second =
                    &jpegxr_bgr[(second_half * 2 + (i * 2))..(second_half * 2 + ((i + 1) * 2))];

                // We decoded as grayscale, so just use the red values (all of the rgb values
                // should be the same)
                reconstructed_dxt.push(jpegxr_alpha_first[0]);
                reconstructed_dxt.push(jpegxr_alpha_second[0]);
                // Copy the corresponding lookup table
                reconstructed_dxt.extend(alpha_lookup_table);

                // Copy the two 16-bit values
                reconstructed_dxt.extend(jpegxr_bgr_first);
                reconstructed_dxt.extend(jpegxr_bgr_second);

                // Copy the lookup table
                reconstructed_dxt.extend(rgb_lookup_table);
            }

            reconstructed_dxt
        }
        ATFTextureData::CompressedRawAlpha {
            dxt5,
            pvrtc: _,
            etc1: _,
            etc2: _,
        } => {
            // DXT5 seems to be the most widely supported, so let's use that for now.
            // TODO - fallback to other formats if DXT5 data isn't present
            // (or if we're on Android/iOS).
            if dxt5.is_empty() {
                avm2_stub_method!(
                    activation,
                    "flash.display3D.textures.Texture",
                    "uploadCompressedTextureFromByteArray",
                    "with empty DXT5 data in CompressedRawAlpha"
                );
            }
            dxt5.clone()
        }
        ATFTextureData::Unknown(_) => {
            return Err(format!("Unsupported ATF format: {:?}", atf_texture.format).into())
        }
    };

    texture
        .context3d()
        .copy_pixels_to_texture(bitmap, texture.handle(), 0);

    Ok(())
}

fn jpegxr_to_raw_pixels<R: Read + Seek>(atf_width: u32, atf_height: u32, bytes: R) -> Vec<u8> {
    let mut decoder =
        jpegxr::ImageDecode::with_reader(bytes).expect("Failed to decode JPEG-XR image");

    let pixel_format = decoder
        .get_pixel_format()
        .expect("Failed to get pixel format");
    let (jpeg_width, jpeg_height) = decoder.get_size().expect("Failed to get JPEG-XR size");

    assert_eq!(jpeg_width as u32, atf_width, "Mismatched JPEG-XR width");
    assert_eq!(jpeg_height as u32, atf_height, "Mismatched JPEG-XR height");

    let info = jpegxr::PixelInfo::from_format(pixel_format);
    let stride = jpeg_width as usize * info.bits_per_pixel() / 8;

    let size = stride * jpeg_height as usize;

    let mut output = vec![0; size];

    decoder
        .copy_all(&mut output, stride)
        .expect("Failed to decode");
    output
}

fn jpegxr_to_tiff<R: Read + Seek>(
    atf_width: u32,
    atf_height: u32,
    bytes: R,
) -> (image::DynamicImage, jpegxr::PixelFormat) {
    let mut decoder =
        jpegxr::ImageDecode::with_reader(bytes).expect("Failed to decode JPEG-XR image");

    let pixel_format = decoder
        .get_pixel_format()
        .expect("Failed to get pixel format");

    let (jpeg_width, jpeg_height) = decoder.get_size().expect("Failed to get JPEG-XR size");
    let jpeg_width = jpeg_width as u32;
    let jpeg_height = jpeg_height as u32;

    assert_eq!(jpeg_width, atf_width, "Mismatched JPEG-XR width");
    assert_eq!(jpeg_height, atf_height, "Mismatched JPEG-XR height");

    let info = jpegxr::PixelInfo::from_format(pixel_format);
    let stride = jpeg_width as usize * info.bits_per_pixel() / 8;
    let size = stride * jpeg_height as usize;

    // We convert the result to a TIFF - this makes the jpegxr library handle
    // all of the weird JPEG-XR alpha formats for us. We can then use the normal
    // `image` crate to decode the TIFF to an rgba array.
    let mut bmp_buffer = vec![0; size];
    decoder
        .convert_to_tiff(&mut Cursor::new(&mut bmp_buffer))
        .expect("Failed to convert to bitmap");

    let image_reader =
        image::ImageReader::with_format(Cursor::new(bmp_buffer), image::ImageFormat::Tiff);
    (
        image_reader.decode().expect("Failed to decode Bitmap"),
        pixel_format,
    )
}
