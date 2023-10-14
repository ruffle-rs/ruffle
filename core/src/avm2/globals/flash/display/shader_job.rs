use ruffle_render::{
    bitmap::PixelRegion,
    pixel_bender::{
        PixelBenderParam, PixelBenderParamQualifier, PixelBenderShaderArgument,
        PixelBenderShaderHandle, PixelBenderType, OUT_COORD_NAME,
    },
};

use crate::{
    avm2::{string::AvmString, Activation, Error, Object, TObject, Value},
    avm2_stub_method,
    pixel_bender::PixelBenderTypeExt,
};

pub fn get_shader_args<'gc>(
    shader_obj: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<
    (
        PixelBenderShaderHandle,
        Vec<PixelBenderShaderArgument<'static>>,
    ),
    Error<'gc>,
> {
    // FIXME - determine what errors Flash Player throws here
    // instead of using `expect`
    let shader_data = shader_obj
        .get_public_property("data", activation)?
        .as_object()
        .expect("Missing ShaderData object")
        .as_shader_data()
        .expect("ShaderData object is not a ShaderData instance");

    let shader_handle = shader_data.pixel_bender_shader();
    let shader_handle = shader_handle
        .as_ref()
        .expect("ShaderData object has no shader");
    let shader = shader_handle.0.parsed_shader();

    let args = shader
        .params
        .iter()
        .enumerate()
        .flat_map(|(index, param)| {
            match param {
                PixelBenderParam::Normal {
                    qualifier,
                    param_type,
                    name,
                    ..
                } => {
                    if matches!(qualifier, PixelBenderParamQualifier::Output) {
                        return None;
                    }

                    if name == OUT_COORD_NAME {
                        // Pass in a dummy value - this will be ignored in favor of the actual pixel coordinate
                        return Some(PixelBenderShaderArgument::ValueInput {
                            index: index as u8,
                            value: PixelBenderType::TFloat2(f32::NAN, f32::NAN),
                        });
                    }
                    let shader_param = shader_data
                        .get_public_property(
                            AvmString::new_utf8(activation.context.gc_context, name),
                            activation,
                        )
                        .expect("Missing normal property");

                    let shader_param = shader_param
                        .as_object()
                        .expect("Shader property is not an object");

                    let value = shader_param
                        .get_public_property("value", activation)
                        .expect("Missing value property");
                    let pb_val = PixelBenderType::from_avm2_value(activation, value, param_type)
                        .expect("Failed to convert AVM2 value to PixelBenderType");

                    Some(PixelBenderShaderArgument::ValueInput {
                        index: index as u8,
                        value: pb_val,
                    })
                }
                PixelBenderParam::Texture {
                    index,
                    channels,
                    name,
                } => {
                    let shader_input = shader_data
                        .get_public_property(
                            AvmString::new_utf8(activation.context.gc_context, name),
                            activation,
                        )
                        .expect("Missing property")
                        .as_object()
                        .expect("Shader input is not an object");

                    let input = shader_input
                        .get_public_property("input", activation)
                        .expect("Missing input property");

                    let texture = if let Value::Null = input {
                        None
                    } else {
                        let input = input
                            .as_object()
                            .expect("ShaderInput.input is not an object");

                        let bitmap = input.as_bitmap_data().expect(
                            "ShaderInput.input is not a BitmapData (FIXE - support other types)",
                        );

                        Some(bitmap.bitmap_handle(
                            activation.context.gc_context,
                            activation.context.renderer,
                        ))
                    };

                    Some(PixelBenderShaderArgument::ImageInput {
                        index: *index,
                        channels: *channels,
                        name: name.clone(),
                        texture: texture.map(|t| t.into()),
                    })
                }
            }
        })
        .collect();
    Ok((shader_handle.clone(), args))
}

/// Implements `ShaderJob.start`.
pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation,
        "flash.display.ShaderJob",
        "start",
        "async execution and non-BitmapData inputs"
    );

    let shader = this
        .get_public_property("shader", activation)?
        .as_object()
        .expect("Missing Shader object");

    let (shader_handle, arguments) = get_shader_args(shader, activation)?;

    let target = this
        .get_public_property("target", activation)?
        .as_object()
        .expect("ShaderJob.target is not an object");

    let target_bitmap = target
        .as_bitmap_data()
        .expect("ShaderJob.target is not a BitmapData (FIXME - support other types)")
        .sync();

    // Perform both a GPU->CPU and CPU->GPU sync before writing to it.
    // FIXME - are both necessary?
    let mut target_bitmap_data = target_bitmap.write(activation.context.gc_context);
    target_bitmap_data.update_dirty_texture(activation.context.renderer);

    let target_handle = target_bitmap_data
        .bitmap_handle(activation.context.renderer)
        .expect("Missing handle");

    let sync_handle = activation
        .context
        .renderer
        .run_pixelbender_shader(shader_handle, &arguments, target_handle)
        .expect("Failed to run shader");

    let width = target_bitmap_data.width();
    let height = target_bitmap_data.height();
    target_bitmap_data.set_gpu_dirty(
        activation.context.gc_context,
        sync_handle,
        PixelRegion::for_whole_size(width, height),
    );

    Ok(Value::Undefined)
}
