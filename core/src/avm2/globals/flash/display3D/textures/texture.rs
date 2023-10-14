use gc_arena::GcCell;
use ruffle_render::backend::Context3DTextureFormat;

use crate::avm2::object::TextureObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::Activation;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};
use crate::avm2_stub_method;
use crate::bitmap::bitmap_data::BitmapData;
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::bitmap::bitmap_data::Color;

pub fn do_copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    data: Object<'gc>,
    texture: TextureObject<'gc>,
    byte_array_offset: u32,
    side: u32,
    mip_level: u32,
) -> Result<(), Error<'gc>> {
    if mip_level != 0 {
        avm2_stub_method!(
            activation,
            "flash.display3D.textures.Texture",
            "uploadFromByteArray",
            "with miplevel != 0"
        );
        return Ok(());
    }

    // FIXME - see if we can avoid this intermediate BitmapDataWrapper, and copy
    // directly from a buffer to the target GPU texture
    let bitmap_data = match texture.original_format() {
        Context3DTextureFormat::Bgra => {
            let width = texture.handle().width();
            let height = texture.handle().height();

            let bytearray = data.as_bytearray().unwrap();

            let colors: Vec<_> = bytearray
                .read_at((4 * width * height) as usize, byte_array_offset as usize)
                .expect("Failed to read")
                .chunks_exact(4)
                .map(|chunk| {
                    // The ByteArray is in BGRA format. FIXME - should this be premultiplied?
                    Color::argb(chunk[3], chunk[2], chunk[1], chunk[0])
                })
                .collect();

            let bitmap_data = BitmapData::new_with_pixels(width, height, true, colors);
            BitmapDataWrapper::new(GcCell::new(activation.context.gc_context, bitmap_data))
        }
        _ => {
            tracing::warn!(
                "uploadFromByteArray with unsupported format: {:?}",
                texture.original_format()
            );
            return Ok(());
        }
    };
    texture
        .context3d()
        .copy_bitmap_to_texture(bitmap_data.sync(), texture.handle(), side);
    Ok(())
}

#[cfg(not(feature = "jpegxr"))]
pub(super) fn do_compressed_upload<'gc>(
    _: &mut Activation<'_, 'gc>,
    _: TextureObject<'gc>,
    _: Object<'gc>,
    _: usize,
    _: bool,
) -> Result<(), Error<'gc>> {
    Err("Support for compressed textures not compiled in.".into())
}

#[cfg(feature = "jpegxr")]
pub(super) fn do_compressed_upload<'gc>(
    activation: &mut Activation<'_, 'gc>,
    texture: TextureObject<'gc>,
    data: Object<'gc>,
    byte_array_offset: usize,
    is_cube: bool,
) -> Result<(), Error<'gc>> {
    use ruffle_render::atf::ATFTexture;
    use std::io::Cursor;

    let atf_texture =
        ATFTexture::from_bytes(&data.as_bytearray().unwrap().bytes()[byte_array_offset..])
            .expect("Failed to parse ATF texture");

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
    let mut first_mip = Cursor::new(&atf_texture.face_mip_data[0][0]);
    let mut decoder =
        jpegxr::ImageDecode::with_reader(&mut first_mip).expect("Failed to decode JPEG-XR image");

    let pixel_format = decoder
        .get_pixel_format()
        .expect("Failed to get pixel format");
    let (jpeg_width, jpeg_height) = decoder.get_size().expect("Failed to get JPEG-XR size");
    let jpeg_width = jpeg_width as u32;
    let jpeg_height = jpeg_height as u32;

    assert_eq!(jpeg_width, atf_texture.width, "Mismatched JPEG-XR width");
    assert_eq!(jpeg_height, atf_texture.height, "Mismatched JPEG-XR height");

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
        image::io::Reader::with_format(Cursor::new(bmp_buffer), image::ImageFormat::Tiff);
    let bitmap = image_reader
        .decode()
        .expect("Failed to decode Bitmap")
        .to_rgba8();

    // FIXME - are we handling premultiplied alpha correct?
    let colors: Vec<_> = bitmap
        .chunks_exact(4)
        .map(|color| Color::argb(color[3], color[0], color[1], color[2]))
        .collect();

    let bitmap_data = BitmapData::new_with_pixels(
        texture.handle().width(),
        texture.handle().height(),
        true,
        colors,
    );

    let bitmap_data =
        BitmapDataWrapper::new(GcCell::new(activation.context.gc_context, bitmap_data));

    texture
        .context3d()
        .copy_bitmap_to_texture(bitmap_data.sync(), texture.handle(), 0);

    Ok(())
}

pub fn upload_compressed_texture_from_byte_array_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let texture = this.as_texture().unwrap();
    let data = args.get_object(activation, 0, "data")?;
    let byte_array_offset = args.get_u32(activation, 1)? as usize;

    if !matches!(texture.original_format(), Context3DTextureFormat::Bgra) {
        avm2_stub_method!(
            activation,
            "flash.display3D.textures.Texture",
            "uploadCompressedTextureFromByteArray",
            "with unsupported format"
        );
        return Ok(Value::Undefined);
    }

    do_compressed_upload(activation, texture, data, byte_array_offset, false)?;

    Ok(Value::Undefined)
}

pub fn upload_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let texture = this.as_texture().unwrap();
    let data = args.get_object(activation, 0, "data")?;
    let byte_array_offset = args.get_u32(activation, 1)?;
    let mip_level = args.get_u32(activation, 2)?;

    do_copy(activation, data, texture, byte_array_offset, 0, mip_level)?;
    Ok(Value::Undefined)
}

pub fn upload_from_bitmap_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(texture) = this.as_texture() {
        if let Some(source) = args[0].coerce_to_object(activation)?.as_bitmap_data() {
            let mip_level = args[1].coerce_to_u32(activation)?;
            if mip_level == 0 {
                texture
                    .context3d()
                    .copy_bitmap_to_texture(source.sync(), texture.handle(), 0);
            } else {
                avm2_stub_method!(
                    activation,
                    "flash.display3D.textures.Texture",
                    "uploadFromBitmapData",
                    "with miplevel != 0"
                );
            }
        } else {
            panic!("Invalid source: {:?}", args[0]);
        }
    }
    Ok(Value::Undefined)
}
