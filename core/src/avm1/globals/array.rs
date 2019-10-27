//! Array prototype

use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ObjectCell, ScriptObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};

/// Implements `Array`'s callable.
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: ObjectCell<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let mut this_mutref = this.write(context.gc_context);

    if let Value::Number(num) = args[0] {
        if args.len() == 1 {
            //TODO: Error out on noninteger num
            let true_length = num.floor();
            this_mutref.define_value("length", Value::Number(true_length), EnumSet::empty());

            for i in 0..(true_length as usize) {
                this_mutref.define_value(&format!("{}", i), Value::Undefined, EnumSet::empty());
            }

            return Ok(Value::Undefined.into());
        }
    }

    this_mutref.define_value("length", Value::Number(args.len() as f64), EnumSet::empty());

    for (i, arg) in args.iter().enumerate() {
        this_mutref.define_value(&format!("{}", i), arg.clone(), EnumSet::empty());
    }

    Ok(Value::Undefined.into())
}

/// Implements `Array.prototype.toString`
fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: ObjectCell<'gc>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let length = this
        .read()
        .get("length", avm, ac, this)?
        .resolve(avm, ac)?
        .as_number() as u64;
    let mut result = "".to_owned();

    for i in 0..length {
        let value_str = this
            .read()
            .get(&format!("{}", i), avm, ac, this)?
            .resolve(avm, ac)?
            .as_string()?
            .to_owned();

        if i != 0 {
            result = format!("{},{}", result, value_str);
        } else {
            result.push_str(&value_str);
        }
    }

    Ok(ReturnValue::Immediate(result.into()))
}

/// Create `Array.prototype` and fill it with it's built-in methods.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    obj_proto: ObjectCell<'gc>,
    fn_proto: ObjectCell<'gc>,
) -> ObjectCell<'gc> {
    let mut array_proto = ScriptObject::object(gc_context, Some(obj_proto));

    array_proto.force_set_function(
        "toString",
        to_string,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    GcCell::allocate(gc_context, Box::new(array_proto))
}
