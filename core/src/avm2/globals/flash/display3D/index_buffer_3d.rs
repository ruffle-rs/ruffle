use crate::avm2::object::{ClassObject, TObject};
use crate::avm2::Activation;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

pub fn index_buffer_3d_allocator<'gc>(
    _class: ClassObject<'gc>,
    _activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error<'gc>> {
    panic!("This allocator should not be called!")
}

pub fn upload_from_byte_array<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("IndexBuffer3D.uploadFromByteArray - not yet implemented");
    Ok(Value::Undefined)
}

pub fn upload_from_vector<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(index_buffer) = this.and_then(|this| this.as_index_buffer()) {
        let vector = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let vector = vector
            .as_vector_storage()
            .ok_or_else(|| Error::from("ArgumentError: Parameter must be a Vector"))?;

        let start_offset = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let count = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;

        index_buffer.set_count(count as usize, activation.context.gc_context);

        if start_offset != 0 {
            panic!("What exactly does start_offset do?");
        }

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
            activation,
        );
    }
    Ok(Value::Undefined)
}
