//! Global scope built-ins

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::NativeMethod;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{FunctionObject, Object, ScriptObject, TObject};
use crate::avm2::r#trait::Trait;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
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

/// Add an ES3-style builtin to the global scope.
fn oldstyle_class<'gc>(
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

/// Add a class builtin to the global scope.
fn class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    mut global_scope: Object<'gc>,
    class_def: GcCell<'gc, Class<'gc>>,
) -> Result<(), Error> {
    let class_trait = Trait::from_class(class_def);

    global_scope.install_trait(activation, class_trait, global_scope)
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
///
/// Due to a limitation of our type system and our garbage collector, the
/// player needs a valid `Avm2` but cannot provide us an `UpdateContext` yet.
/// As a result, global scope initialization is split into an "oldstyle phase"
/// and a "player-globals phase". This is the former phase, where we initialize
/// as much as we can without an `UpdateContext`. Note that not all
/// `SystemPrototypes` will be necessarily valid at this point in time, and
/// using them right away will result in objects of the wrong type.
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

    oldstyle_class(
        mc,
        gs,
        "",
        "Object",
        object::constructor,
        object_proto,
        fn_proto,
    );
    oldstyle_class(
        mc,
        gs,
        "",
        "Function",
        function::constructor,
        fn_proto,
        fn_proto,
    );
    oldstyle_class(
        mc,
        gs,
        "",
        "Class",
        class::constructor,
        class_proto,
        fn_proto,
    );
    oldstyle_class(
        mc,
        gs,
        "",
        "String",
        string::constructor,
        string_proto,
        fn_proto,
    );
    oldstyle_class(
        mc,
        gs,
        "",
        "Boolean",
        boolean::constructor,
        boolean_proto,
        fn_proto,
    );
    oldstyle_class(
        mc,
        gs,
        "",
        "Number",
        number::constructor,
        number_proto,
        fn_proto,
    );
    oldstyle_class(mc, gs, "", "int", int::constructor, int_proto, fn_proto);
    oldstyle_class(mc, gs, "", "uint", uint::constructor, uint_proto, fn_proto);
    oldstyle_class(
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

/// Initialize all remaining builtin classes.
///
/// Due to a limitation of our type system and our garbage collector, the
/// player needs a valid `Avm2` but cannot provide us an `UpdateContext` yet.
/// As a result, global scope initialization is split into an "oldstyle phase"
/// and a "player-globals phase". This is the latter phase.
pub fn load_player_globals<'gc>(activation: &mut Activation<'_, 'gc, '_>) -> Result<(), Error> {
    let gs = activation.avm2().globals();

    // package `flash.events`
    class(
        activation,
        gs,
        flash::events::eventdispatcher::create_class(activation.context.gc_context),
    )?;

    // package `flash.display`
    class(
        activation,
        gs,
        flash::display::displayobject::create_class(activation.context.gc_context),
    )?;
    class(
        activation,
        gs,
        flash::display::interactiveobject::create_class(activation.context.gc_context),
    )?;
    class(
        activation,
        gs,
        flash::display::displayobjectcontainer::create_class(activation.context.gc_context),
    )?;
    class(
        activation,
        gs,
        flash::display::sprite::create_class(activation.context.gc_context),
    )?;
    class(
        activation,
        gs,
        flash::display::movieclip::create_class(activation.context.gc_context),
    )?;

    Ok(())
}
