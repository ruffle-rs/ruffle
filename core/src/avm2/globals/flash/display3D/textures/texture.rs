use gc_arena::GcCell;

use ruffle_render::backend::Context3DTextureFormat;

use super::atf_jpegxr::do_compressed_upload;
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
    texture.context3d().copy_bitmapdata_to_texture(
        bitmap_data.sync(activation.context.renderer),
        texture.handle(),
        side,
    );
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

    if !matches!(
        texture.original_format(),
        Context3DTextureFormat::Bgra | Context3DTextureFormat::CompressedAlpha
    ) {
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
        let source_obj = args.get_object(activation, 0, "source")?;

        if let Some(source) = source_obj.as_bitmap_data() {
            let mip_level = args[1].coerce_to_u32(activation)?;
            if mip_level == 0 {
                texture.context3d().copy_bitmapdata_to_texture(
                    source.sync(activation.context.renderer),
                    texture.handle(),
                    0,
                );
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
