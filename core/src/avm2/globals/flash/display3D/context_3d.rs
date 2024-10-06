use crate::avm2::error::{argument_error, error, make_error_2008};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::Activation;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};
use crate::avm2_stub_method;
use ruffle_render::backend::Context3DWrapMode;
use ruffle_render::backend::{
    BufferUsage, Context3DBlendFactor, Context3DCompareMode, Context3DTextureFormat,
    Context3DTriangleFace, Context3DVertexBufferFormat, ProgramType,
};
use ruffle_render::backend::{Context3DProfile, Context3DTextureFilter};
use swf::{Rectangle, Twips};

pub fn create_index_buffer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // FIXME - get bufferUsage and pass it through
        let num_indices = args.get_u32(activation, 0)?;
        return context.create_index_buffer(num_indices, activation);
    }
    Ok(Value::Undefined)
}

pub fn create_vertex_buffer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // FIXME - get bufferUsage and pass it through
        let num_vertices = args.get_u32(activation, 0)?;
        let data_32_per_vertex = args.get_u32(activation, 1)?;

        if data_32_per_vertex > 64 {
            return Err("data_32_per_vertex is greater than 64".into());
        }

        return context.create_vertex_buffer(
            num_vertices,
            data_32_per_vertex as u8,
            BufferUsage::DynamicDraw,
            activation,
        );
    }
    Ok(Value::Undefined)
}

pub fn configure_back_buffer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut context) = this.as_context_3d() {
        let width = args.get_u32(activation, 0)?;
        let height = args.get_u32(activation, 1)?;
        let anti_alias = args.get_u32(activation, 2)?;
        let enable_depth_and_stencil = args.get_bool(3);

        let old_swf = activation.context.swf.version() < 30;

        if old_swf && width == 0 && height == 0 && anti_alias == 0 && !enable_depth_and_stencil {
            return Ok(Value::Undefined);
        }

        if width < 32 || width > 16384 {
            return Err(Error::AvmError(error(
                activation,
                if old_swf {
                    "Error #3669: Bad input size."
                } else {
                    "Error #3780: Requested width of backbuffer is not in allowed range 32 to 16384."
                },
                if old_swf { 3669 } else { 3780 },
            )?));
        }

        if height < 32 || height > 16384 {
            return Err(Error::AvmError(error(
                activation,
                if old_swf {
                    "Error #3669: Bad input size."
                } else {
                    "Error #3781: Requested height of backbuffer is not in allowed range 32 to 16384."
                },
                if old_swf { 3669 } else { 3781 },
            )?));
        }

        let wants_best_resolution = args.get(4).unwrap_or(&Value::Undefined).coerce_to_boolean();
        let wants_best_resolution_on_browser_zoom =
            args.get(5).unwrap_or(&Value::Undefined).coerce_to_boolean();

        if wants_best_resolution {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "configureBackBuffer",
                "wantsBestResolution"
            );
        }
        if wants_best_resolution_on_browser_zoom {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "configureBackBuffer",
                "wantsBestResolutionOnBrowserZoom"
            );
        }

        context.configure_back_buffer(
            width,
            height,
            anti_alias,
            enable_depth_and_stencil,
            wants_best_resolution,
            wants_best_resolution_on_browser_zoom,
        );
    }
    Ok(Value::Undefined)
}

pub fn set_vertex_buffer_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        let index = args.get_u32(activation, 0)?;
        let buffer = args.try_get_object(activation, 1);

        let buffer = if let Some(buffer) = buffer {
            // Note - we only check the format string if the buffer is non-null
            let format = args.get_string(activation, 3)?;

            let format = if &*format == b"float4" {
                Context3DVertexBufferFormat::Float4
            } else if &*format == b"float3" {
                Context3DVertexBufferFormat::Float3
            } else if &*format == b"float2" {
                Context3DVertexBufferFormat::Float2
            } else if &*format == b"float1" {
                Context3DVertexBufferFormat::Float1
            } else if &*format == b"bytes4" {
                Context3DVertexBufferFormat::Bytes4
            } else {
                return Err(Error::AvmError(argument_error(
                    activation,
                    "Error #2008: Parameter vertexStreamFormat must be one of the accepted values.",
                    2008,
                )?));
            };

            Some((buffer.as_vertex_buffer().unwrap(), format))
        } else {
            None
        };

        let buffer_offset = args.get_u32(activation, 2)?;

        context.set_vertex_buffer_at(index, buffer, buffer_offset);
    }
    Ok(Value::Undefined)
}

pub fn create_program<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        return context.create_program(activation);
    }
    Ok(Value::Undefined)
}

pub fn set_program<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        let program = args
            .try_get_object(activation, 0)
            .map(|p| p.as_program_3d().unwrap());
        context.set_program(program);
    }
    Ok(Value::Undefined)
}

pub fn draw_triangles<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        let index_buffer = args
            .get_object(activation, 0, "indexBuffer")?
            .as_index_buffer()
            .unwrap();

        let first_index = args.get_u32(activation, 1)?;
        let num_triangles = args.get_u32(activation, 2)? as i32;

        context.draw_triangles(index_buffer, first_index, num_triangles);
    }
    Ok(Value::Undefined)
}

pub fn present<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        context.present(activation)?;
    }
    Ok(Value::Undefined)
}

pub fn get_profile<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        return match context.with_context_3d(|context| context.profile()) {
            Context3DProfile::Baseline => Ok("baseline".into()),
            Context3DProfile::BaselineConstrained => Ok("baselineConstrained".into()),
            Context3DProfile::BaselineExtended => Ok("baselineExtended".into()),
            Context3DProfile::Standard => Ok("standard".into()),
            Context3DProfile::StandardConstrained => Ok("standardConstrained".into()),
            Context3DProfile::StandardExtended => Ok("standardExtended".into()),
        };
    }
    Ok(Value::Undefined)
}

pub fn set_culling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        let culling = args.get_string(activation, 0)?;

        let culling = if &*culling == b"none" {
            Context3DTriangleFace::None
        } else if &*culling == b"back" {
            Context3DTriangleFace::Back
        } else if &*culling == b"front" {
            Context3DTriangleFace::Front
        } else if &*culling == b"frontAndBack" {
            Context3DTriangleFace::FrontAndBack
        } else {
            tracing::error!("Unknown culling {:?}", culling);
            Context3DTriangleFace::None
        };

        context.set_culling(culling);
    }
    Ok(Value::Undefined)
}

pub fn set_program_constants_from_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        let program_type = args.get_string(activation, 0)?;

        let is_vertex = if &*program_type == b"vertex" {
            ProgramType::Vertex
        } else if &*program_type == b"fragment" {
            ProgramType::Fragment
        } else {
            panic!("Unknown program type {program_type:?}");
        };

        let first_register = args.get_u32(activation, 1)?;

        let mut matrix = args.get_object(activation, 2, "matrix")?;

        let user_transposed_matrix = args.get_bool(3);

        // Hack - we store in column-major form, but we need it in row-major form
        // So, do the *opposite* of what the user pasess in`
        // or that's what I thought, but doing this seems to work???
        //
        // It seems like the documentation is wrong - we really copy to the registers
        // in column-major order.
        // See https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/display3D/Context3D.hx#L1532-L1550
        if user_transposed_matrix {
            matrix = matrix
                .call_public_property("clone", &[], activation)?
                .coerce_to_object(activation)?;

            matrix.call_public_property("transpose", &[], activation)?;
        }

        let matrix_raw_data = matrix
            .get_public_property("rawData", activation)?
            .coerce_to_object(activation)?;
        let matrix_raw_data = matrix_raw_data
            .as_vector_storage()
            .unwrap()
            .iter()
            .map(|val| val.as_f64() as f32)
            .collect::<Vec<f32>>();

        context.set_program_constants_from_matrix(is_vertex, first_register, matrix_raw_data);
    }
    Ok(Value::Undefined)
}

pub fn set_program_constants_from_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        let program_type = args.get_string(activation, 0)?;

        let program_type = if &*program_type == b"vertex" {
            ProgramType::Vertex
        } else if &*program_type == b"fragment" {
            ProgramType::Fragment
        } else {
            panic!("Unknown program type {:?}", program_type);
        };

        let first_register = args.get_u32(activation, 1)?;

        let vector = args.get_object(activation, 2, "vector")?;
        let vector = vector.as_vector_storage().unwrap();

        let num_registers = args.get_i32(activation, 3)?;

        let to_take = if num_registers != -1 {
            // Each register requires 4 floating-point values
            // FIXME - throw an error if 'vector' is too small
            num_registers as usize * 4
        } else {
            vector.length()
        };

        let raw_data = vector
            .iter()
            .map(|val| val.as_f64() as f32)
            .take(to_take)
            .collect::<Vec<f32>>();

        context.set_program_constants_from_matrix(program_type, first_register, raw_data);
    }
    Ok(Value::Undefined)
}

pub fn clear<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let red = args[0].as_f64();
        let green = args[1].as_f64();
        let blue = args[2].as_f64();
        let alpha = args[3].as_f64();
        let depth = args[4].as_f64();
        let stencil = args[5].as_i32() as u32;
        let mask = args[6].as_i32() as u32;
        context.set_clear(red, green, blue, alpha, depth, stencil, mask);
    }
    Ok(Value::Undefined)
}

pub fn create_texture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let width = args[0].as_i32() as u32;
        let height = args[1].as_i32() as u32;
        let format = args[2].coerce_to_string(activation)?;
        let optimize_for_render_to_texture = args[3].coerce_to_boolean();
        let streaming_levels = args[4].as_i32() as u32;
        let format = Context3DTextureFormat::from_wstr(&format).ok_or_else(|| {
            Error::RustError(
                format!("Unsupported texture format in createTexture: {:?}", format).into(),
            )
        })?;

        let class = activation.avm2().classes().texture;

        return context.create_texture(
            width,
            height,
            format,
            optimize_for_render_to_texture,
            streaming_levels,
            class,
            activation,
        );
    }
    Ok(Value::Undefined)
}

pub fn create_rectangle_texture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let width = args[0].as_i32() as u32;
        let height = args[1].as_i32() as u32;
        let format = args[2].coerce_to_string(activation)?;
        let optimize_for_render_to_texture = args[3].coerce_to_boolean();
        let format = Context3DTextureFormat::from_wstr(&format).ok_or_else(|| {
            Error::RustError(
                format!(
                    "Unsupported texture format in createRectangleTexture: {:?}",
                    format
                )
                .into(),
            )
        })?;

        let class = activation.avm2().classes().rectangletexture;

        return context.create_texture(
            width,
            height,
            format,
            optimize_for_render_to_texture,
            0,
            class,
            activation,
        );
    }
    Ok(Value::Undefined)
}

pub fn create_cube_texture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let size = args[0].as_i32() as u32;
        let format = args[1].coerce_to_string(activation)?;
        let optimize_for_render_to_texture = args[2].coerce_to_boolean();
        let streaming_levels = args[3].as_i32() as u32;
        let format = Context3DTextureFormat::from_wstr(&format).ok_or_else(|| {
            Error::RustError(
                format!(
                    "Unsupported texture format in createCubeTexture: {:?}",
                    format
                )
                .into(),
            )
        })?;

        return context.create_cube_texture(
            size,
            format,
            optimize_for_render_to_texture,
            streaming_levels,
            activation,
        );
    }
    Ok(Value::Undefined)
}

pub fn set_texture_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let sampler = args[0].as_i32() as u32;
        let mut cube = false;
        let texture_object = args.try_get_object(activation, 1);
        let texture = if let Some(texture_object) = texture_object {
            cube = texture_object.is_of_type(
                activation
                    .avm2()
                    .classes()
                    .cubetexture
                    .inner_class_definition(),
            );

            Some(texture_object.as_texture().unwrap().handle())
        } else {
            None
        };

        context.set_texture_at(sampler, texture, cube);
    }
    Ok(Value::Undefined)
}

pub fn set_color_mask<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let red = args[0].coerce_to_boolean();
        let green = args[1].coerce_to_boolean();
        let blue = args[2].coerce_to_boolean();
        let alpha = args[3].coerce_to_boolean();
        context.set_color_mask(red, green, blue, alpha);
    }
    Ok(Value::Undefined)
}

pub fn set_depth_test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let depth_mask = args[0].coerce_to_boolean();
        let pass_compare_mode = args[1].coerce_to_string(activation)?;
        let pass_compare_mode =
            if let Some(mode) = Context3DCompareMode::from_wstr(&pass_compare_mode) {
                mode
            } else {
                return Err(format!("Unsupported depth test mode: {:?}", pass_compare_mode).into());
            };
        context.set_depth_test(depth_mask, pass_compare_mode);
    }
    Ok(Value::Undefined)
}

pub fn set_blend_factors<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let source_factor = args[0].coerce_to_string(activation)?;
        let destination_factor = args[1].coerce_to_string(activation)?;

        let source_factor = if let Some(factor) = Context3DBlendFactor::from_wstr(&source_factor) {
            factor
        } else {
            return Err(format!("Unsupported source blend factor: {:?}", source_factor).into());
        };
        let destination_factor = if let Some(factor) =
            Context3DBlendFactor::from_wstr(&destination_factor)
        {
            factor
        } else {
            return Err(format!("Unsupported dest blend factor: {:?}", destination_factor).into());
        };
        context.set_blend_factors(source_factor, destination_factor);
    }
    Ok(Value::Undefined)
}

pub fn set_render_to_texture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let context = this.as_context_3d().unwrap();
    let texture = args
        .get_object(activation, 0, "texture")?
        .as_texture()
        .unwrap();
    let enable_depth_and_stencil = args.get_bool(1);
    let anti_alias = args.get_u32(activation, 2)?;
    let surface_selector = args.get_u32(activation, 3)?;
    let color_output_index = args.get_u32(activation, 4)?;

    let mut error = None;
    if texture.instance_class() == activation.avm2().class_defs().cubetexture {
        if surface_selector > 5 {
            error = Some((
                3772,
                "Error #3772: Cube textures need to have surfaceSelector [0..5].",
            ));
        }
    } else if texture.instance_class() == activation.avm2().class_defs().rectangletexture {
        if surface_selector != 0 {
            error = Some((
                3773,
                "Error #3773: Rectangle textures need to have surfaceSelector = 0.",
            ));
        }
    } else {
        // normal Texture or video texture (but the latter should probably not be supported here anyway)
        if surface_selector != 0 {
            error = Some((
                3771,
                "Error #3771: 2D textures need to have surfaceSelector = 0.",
            ));
        }
    }
    if let Some((code, message)) = error {
        return Err(Error::AvmError(argument_error(activation, message, code)?));
    }

    if anti_alias != 0 {
        avm2_stub_method!(
            activation,
            "flash.display3D.Context3D",
            "setRenderToTexture",
            "antiAlias != 0"
        );
    }

    if color_output_index != 0 {
        avm2_stub_method!(
            activation,
            "flash.display3D.Context3D",
            "setRenderToTexture",
            "colorOutputIndex != 0"
        );
    }

    context.set_render_to_texture(
        texture.handle(),
        enable_depth_and_stencil,
        anti_alias,
        surface_selector,
    );
    Ok(Value::Undefined)
}

pub fn set_render_to_back_buffer<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let context = this.as_context_3d().unwrap();
    context.set_render_to_back_buffer();
    Ok(Value::Undefined)
}

pub fn set_sampler_state_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.as_context_3d() {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let sampler = args[0].as_i32() as u32;
        let wrap = args[1].coerce_to_string(activation)?;
        let filter = args[2].coerce_to_string(activation)?;
        let mip_filter = args[3].coerce_to_string(activation)?;

        let wrap = Context3DWrapMode::from_wstr(&wrap)
            .ok_or_else(|| make_error_2008(activation, "wrap"))?;

        let filter = Context3DTextureFilter::from_wstr(&filter)
            .ok_or_else(|| make_error_2008(activation, "filter"))?;

        if matches!(
            filter,
            Context3DTextureFilter::Anisotropic2X
                | Context3DTextureFilter::Anisotropic4X
                | Context3DTextureFilter::Anisotropic8X
                | Context3DTextureFilter::Anisotropic16X
        ) {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "setSamplerStateAt",
                "filter == 'anisotropic'"
            );
        }

        if &*mip_filter != b"mipnone" {
            avm2_stub_method!(
                activation,
                "flash.display3D.Context3D",
                "setSamplerStateAt",
                "mipFilter != 'none'"
            );
        }

        context.set_sampler_state_at(sampler, wrap, filter);
    }
    Ok(Value::Undefined)
}

pub fn set_scissor_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let context3d = this.as_context_3d().unwrap();
    let rectangle = args.try_get_object(activation, 0);
    let rectangle = if let Some(rectangle) = rectangle {
        let x = rectangle
            .get_public_property("x", activation)?
            .coerce_to_number(activation)?;
        let y = rectangle
            .get_public_property("y", activation)?
            .coerce_to_number(activation)?;
        let width = rectangle
            .get_public_property("width", activation)?
            .coerce_to_number(activation)?;
        let height = rectangle
            .get_public_property("height", activation)?
            .coerce_to_number(activation)?;
        Some(Rectangle {
            x_min: Twips::from_pixels(x),
            y_min: Twips::from_pixels(y),
            x_max: Twips::from_pixels(x + width),
            y_max: Twips::from_pixels(y + height),
        })
    } else {
        None
    };

    context3d.set_scissor_rectangle(rectangle);
    Ok(Value::Undefined)
}

pub fn dispose<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.display3D.Context3D", "dispose");
    this.as_context_3d()
        .unwrap()
        .stage3d()
        .set_context3d(None, activation.context.gc_context);
    Ok(Value::Undefined)
}
