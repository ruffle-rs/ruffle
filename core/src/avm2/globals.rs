//! Global scope built-ins

use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionObject;
use crate::avm2::method::NativeMethod;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::script_object::ScriptObject;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, MutationContext};
use std::f64::NAN;

mod boolean;
mod class;
mod flash;
mod function;
mod int;
mod namespace;
mod number;
mod object;
mod string;
mod r#uint;

fn trace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(s) = args.get(0) {
        log::info!(target: "avm_trace", "{}", s.clone().coerce_to_string(activation)?);
    }

    Ok(Value::Undefined)
}

/// This structure represents all system builtins' prototypes.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SystemPrototypes<'gc> {
    pub object: Object<'gc>,
    pub function: Object<'gc>,
    pub class: Object<'gc>,
    pub string: Object<'gc>,
    pub boolean: Object<'gc>,
    pub number: Object<'gc>,
    pub int: Object<'gc>,
    pub uint: Object<'gc>,
    pub namespace: Object<'gc>,
}

/// Add a free-function builtin to the global scope.
fn function<'gc>(
    mc: MutationContext<'gc, '_>,
    mut global_scope: Object<'gc>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    nf: NativeMethod<'gc>,
    fn_proto: Object<'gc>,
) {
    global_scope
        .install_dynamic_property(
            mc,
            QName::new(Namespace::package(package), name),
            FunctionObject::from_builtin(mc, nf, fn_proto).into(),
        )
        .unwrap()
}

/// Add a class builtin to the global scope.
fn class<'gc>(
    mc: MutationContext<'gc, '_>,
    mut global_scope: Object<'gc>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    constr: NativeMethod<'gc>,
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

/// Add a builtin constant to the global scope.
fn constant<'gc>(
    mc: MutationContext<'gc, '_>,
    mut global_scope: Object<'gc>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    value: Value<'gc>,
) {
    global_scope.install_const(mc, QName::new(Namespace::package(package), name), 0, value)
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
    let class_proto = class::create_proto(mc, object_proto, fn_proto);
    let string_proto = string::create_proto(mc, object_proto, fn_proto);
    let boolean_proto = boolean::create_proto(mc, object_proto, fn_proto);
    let number_proto = number::create_proto(mc, object_proto, fn_proto);
    let int_proto = int::create_proto(mc, object_proto, fn_proto);
    let uint_proto = uint::create_proto(mc, object_proto, fn_proto);
    let namespace_proto = namespace::create_proto(mc, object_proto, fn_proto);

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
    class(
        mc,
        gs,
        "",
        "Class",
        class::constructor,
        class_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "",
        "String",
        string::constructor,
        string_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "",
        "Boolean",
        boolean::constructor,
        boolean_proto,
        fn_proto,
    );
    class(
        mc,
        gs,
        "",
        "Number",
        number::constructor,
        number_proto,
        fn_proto,
    );
    class(mc, gs, "", "int", int::constructor, int_proto, fn_proto);
    class(mc, gs, "", "uint", uint::constructor, uint_proto, fn_proto);
    class(
        mc,
        gs,
        "",
        "Namespace",
        namespace::constructor,
        namespace_proto,
        fn_proto,
    );
    function(mc, gs, "", "trace", trace, fn_proto);
    constant(mc, gs, "", "undefined", Value::Undefined);
    constant(mc, gs, "", "null", Value::Null);
    constant(mc, gs, "", "NaN", NAN.into());
    constant(mc, gs, "", "Infinity", f64::INFINITY.into());

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
        class: class_proto,
        string: string_proto,
        boolean: boolean_proto,
        number: number_proto,
        int: int_proto,
        uint: uint_proto,
        namespace: namespace_proto,
    };

    (gs, system_prototypes)
}
