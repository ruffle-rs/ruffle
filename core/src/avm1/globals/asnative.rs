use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{FunctionObject, NativeFunction};
use crate::avm1::parameters::ParametersExt;
use crate::avm1::{Object, Value};

pub fn asnative<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() != 2 {
        return Ok(Value::Undefined);
    }
    let category = args.get_u32(activation, 0)?;
    let index = args.get_u32(activation, 1)?;

    let category_lookup: Option<fn(u32) -> Option<NativeFunction>> = match category {
        100 => Some(crate::avm1::globals::get_native_function),
        101 => Some(crate::avm1::globals::object::get_native_function),
        _ => None,
    };

    if let Some(lookup) = category_lookup {
        let function = lookup(index)
            .map(FunctionObject::native)
            .unwrap_or_else(FunctionObject::empty);
        return Ok(function
            .build(
                &activation.context.strings,
                activation.prototypes().function,
                None,
            )
            .into());
    }

    Ok(Value::Undefined)
}
