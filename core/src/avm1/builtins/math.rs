use crate::avm1::{Object, Value};
use gc_arena::{GcCell, MutationContext};

fn abs<'gc>(
    _gc_context: MutationContext<'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    let input = args.get(0).unwrap().as_f64().unwrap();
    Value::Number(input.abs())
}

fn round<'gc>(
    _gc_context: MutationContext<'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    let input = args.get(0).unwrap().as_f64().unwrap();
    Value::Number(input.round())
}

pub fn create<'gc>(gc_context: MutationContext<'gc, '_>) -> Value<'gc> {
    let mut math = Object::object();

    math.set_function("abs", abs, gc_context);
    math.set_function("round", round, gc_context);

    Value::Object(GcCell::allocate(gc_context, math))
}
