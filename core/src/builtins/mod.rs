use crate::avm1::Value;
use gc_arena::MutationContext;
use std::collections::HashMap;

mod math;
mod movie_clip;
mod object;

pub use movie_clip::create_movie_clip;
pub use object::Object;

pub fn register_builtins<'gc>(
    gc_context: MutationContext<'gc, '_>,
    globals: &mut HashMap<String, Value<'gc>>,
) {
    globals.insert("Math".to_string(), math::create(gc_context));
}
