use ruffle_render::backend::BufferUsage;
use ruffle_render::backend::Context3DTriangleFace;
use ruffle_render::backend::Context3DVertexBufferFormat;
use ruffle_render::backend::ProgramType;

use crate::avm2::Activation;
use crate::avm2::Multiname;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

pub fn create_index_buffer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // FIXME - get bufferUsage and pass it through
        let num_indices = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        return context.create_index_buffer(num_indices, activation);
    }
    Ok(Value::Undefined)
}

pub fn create_vertex_buffer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        // FIXME - get bufferUsage and pass it through
        let num_vertices = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let data_per_vertex = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        return context.create_vertex_buffer(
            num_vertices,
            data_per_vertex,
            BufferUsage::DynamicDraw,
            activation,
        );
    }
    Ok(Value::Undefined)
}

pub fn configure_back_buffer<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut context) = this.and_then(|this| this.as_context_3d()) {
        let width = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let height = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        // FIXME - get other parameters

        //let anti_alias = args.get(2).unwrap_or(&Value::Undefined).coerce_to_u32(activation)?;
        //let enable_depth_and_stencil = args.get(3).unwrap_or(&Value::Undefined).coerce_to_boolean();

        context.configure_back_buffer(activation, width, height, 0, true, true, true);
    }
    Ok(Value::Undefined)
}

pub fn set_vertex_buffer_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let index = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let buffer = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let buffer_offset = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;

        // FIXME - use the format
        let format = args
            .get(3)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

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
            panic!("Unknown vertex format {:?}", format);
        };

        let buffer = buffer.as_vertex_buffer().unwrap();
        context.set_vertex_buffer_at(index, buffer, buffer_offset, format, activation);
    }
    Ok(Value::Undefined)
}

pub fn create_program<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        return context.create_program(activation);
    }
    Ok(Value::Undefined)
}

pub fn set_program<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
    activation: &mut Activation<'_, 'gc, '_>,
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

        let first_index = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let num_triangles = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)? as i32;

        context.draw_triangles(activation, index_buffer, first_index, num_triangles);
    }
    Ok(Value::Undefined)
}

pub fn present<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        context.present(activation)?;
    }
    Ok(Value::Undefined)
}

pub fn set_culling<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let culling = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        let culling = if &*culling == b"none" {
            Context3DTriangleFace::None
        } else if &*culling == b"back" {
            Context3DTriangleFace::Back
        } else if &*culling == b"front" {
            Context3DTriangleFace::Front
        } else if &*culling == b"front_and_back" {
            Context3DTriangleFace::FrontAndBack
        } else {
            log::error!("Unknown culling {:?}", culling);
            Context3DTriangleFace::None
        };

        context.set_culling(activation, culling);
    }
    Ok(Value::Undefined)
}

pub fn set_program_constants_from_matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(context) = this.and_then(|this| this.as_context_3d()) {
        let program_type = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_string(activation)?;

        let is_vertex = if &*program_type == b"vertex" {
            ProgramType::Vertex
        } else if &*program_type == b"fragment" {
            ProgramType::Fragment
        } else {
            panic!("Unknown program type {:?}", program_type);
        };

        let first_register = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;

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
                .call_property(&Multiname::public("clone"), &[], activation)?
                .coerce_to_object(activation)?;

            matrix.call_property(&Multiname::public("transpose"), &[], activation)?;
        }

        let matrix_raw_data = matrix
            .get_property(&Multiname::public("rawData"), activation)?
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

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
