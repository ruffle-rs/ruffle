use crate::avm2::activation::Activation;
use crate::avm2::globals::slots::flash_display_shader_input as input_slots;
use crate::avm2::globals::slots::flash_display_shader_parameter as parameter_slots;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::pixel_bender::PixelBenderTypeExt;
use crate::string::AvmString;

use ruffle_macros::istr;
use ruffle_render::pixel_bender::PixelBenderParam;

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
            let param_value = activation
                .avm2()
                .classes()
                .shaderparameter
                .construct(activation, &[])?;

            let param_object = param_value.as_object().unwrap();

            let type_name = AvmString::new_utf8(activation.gc(), param_type.to_string());

            param_object.set_slot(parameter_slots::_INDEX, index.into(), activation)?;
            param_object.set_slot(parameter_slots::_TYPE, type_name.into(), activation)?;
            for meta in metadata {
                let name = AvmString::new_utf8(activation.gc(), &meta.key);
                let value = meta.value.clone().as_avm2_value(activation, false)?;
                param_value.set_public_property(name, value, activation)?;

                if &*name == b"defaultValue" {
                    param_object.set_slot(parameter_slots::_VALUE, value, activation)?;
                }
            }
            param_object.set_string_property_local(
                istr!("name"),
                AvmString::new_utf8(activation.gc(), name).into(),
                activation,
            )?;
            Ok(param_value)
        }
        PixelBenderParam::Texture { name, channels, .. } => {
            let obj = activation
                .avm2()
                .classes()
                .shaderinput
                .construct(activation, &[])?
                .as_object()
                .unwrap();

            obj.set_slot(input_slots::_CHANNELS, (*channels).into(), activation)?;
            obj.set_slot(input_slots::_INDEX, index.into(), activation)?;
            obj.set_string_property_local(
                istr!("name"),
                AvmString::new_utf8(activation.gc(), name).into(),
                activation,
            )?;
            Ok(obj.into())
        }
    }
}
