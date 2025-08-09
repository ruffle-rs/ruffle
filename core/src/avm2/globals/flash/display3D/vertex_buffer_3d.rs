use crate::avm2::activation::Activation;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::Error;
use crate::avm2::Value;

pub fn upload_from_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(vertex_buffer) = this.as_vertex_buffer() {
        let byte_array = args.get_object(activation, 0, "data")?;

        let byte_array = byte_array
            .as_bytearray()
            .expect("Parameter must be a ByteArray");

        let byte_offset = args.get_u32(1);
        let start_vertex = args.get_u32(2);
        let num_vertices = args.get_u32(3);

        let data = byte_array
            .read_at(
                num_vertices as usize * 4 * vertex_buffer.data32_per_vertex() as usize,
                byte_offset as usize,
            )
            .map_err(|e| e.to_avm(activation))?;

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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(vertex_buffer) = this.as_vertex_buffer() {
        let vector = args.get_object(activation, 0, "data")?;

        let vector = vector
            .as_vector_storage()
            .expect("Parameter must be a Vector");

        let start_vertex = args.get_u32(1);
        let num_vertices = args.get_u32(2);

        let data = vector
            .iter()
            .map(|val| val.coerce_to_number(activation).map(|val| val as f32))
            .take(num_vertices as usize * vertex_buffer.data32_per_vertex() as usize)
            .collect::<Result<Vec<f32>, _>>()?;

        let data_bytes = bytemuck::cast_slice::<f32, u8>(&data);

        vertex_buffer.context3d().upload_vertex_buffer_data(
            vertex_buffer,
            data_bytes,
            start_vertex as usize,
            vertex_buffer.data32_per_vertex(),
        );
    }
    Ok(Value::Undefined)
}
