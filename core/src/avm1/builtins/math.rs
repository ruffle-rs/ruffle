use crate::avm1::{Object, Value};
use gc_arena::{GcCell, MutationContext};

fn abs<'gc>(args: &[Value<'gc>]) -> Value<'gc> {
    let input = args.get(0).unwrap().as_f64().unwrap();
    Value::Number(input.abs())
}

fn round<'gc>(args: &[Value<'gc>]) -> Value<'gc> {
    let input = args.get(0).unwrap().as_f64().unwrap();
    Value::Number(input.round())
}

pub fn create<'gc>(gc_context: MutationContext<'gc, '_>) -> Value<'gc> {
    let mut math = Object::new();

    math.set("abs", Value::NativeFunction(abs));
    math.set("round", Value::NativeFunction(round));

    Value::Object(GcCell::allocate(gc_context, math))
}
