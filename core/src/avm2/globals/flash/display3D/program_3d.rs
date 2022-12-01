use crate::avm2::Activation;

use crate::avm2::TObject;
use crate::avm2::Value;
use crate::avm2::{Error, Object};

pub fn upload<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|this| this.as_program_3d()) {
        let vertex_agal = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let vertex_agal = vertex_agal
            .as_bytearray()
            .ok_or_else(|| Error::from("ArgumentError: Parameter must be a ByteArray"))?;
        let vertex_agal = vertex_agal.bytes().to_vec();

        let fragment_agal = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;
        let fragment_agal = fragment_agal
            .as_bytearray()
            .ok_or_else(|| Error::from("ArgumentError: Parameter must be a ByteArray"))?;
        let fragment_agal = fragment_agal.bytes().to_vec();

        this.context3d()
            .upload_shaders(activation, this, vertex_agal, fragment_agal);
    }
    Ok(Value::Undefined)
}
