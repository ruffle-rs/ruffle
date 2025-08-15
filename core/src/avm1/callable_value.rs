use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, Value};
use crate::string::AvmString;
use gc_arena::Collect;

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum CallableValue<'gc> {
    UnCallable(Value<'gc>),
    Callable(Object<'gc>, Value<'gc>),
}

impl<'gc> From<CallableValue<'gc>> for Value<'gc> {
    fn from(c: CallableValue<'gc>) -> Self {
        match c {
            CallableValue::UnCallable(v) => v,
            CallableValue::Callable(_, v) => v,
        }
    }
}

impl<'gc> CallableValue<'gc> {
    pub fn call_with_default_this(
        self,
        default_this: Object<'gc>,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        match self {
            CallableValue::Callable(this, Value::Object(val)) => {
                val.call(name, activation, this.into(), args)
            }
            CallableValue::UnCallable(Value::Object(val)) => {
                val.call(name, activation, default_this.into(), args)
            }
            _ => Ok(Value::Undefined),
        }
    }
}
