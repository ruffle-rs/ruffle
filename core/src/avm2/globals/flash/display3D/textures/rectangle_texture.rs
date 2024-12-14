use crate::avm2::parameters::ParametersExt;
use crate::avm2::Activation;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

use super::texture::do_copy;

pub fn upload_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let texture = this.as_texture().unwrap();
    let data = args.get_object(activation, 0, "data")?;
    let byte_array_offset = args.get_u32(activation, 1)?;

    do_copy(activation, data, texture, byte_array_offset, 0, 0)?;
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
            texture.context3d().copy_bitmapdata_to_texture(
                source.sync(activation.context.renderer),
                texture.handle(),
                0,
            );
        } else {
            panic!("Invalid source: {:?}", args[0]);
        }
    }
    Ok(Value::Undefined)
}
