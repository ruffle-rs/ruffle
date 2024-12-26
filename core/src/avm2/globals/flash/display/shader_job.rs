use crate::avm2::bytearray::Endian;
use crate::avm2::globals::slots::{
    flash_display_shader as shader_slots, flash_display_shader_input as shader_input_slots,
    flash_display_shader_job as shader_job_slots,
    flash_display_shader_parameter as shader_parameter_slots,
};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::pixel_bender::PixelBenderTypeExt;
use crate::string::AvmString;

use crate::avm2_stub_method;

use ruffle_render::backend::{PixelBenderOutput, PixelBenderTarget};
use ruffle_render::bitmap::PixelRegion;
use ruffle_render::pixel_bender::{
    ImageInputTexture, PixelBenderParam, PixelBenderParamQualifier, PixelBenderShaderArgument,
    PixelBenderShaderHandle, PixelBenderType, OUT_COORD_NAME,
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
        .get_slot(shader_slots::_DATA)
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
                        .get_string_property_local(
                            AvmString::new_utf8(activation.gc(), name),
                            activation,
                        )
                        .expect("Missing normal property");

                    let shader_param = shader_param
                        .as_object()
                        .expect("Shader property is not an object");

                    if !shader_param.is_of_type(
                        activation
                            .avm2()
                            .classes()
                            .shaderparameter
                            .inner_class_definition(),
                    ) {
                        panic!("Expected shader parameter to be of class ShaderParameter");
                    }

                    let value = shader_param.get_slot(shader_parameter_slots::_VALUE);

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
                        .get_string_property_local(
                            AvmString::new_utf8(activation.gc(), name),
                            activation,
                        )
                        .expect("Missing property")
                        .as_object()
                        .expect("Shader input is not an object");

                    if !shader_input.is_of_type(
                        activation
                            .avm2()
                            .classes()
                            .shaderinput
                            .inner_class_definition(),
                    ) {
                        panic!("Expected shader input to be of class ShaderInput");
                    }

                    let input = shader_input.get_slot(shader_input_slots::_INPUT);

                    let width = shader_input.get_slot(shader_input_slots::_WIDTH).as_u32();
                    let height = shader_input.get_slot(shader_input_slots::_HEIGHT).as_u32();

                    let input_channels = shader_input
                        .get_slot(shader_input_slots::_CHANNELS)
                        .as_u32();

                    assert_eq!(*channels as u32, input_channels);

                    let texture = if let Some(input) = input.as_object() {
                        let input_texture = if let Some(bitmap) = input.as_bitmap_data() {
                            ImageInputTexture::Bitmap(
                                bitmap.bitmap_handle(activation.gc(), activation.context.renderer),
                            )
                        } else if let Some(byte_array) = input.as_bytearray() {
                            let expected_len = (width * height * input_channels) as usize
                                * std::mem::size_of::<f32>();
                            assert_eq!(byte_array.len(), expected_len);
                            assert_eq!(byte_array.endian(), Endian::Little);
                            ImageInputTexture::Bytes {
                                width,
                                height,
                                channels: input_channels,
                                bytes: byte_array.read_at(0, byte_array.len()).unwrap().to_vec(),
                            }
                        } else if let Some(vector) = input.as_vector_storage() {
                            let expected_len = (width * height * input_channels) as usize;
                            assert_eq!(vector.length(), expected_len);
                            ImageInputTexture::Bytes {
                                width,
                                height,
                                channels: input_channels,
                                bytes: vector
                                    .iter()
                                    .flat_map(|val| (val.as_f64() as f32).to_le_bytes())
                                    .collect(),
                            }
                        } else {
                            panic!("Unexpected input object {input:?}");
                        };
                        Some(input_texture)
                    } else {
                        // Null input
                        None
                    };

                    Some(PixelBenderShaderArgument::ImageInput {
                        index: *index,
                        channels: *channels,
                        name: name.clone(),
                        texture,
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let wait_for_completion = args.get_bool(0);
    if !wait_for_completion {
        avm2_stub_method!(
            activation,
            "flash.display.ShaderJob",
            "start",
            "with waitForCompletion=false"
        );
    }
    let shader = this
        .get_slot(shader_job_slots::_SHADER)
        .as_object()
        .expect("Missing Shader object");

    let (shader_handle, arguments) = get_shader_args(shader, activation)?;

    let target = this
        .get_slot(shader_job_slots::_TARGET)
        .as_object()
        .expect("ShaderJob.target is not an object");

    let output_width = this.get_slot(shader_job_slots::_WIDTH).as_u32();

    let output_height = this.get_slot(shader_job_slots::_HEIGHT).as_u32();

    let pixel_bender_target = if let Some(bitmap) = target.as_bitmap_data() {
        let target_bitmap = bitmap.sync(activation.context.renderer);
        // Perform both a GPU->CPU and CPU->GPU sync before writing to it.
        // FIXME - are both necessary?
        let mut target_bitmap_data = target_bitmap.write(activation.gc());
        target_bitmap_data.update_dirty_texture(activation.context.renderer);

        PixelBenderTarget::Bitmap(
            target_bitmap_data
                .bitmap_handle(activation.context.renderer)
                .expect("Missing handle"),
        )
    } else {
        PixelBenderTarget::Bytes {
            width: output_width,
            height: output_height,
        }
    };

    let output = activation
        .context
        .renderer
        .run_pixelbender_shader(shader_handle, &arguments, &pixel_bender_target)
        .expect("Failed to run shader");

    match output {
        PixelBenderOutput::Bitmap(sync_handle) => {
            let target_bitmap = target
                .as_bitmap_data()
                .unwrap()
                .sync(activation.context.renderer);
            let mut target_bitmap_data = target_bitmap.write(activation.gc());
            let width = target_bitmap_data.width();
            let height = target_bitmap_data.height();
            target_bitmap_data.set_gpu_dirty(
                activation.gc(),
                sync_handle,
                PixelRegion::for_whole_size(width, height),
            );
        }
        PixelBenderOutput::Bytes(pixels) => {
            if let Some(mut bytearray) = target.as_bytearray_mut() {
                bytearray.write_at(&pixels, 0).unwrap();
            } else if let Some(mut vector) = target.as_vector_storage_mut(activation.gc()) {
                let new_storage: Vec<_> = bytemuck::cast_slice::<u8, f32>(&pixels)
                    .iter()
                    .map(|p| Value::from(*p as f64))
                    .collect();
                vector.replace_storage(new_storage);
            } else {
                panic!("Unexpected target object {target:?}");
            }
        }
    }

    Ok(Value::Undefined)
}
