use crate::{
    avm2::{
        parameters::ParametersExt, string::AvmString, Activation, Error, Object, TObject, Value,
    },
    pixel_bender::{PixelBenderParam, PixelBenderParamQualifier},
};

use super::shader_parameter::make_shader_parameter;

/// Implements `ShaderData.init`, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut this = this.unwrap();
    let bytecode = args.get_object(activation, 0, "bytecode")?;
    let bytecode = bytecode.as_bytearray().unwrap();
    let shader = crate::pixel_bender::parse_shader(bytecode.bytes());

    for meta in shader.metadata {
        let name = AvmString::new_utf8(activation.context.gc_context, &meta.key);
        let value = meta.value.into_avm2_value(activation)?;
        this.set_public_property(name, value, activation)?;
    }
    this.set_public_property(
        "name",
        AvmString::new_utf8(activation.context.gc_context, &shader.name).into(),
        activation,
    )?;

    for (index, param) in shader.params.into_iter().enumerate() {
        let name = match &param {
            PixelBenderParam::Normal {
                name, qualifier, ..
            } => {
                // Neither of these show up in Flash Player
                if name == "_OutCoord" || matches!(qualifier, PixelBenderParamQualifier::Output) {
                    continue;
                }
                name
            }
            PixelBenderParam::Texture { name, .. } => name,
        };

        let name = AvmString::new_utf8(activation.context.gc_context, name);
        let param_obj = make_shader_parameter(activation, param, index)?;
        this.set_public_property(name, param_obj, activation)?;
    }
    Ok(Value::Undefined)
}
