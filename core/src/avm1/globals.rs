use crate::avm1::object::Object;
use gc_arena::MutationContext;

mod math;

pub fn create_globals<'gc>(gc_context: MutationContext<'gc, '_>) -> Object<'gc> {
    let mut globals = Object::object(gc_context);

    globals.set_object("Math", math::create(gc_context));

    globals
}
