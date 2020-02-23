//! Global scope built-ins

use crate::avm2::function::{FunctionObject, NativeFunction};
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

fn class<'gc>(
    mc: MutationContext<'gc, '_>,
    mut global_scope: Object<'gc>,
    package: &str,
    name: &str,
    constr: NativeFunction<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) {
    global_scope
        .install_dynamic_property(
            mc,
            QName::new(Namespace::package(package), name),
            FunctionObject::from_builtin_constr(mc, constr, proto, fn_proto)
                .unwrap()
                .into(),
        )
        .unwrap();
}

/// Construct a new global scope.
///
/// This function returns both the global scope object, as well as all builtin
/// prototypes that other parts of the VM will need to use.
pub fn construct_global_scope<'gc>(
    mc: MutationContext<'gc, '_>,
) -> (Object<'gc>, SystemPrototypes<'gc>) {
    let gs = ScriptObject::bare_object(mc);

    // public / root package
    let object_proto = ScriptObject::bare_object(mc);
    let fn_proto = function::create_proto(mc, object_proto);

    object::fill_proto(mc, object_proto, fn_proto);

    class(
        mc,
        gs,
        "",
        "Object",
        object::constructor,
        object_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "",
        "Function",
        function::constructor,
        fn_proto,
        fn_proto,
    );

    // package `flash.events`
    let eventdispatcher_proto =
        flash::events::eventdispatcher::create_proto(mc, object_proto, fn_proto);

    class(
        mc,
        gs,
        "flash.events",
        "EventDispatcher",
        flash::events::eventdispatcher::constructor,
        eventdispatcher_proto,
        fn_proto,
    );

    // package `flash.display`
    let displayobject_proto =
        flash::display::displayobject::create_proto(mc, eventdispatcher_proto, fn_proto);
    let interactiveobject_proto =
        flash::display::interactiveobject::create_proto(mc, displayobject_proto, fn_proto);
    let displayobjectcontainer_proto =
        flash::display::displayobjectcontainer::create_proto(mc, interactiveobject_proto, fn_proto);
    let sprite_proto =
        flash::display::sprite::create_proto(mc, displayobjectcontainer_proto, fn_proto);
    let movieclip_proto = flash::display::movieclip::create_proto(mc, sprite_proto, fn_proto);

    class(
        mc,
        gs,
        "flash.display",
        "DisplayObject",
        flash::display::displayobject::constructor,
        displayobject_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "flash.display",
        "InteractiveObject",
        flash::display::interactiveobject::constructor,
        interactiveobject_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "flash.display",
        "DisplayObjectContainer",
        flash::display::displayobjectcontainer::constructor,
        sprite_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "flash.display",
        "Sprite",
        flash::display::sprite::constructor,
        sprite_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "flash.display",
        "MovieClip",
        flash::display::movieclip::constructor,
        movieclip_proto,
        fn_proto,
    );

    let system_prototypes = SystemPrototypes {
        object: object_proto,
        function: fn_proto,
    };

    (gs, system_prototypes)
}
