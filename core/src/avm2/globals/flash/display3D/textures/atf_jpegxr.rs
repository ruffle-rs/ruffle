use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::error::{Error, make_error_3675, make_error_3679};
use crate::avm2::object::{Object, TextureObject};
use crate::avm2_stub_method;
use ruffle_render::atf::ATFTexture;
use ruffle_render::atf::ATFTextureData;
use ruffle_render::jpegxr::{self, JpegXrImage};

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

    if atf_texture.width != texture.handle().width()
        || atf_texture.height != texture.handle().height()
    {
        return Err(make_error_3679(activation));
    }

    if is_cube != atf_texture.cubemap {
        return Err(make_error_3675(activation));
    }

    // Just use the first mip level for now. We ignore the builtin format - the JPEG-XR format
    // appears to override it
    let bitmap = match &atf_texture.face_mip_data[0][0] {
        ATFTextureData::JpegXR(bytes) => check_size(
            jpegxr::decode_rgba8(bytes).expect("Failed to decode JPEG-XR image"),
            atf_texture.width,
            atf_texture.height,
        ),
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
            assert!(
                jpegxr::is_gray8(orig_jpegxr_alpha).expect("Failed to decode JPEG-XR image"),
                "Unexpected JPEG-XR alpha format"
            );
            let jpegxr_alpha = check_size(
                jpegxr::decode_rgba8(orig_jpegxr_alpha).expect("Failed to decode JPEG-XR image"),
                atf_texture.width / 4,
                atf_texture.height / 2,
            );

            let jpegxr_bgr = check_size(
                jpegxr::decode_rgb565(orig_jpegxr_bgr).expect("Failed to decode JPEG-XR image"),
                atf_texture.width / 4,
                atf_texture.height / 2,
            );

            let mut reconstructed_dxt = vec![];

            assert_eq!(dxt1_alpha.len() / 6, dxt5_rgb.len() / 4, "Bad DXT data");
            assert_eq!(
                jpegxr_alpha.len() / 8,
                jpegxr_bgr.len() / 4,
                "Bad JPEG-XR data"
            );

            assert_eq!(
                dxt1_alpha.len() / 6,
                jpegxr_alpha.len() / 8,
                "Dxt data doesn't match jpegxr data"
            );

            let second_half = ((atf_texture.width / 4) * (atf_texture.height / 4)) as usize;

            // The two values for each pixels are split across the upper and lower halves of the image.
            // See https://github.com/adobe/dds2atf/blob/cbc479be2e77daa273306161af571f8255aec78d/pvr2atfcore.cpp#L884
            for i in 0..(dxt1_alpha.len() / 6) {
                let alpha_lookup_table = &dxt1_alpha[i * 6..(i + 1) * 6];
                let jpegxr_alpha_first = &jpegxr_alpha[i * 4..(i + 1) * 4];
                // Each pixel is 4 bytes (RGBA), so multiply the pixel index by 4
                let jpegxr_alpha_second =
                    &jpegxr_alpha[((second_half + i) * 4)..((second_half + i + 1) * 4)];

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
            return Err(format!("Unsupported ATF format: {:?}", atf_texture.format).into());
        }
    };

    texture
        .context3d()
        .copy_pixels_to_texture(bitmap, texture.handle(), 0);

    Ok(())
}

fn check_size(image: JpegXrImage, atf_width: u32, atf_height: u32) -> Vec<u8> {
    assert_eq!(image.width, atf_width, "Mismatched JPEG-XR width");
    assert_eq!(image.height, atf_height, "Mismatched JPEG-XR height");
    image.data
}
