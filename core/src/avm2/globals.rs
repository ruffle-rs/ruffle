//! Global scope built-ins

use crate::avm2::function::FunctionObject;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::script_object::ScriptObject;
use gc_arena::{Collect, MutationContext};

mod flash;
mod function;
mod object;

/// This structure represents all system builtins' prototypes.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SystemPrototypes<'gc> {
    pub object: Object<'gc>,
    pub function: Object<'gc>,
}

/// Construct a new global scope.
///
/// This function returns both the global scope object, as well as all builtin
/// prototypes that other parts of the VM will need to use.
pub fn construct_global_scope<'gc>(
    mc: MutationContext<'gc, '_>,
) -> (Object<'gc>, SystemPrototypes<'gc>) {
    let mut global_scope = ScriptObject::bare_object(mc);

    let object_proto = ScriptObject::bare_object(mc);
    let function_proto = function::create_proto(mc, object_proto);
    let sprite_proto = flash::display::sprite::create_proto(mc, object_proto, function_proto);
    let movieclip_proto = flash::display::movieclip::create_proto(mc, sprite_proto, function_proto);

    object::fill_proto(mc, object_proto, function_proto);

    let system_prototypes = SystemPrototypes {
        object: object_proto,
        function: function_proto,
    };

    global_scope
        .install_dynamic_property(
            mc,
            QName::new(Namespace::public_namespace(), "Object"),
            FunctionObject::from_builtin_constr(
                mc,
                object::constructor,
                object_proto,
                function_proto,
            )
            .unwrap()
            .into(),
        )
        .unwrap();
    global_scope
        .install_dynamic_property(
            mc,
            QName::new(Namespace::public_namespace(), "Function"),
            FunctionObject::from_builtin_constr(
                mc,
                function::constructor,
                function_proto,
                function_proto,
            )
            .unwrap()
            .into(),
        )
        .unwrap();
    global_scope
        .install_dynamic_property(
            mc,
            QName::new(Namespace::package("flash.display"), "Sprite"),
            FunctionObject::from_builtin_constr(
                mc,
                flash::display::sprite::constructor,
                sprite_proto,
                function_proto,
            )
            .unwrap()
            .into(),
        )
        .unwrap();
    global_scope
        .install_dynamic_property(
            mc,
            QName::new(Namespace::package("flash.display"), "MovieClip"),
            FunctionObject::from_builtin_constr(
                mc,
                flash::display::movieclip::constructor,
                movieclip_proto,
                function_proto,
            )
            .unwrap()
            .into(),
        )
        .unwrap();

    (global_scope, system_prototypes)
}
