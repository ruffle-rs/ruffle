use crate::avm2::object::{ClassObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::Activation;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

pub fn index_buffer_3d_allocator<'gc>(
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
    if let Some(index_buffer) = this.as_index_buffer() {
        let byte_array = args.get_object(activation, 0, "byteArray")?;
        let byte_array = byte_array
            .as_bytearray()
            .expect("Parameter must be a ByteArray");

        let byte_offset = args.get_u32(activation, 1)?;
        let start_offset = args.get_u32(activation, 2)?;
        let count = args.get_u32(activation, 3)?;

        let data = byte_array
            // Each index is always 16 bits (2 bytes)
            .read_at(count as usize * 2, byte_offset as usize)
            .map_err(|e| e.to_avm(activation))?
            .to_vec();

        index_buffer.context3d().upload_index_buffer_data(
            index_buffer,
            data,
            start_offset as usize,
        );
    }
    Ok(Value::Undefined)
}

pub fn upload_from_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(index_buffer) = this.as_index_buffer() {
        let vector = args.get_object(activation, 0, "data")?;

        let vector = vector
            .as_vector_storage()
            .expect("Parameter must be a Vector");

        let start_offset = args.get_u32(activation, 1)?;
        let count = args.get_u32(activation, 2)?;

        index_buffer.set_count(count as usize);

        let data: Result<Vec<u16>, _> = vector
            .iter()
            .map(|val| {
                // FIXME - use the low 16 bytes
                val.coerce_to_u32(activation).map(|val| val as u16)
            })
            .take(count as usize)
            .collect();

        let data_bytes = bytemuck::cast_slice::<u16, u8>(data?.as_slice()).to_vec();

        index_buffer.context3d().upload_index_buffer_data(
            index_buffer,
            data_bytes,
            start_offset as usize,
        );
    }
    Ok(Value::Undefined)
}
