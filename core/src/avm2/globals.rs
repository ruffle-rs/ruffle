//! Global scope built-ins

use crate::avm2::object::{Object, TObject};
use crate::avm2::script_object::ScriptObject;
use gc_arena::MutationContext;

pub fn construct_global_scope<'gc>(mc: MutationContext<'gc, '_>) -> Object<'gc> {
    let global_scope = ScriptObject::bare_object(mc);

    global_scope
}
