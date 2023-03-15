use ruffle_render::backend::BufferUsage;
use ruffle_render::backend::Context3DBlendFactor;
use ruffle_render::backend::Context3DCompareMode;
use ruffle_render::backend::Context3DTextureFormat;
use ruffle_render::backend::Context3DTriangleFace;
use ruffle_render::backend::Context3DVertexBufferFormat;
use ruffle_render::backend::ProgramType;

use crate::avm2::parameters::ParametersExt;
use crate::avm2::Activation;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

pub fn create_index_buffer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // FIXME - get bufferUsage and pass it through
        let num_indices = args.get_u32(activation, 0)?;
        return context.create_index_buffer(num_indices, activation);
    }
    Ok(Value::Undefined)
}

pub fn create_vertex_buffer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
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
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut context) = this.and_then(|this| this.as_context_3d()) {
        let width = args.get_u32(activation, 0)?;
        let height = args.get_u32(activation, 1)?;

        let anti_alias = args.get_u32(activation, 2)?;
        let enable_depth_and_stencil = args.get(3).unwrap_or(&Value::Undefined).coerce_to_boolean();
        let wants_best_resolution = args.get(4).unwrap_or(&Value::Undefined).coerce_to_boolean();
        let wants_best_resolution_on_browser_zoom =
            args.get(5).unwrap_or(&Value::Undefined).coerce_to_boolean();

        context.configure_back_buffer(
            activation,
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
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let index = args.get_u32(activation, 0)?;
        let buffer = if matches!(args[1], Value::Null) {
            None
        } else {
            Some(
                args.get(1)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_object(activation)?
                    .as_vertex_buffer()
                    .unwrap(),
            )
        };

        let buffer_offset = args.get_u32(activation, 2)?;

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
            panic!("Unknown vertex format {format:?}");
        };

        context.set_vertex_buffer_at(index, buffer, buffer_offset, format, activation);
    }
    Ok(Value::Undefined)
}

pub fn create_program<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        return context.create_program(activation);
    }
    Ok(Value::Undefined)
}

pub fn set_program<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let program = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?
            .as_program_3d()
            .unwrap();

        context.set_program(activation, program);
    }
    Ok(Value::Undefined)
}

pub fn draw_triangles<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let index_buffer = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?
            .as_index_buffer()
            .unwrap();

        let first_index = args.get_u32(activation, 1)?;
        let num_triangles = args.get_u32(activation, 2)? as i32;

        context.draw_triangles(activation, index_buffer, first_index, num_triangles);
    }
    Ok(Value::Undefined)
}

pub fn present<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        context.present(activation)?;
    }
    Ok(Value::Undefined)
}

pub fn set_culling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let culling = args.get_string(activation, 0)?;

        let culling = if &*culling == b"none" {
            Context3DTriangleFace::None
        } else if &*culling == b"back" {
            Context3DTriangleFace::Back
        } else if &*culling == b"front" {
            Context3DTriangleFace::Front
        } else if &*culling == b"front_and_back" {
            Context3DTriangleFace::FrontAndBack
        } else {
            tracing::error!("Unknown culling {:?}", culling);
            Context3DTriangleFace::None
        };

        context.set_culling(activation, culling);
    }
    Ok(Value::Undefined)
}

pub fn set_program_constants_from_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let program_type = args.get_string(activation, 0)?;

        let is_vertex = if &*program_type == b"vertex" {
            ProgramType::Vertex
        } else if &*program_type == b"fragment" {
            ProgramType::Fragment
        } else {
            panic!("Unknown program type {program_type:?}");
        };

        let first_register = args.get_u32(activation, 1)?;

        let mut matrix = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let user_transposedMatrix = args.get(3).unwrap_or(&Value::Undefined).coerce_to_boolean();

        // Hack - we store in column-major form, but we need it in row-major form
        // So, do the *opposite* of what the user pasess in`
        // or that's what I thought, but doing this seems to work???
        //
        // It seems like the documentation is wrong - we really copy to the registers
        // in column-major order.
        // See https://github.com/openfl/openfl/blob/971a4c9e43b5472fd84d73920a2b7c1b3d8d9257/src/openfl/display3D/Context3D.hx#L1532-L1550
        if user_transposedMatrix {
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
            .map(|val| val.coerce_to_number(activation).map(|val| val as f32))
            .collect::<Result<Vec<f32>, Error>>()?;

        context.set_program_constants_from_matrix(
            activation,
            is_vertex,
            first_register,
            matrix_raw_data,
        );
    }
    Ok(Value::Undefined)
}

pub fn set_program_constants_from_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let program_type = args.get_string(activation, 0)?;

        let program_type = if &*program_type == b"vertex" {
            ProgramType::Vertex
        } else if &*program_type == b"fragment" {
            ProgramType::Fragment
        } else {
            panic!("Unknown program type {:?}", program_type);
        };

        let first_register = args.get_u32(activation, 1)?;

        let vector = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let vector = vector.as_vector_storage().unwrap();

        let num_registers = args.get_i32(activation, 3)?;

        let to_take = if num_registers != -1 {
            // Each register requries 4 floating-point values
            // FIXME - throw an error if 'vector' is too small
            num_registers as usize * 4
        } else {
            vector.length()
        };

        let raw_data = vector
            .iter()
            .map(|val| {
                val.as_number(activation.context.gc_context)
                    .map(|val| val as f32)
            })
            .take(to_take)
            .collect::<Result<Vec<f32>, _>>()?;

        context.set_program_constants_from_matrix(
            activation,
            program_type,
            first_register,
            raw_data,
        );
    }
    Ok(Value::Undefined)
}

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let red = args[0].as_number(activation.context.gc_context)?;
        let green = args[1].as_number(activation.context.gc_context)?;
        let blue = args[2].as_number(activation.context.gc_context)?;
        let alpha = args[3].as_number(activation.context.gc_context)?;
        let depth = args[4].as_number(activation.context.gc_context)?;
        let stencil = args[5].as_integer(activation.context.gc_context)? as u32;
        let mask = args[6].as_integer(activation.context.gc_context)? as u32;
        context.set_clear(activation, red, green, blue, alpha, depth, stencil, mask);
    }
    Ok(Value::Undefined)
}

pub fn create_texture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let width = args[0].as_integer(activation.context.gc_context)? as u32;
        let height = args[1].as_integer(activation.context.gc_context)? as u32;
        let format = args[2].coerce_to_string(activation)?;
        let optimize_for_render_to_texture = args[3].coerce_to_boolean();
        let streaming_levels = args[4].as_integer(activation.context.gc_context)? as u32;
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
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let width = args[0].as_integer(activation.context.gc_context)? as u32;
        let height = args[1].as_integer(activation.context.gc_context)? as u32;
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
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let size = args[0].as_integer(activation.context.gc_context)? as u32;
        let format = args[1].coerce_to_string(activation)?;
        let optimize_for_render_to_texture = args[2].coerce_to_boolean();
        let streaming_levels = args[3].as_integer(activation.context.gc_context)? as u32;
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
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let sampler = args[0].as_integer(activation.context.gc_context)? as u32;
        let mut cube = false;
        let texture = if matches!(args[1], Value::Null) {
            None
        } else {
            let obj = args[1].coerce_to_object(activation)?;
            cube = obj.is_of_type(activation.avm2().classes().cubetexture, activation);
            Some(obj.as_texture().unwrap().handle())
        };
        context.set_texture_at(activation, sampler, texture, cube);
    }
    Ok(Value::Undefined)
}

pub fn set_depth_test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // This is a native method, so all of the arguments have been checked and coerced for us
        let depth_mask = args[0].coerce_to_boolean();
        let pass_compare_mode = args[1].coerce_to_string(activation)?;
        let pass_compare_mode =
            if let Some(mode) = Context3DCompareMode::from_wstr(&pass_compare_mode) {
                mode
            } else {
                return Err(format!("Unsupported depth test mode: {:?}", pass_compare_mode).into());
            };
        context.set_depth_test(activation, depth_mask, pass_compare_mode);
    }
    Ok(Value::Undefined)
}

pub fn set_blend_factors<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
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
        context.set_blend_factors(activation, source_factor, destination_factor);
    }
    Ok(Value::Undefined)
}
