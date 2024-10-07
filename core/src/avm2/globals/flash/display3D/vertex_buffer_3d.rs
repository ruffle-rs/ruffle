use crate::avm2::activation::Activation;
use crate::avm2::object::{ClassObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

pub fn vertex_buffer_3d_allocator<'gc>(
    _class: ClassObject<'gc>,
    _activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    panic!("This allocator should not be called!")
}

pub fn upload_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vertex_buffer) = this.as_vertex_buffer() {
        let byte_array = args.get_object(activation, 0, "data")?;

        let byte_array = byte_array
            .as_bytearray()
            .expect("Parameter must be a ByteArray");

        let byte_offset = args.get_u32(activation, 1)?;
        let start_vertex = args.get_u32(activation, 2)?;
        let num_vertices = args.get_u32(activation, 3)?;

        let data = byte_array
            .read_at(
                num_vertices as usize * 4 * vertex_buffer.data32_per_vertex() as usize,
                byte_offset as usize,
            )
            .map_err(|e| e.to_avm(activation))?
            .to_vec();

        vertex_buffer.context3d().upload_vertex_buffer_data(
            vertex_buffer,
            data,
            start_vertex as usize,
            vertex_buffer.data32_per_vertex(),
        );
    }
    Ok(Value::Undefined)
}

pub fn upload_from_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vertex_buffer) = this.as_vertex_buffer() {
        let vector = args.get_object(activation, 0, "data")?;

        let vector = vector
            .as_vector_storage()
            .expect("Parameter must be a Vector");

        let start_vertex = args.get_u32(activation, 1)?;
        let num_vertices = args.get_u32(activation, 2)?;

        let data: Result<Vec<f32>, _> = vector
            .iter()
            .map(|val| val.coerce_to_number(activation).map(|val| val as f32))
            .take(num_vertices as usize * vertex_buffer.data32_per_vertex() as usize)
            .collect();

        let data_bytes = bytemuck::cast_slice::<f32, u8>(data?.as_slice()).to_vec();

        vertex_buffer.context3d().upload_vertex_buffer_data(
            vertex_buffer,
            data_bytes,
            start_vertex as usize,
            vertex_buffer.data32_per_vertex(),
        );
    }
    Ok(Value::Undefined)
}
