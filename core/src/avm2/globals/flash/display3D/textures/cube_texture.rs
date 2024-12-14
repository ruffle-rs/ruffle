use ruffle_render::backend::Context3DTextureFormat;

use crate::avm2::globals::flash::display3D::textures::atf_jpegxr::do_compressed_upload;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::Activation;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};
use crate::avm2_stub_method;

use super::texture::do_copy;

pub fn upload_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This should work, but it's currently untested
    avm2_stub_method!(
        activation,
        "flash.display3D.textures.CubeTexture",
        "uploadFromByteArray"
    );
    let texture = this.as_texture().unwrap();
    let data = args.get_object(activation, 0, "data")?;
    let byte_array_offset = args.get_u32(activation, 1)?;
    let side = args.get_u32(activation, 2)?;
    let mip_level = args.get_u32(activation, 3)?;

    do_copy(
        activation,
        data,
        texture,
        byte_array_offset,
        side,
        mip_level,
    )?;
    Ok(Value::Undefined)
}

pub fn upload_compressed_texture_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This should work, but it's currently untested
    avm2_stub_method!(
        activation,
        "flash.display3D.textures.CubeTexture",
        "uploadCompressedTextureFromByteArray"
    );

    let texture = this.as_texture().unwrap();
    let data = args.get_object(activation, 0, "data")?;
    let byte_array_offset = args.get_u32(activation, 1)? as usize;
    let async_ = args.get_bool(2);
    if async_ {
        avm2_stub_method!(
            activation,
            "flash.display3D.textures.CubeTexture",
            "uploadCompressedTextureFromByteArray",
            "with async"
        );
    }

    if !matches!(texture.original_format(), Context3DTextureFormat::Bgra) {
        avm2_stub_method!(
            activation,
            "flash.display3D.textures.CubeTexture",
            "uploadCompressedTextureFromByteArray",
            "with unsupported format"
        );
        return Ok(Value::Undefined);
    }

    do_compressed_upload(activation, texture, data, byte_array_offset, true)?;
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
            let side = args[1].coerce_to_u32(activation)?;
            let mip_level = args[2].coerce_to_u32(activation)?;
            if mip_level == 0 {
                texture.context3d().copy_bitmapdata_to_texture(
                    source.sync(activation.context.renderer),
                    texture.handle(),
                    // FIXME - is this right?
                    side,
                );
            } else {
                avm2_stub_method!(
                    activation,
                    "flash.display3D.textures.CubeTexture",
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
