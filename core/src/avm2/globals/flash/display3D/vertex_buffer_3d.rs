use crate::avm2::Activation;
use crate::avm2::ClassObject;
use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

pub fn vertex_buffer_3d_allocator<'gc>(
    _class: ClassObject<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error<'gc>> {
    panic!("This allocator should not be called!")
}

pub fn upload_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vertex_buffer) = this.and_then(|this| this.as_vertex_buffer()) {
        let byte_array = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let byte_array = byte_array
            .as_bytearray()
            .ok_or_else(|| Error::from("ArgumentError: Parameter must be a ByteArray"))?;

        let byte_offset = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let start_vertex = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let num_vertices = args
            .get(3)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;

        let data = byte_array
            .read_at(num_vertices as usize * 2, byte_offset as usize)?
            .to_vec();

        vertex_buffer.context3d().upload_vertex_buffer_data(
            vertex_buffer,
            data,
            start_vertex as usize,
            vertex_buffer.data_per_vertex(),
            activation,
        );
    }
    Ok(Value::Undefined)
}

pub fn upload_from_vector<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vertex_buffer) = this.and_then(|this| this.as_vertex_buffer()) {
        let vector = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let vector = vector
            .as_vector_storage()
            .ok_or_else(|| Error::from("ArgumentError: Parameter must be a Vector"))?;

        let start_vertex = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let num_vertices = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;

        let data: Result<Vec<f32>, _> = vector
            .iter()
            .map(|val| val.coerce_to_number(activation).map(|val| val as f32))
            .take(num_vertices as usize * vertex_buffer.data_per_vertex())
            .collect();

        let data_bytes = bytemuck::cast_slice::<f32, u8>(data?.as_slice()).to_vec();

        vertex_buffer.context3d().upload_vertex_buffer_data(
            vertex_buffer,
            data_bytes,
            start_vertex as usize,
            vertex_buffer.data_per_vertex(),
            activation,
        );
    }
    Ok(Value::Undefined)
}
