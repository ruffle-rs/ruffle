use crate::avm2::activation::Activation;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};

pub fn upload<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_program_3d() {
        let vertex_agal = args.get_object(activation, 0, "source_vertex")?;
        let vertex_agal = vertex_agal
            .as_bytearray()
            .expect("Parameter must be a ByteArray");
        let vertex_agal = vertex_agal.bytes().to_vec();

        let fragment_agal = args.get_object(activation, 1, "source_fragment")?;
        let fragment_agal = fragment_agal
            .as_bytearray()
            .expect("Parameter must be a ByteArray");
        let fragment_agal = fragment_agal.bytes().to_vec();

        this.context3d()
            .upload_shaders(this, vertex_agal, fragment_agal);
    }
    Ok(Value::Undefined)
}
