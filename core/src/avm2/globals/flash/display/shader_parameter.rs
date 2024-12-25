use crate::avm2::globals::slots::flash_display_shader_input as input_slots;
use crate::avm2::globals::slots::flash_display_shader_parameter as parameter_slots;
use ruffle_render::pixel_bender::PixelBenderParam;

use crate::{
    avm2::{string::AvmString, Activation, Error, TObject, Value},
    pixel_bender::PixelBenderTypeExt,
};

pub fn make_shader_parameter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    param: &PixelBenderParam,
    index: usize,
) -> Result<Value<'gc>, Error<'gc>> {
    match param {
        PixelBenderParam::Normal {
            name,
            param_type,
            metadata,
            ..
        } => {
            let obj = activation
                .avm2()
                .classes()
                .shaderparameter
                .construct(activation, &[])?;
            let type_name = AvmString::new_utf8(activation.context.gc(), param_type.to_string());

            obj.set_slot(parameter_slots::_INDEX, index.into(), activation)?;
            obj.set_slot(parameter_slots::_TYPE, type_name.into(), activation)?;
            for meta in metadata {
                let name = AvmString::new_utf8(activation.context.gc(), &meta.key);
                let value = meta.value.clone().as_avm2_value(activation, false)?;
                obj.set_public_property(name, value, activation)?;

                if &*name == b"defaultValue" {
                    obj.set_slot(parameter_slots::_VALUE, value, activation)?;
                }
            }
            obj.set_string_property_local(
                "name",
                AvmString::new_utf8(activation.context.gc(), name).into(),
                activation,
            )?;
            Ok(obj.into())
        }
        PixelBenderParam::Texture { name, channels, .. } => {
            let obj = activation
                .avm2()
                .classes()
                .shaderinput
                .construct(activation, &[])?;
            obj.set_slot(input_slots::_CHANNELS, (*channels).into(), activation)?;
            obj.set_slot(input_slots::_INDEX, index.into(), activation)?;
            obj.set_string_property_local(
                "name",
                AvmString::new_utf8(activation.context.gc(), name).into(),
                activation,
            )?;
            Ok(obj.into())
        }
    }
}
