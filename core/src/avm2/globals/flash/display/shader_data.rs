use crate::avm2::activation::Activation;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::pixel_bender::PixelBenderTypeExt;
use crate::string::AvmString;

use ruffle_macros::istr;
use ruffle_render::pixel_bender::{
    parse_shader, PixelBenderParam, PixelBenderParamQualifier, OUT_COORD_NAME,
};

use super::shader_parameter::make_shader_parameter;

pub use crate::avm2::object::shader_data_allocator;

/// Implements `ShaderData.init`, which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let bytecode = args.get_object(activation, 0, "bytecode")?;
    let bytecode = bytecode.as_bytearray().unwrap();
    let shader = parse_shader(bytecode.bytes()).expect("Failed to parse PixelBender");

    for meta in &shader.metadata {
        let name = AvmString::new_utf8(activation.gc(), &meta.key);
        // Top-level metadata appears to turn `TInt` into a plain integer value,
        // rather than a single-element array.
        let value = meta.value.as_avm2_value(activation, true)?;
        this.set_string_property_local(name, value, activation)?;
    }
    this.set_string_property_local(
        istr!("name"),
        AvmString::new_utf8(activation.gc(), &shader.name).into(),
        activation,
    )?;

    let mut normal_index = 0;
    let mut texture_index = 0;

    for param in &shader.params {
        let (name, index) = match &param {
            PixelBenderParam::Normal {
                name, qualifier, ..
            } => {
                // Neither of these show up in Flash Player
                if name == OUT_COORD_NAME || matches!(qualifier, PixelBenderParamQualifier::Output)
                {
                    continue;
                }
                let index = normal_index;
                normal_index += 1;
                (name, index)
            }
            PixelBenderParam::Texture { name, .. } => {
                let index = texture_index;
                texture_index += 1;
                (name, index)
            }
        };

        let name = AvmString::new_utf8(activation.gc(), name);
        let param_obj = make_shader_parameter(activation, param, index)?;
        this.set_string_property_local(name, param_obj, activation)?;
    }

    let shader_handle = activation
        .context
        .renderer
        .compile_pixelbender_shader(shader)
        .expect("Failed to compile PixelBender shader");

    this.as_shader_data()
        .unwrap()
        .set_pixel_bender_shader(shader_handle);
    Ok(Value::Undefined)
}
