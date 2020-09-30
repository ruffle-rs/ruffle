//! Global scope built-ins

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::NativeMethod;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{
    implicit_deriver, ArrayObject, FunctionObject, NamespaceObject, Object, PrimitiveObject,
    ScriptObject, StageObject, TObject,
};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::f64::NAN;

mod array;
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
        let message = s.clone().coerce_to_string(activation)?;
        activation.context.log.avm_trace(&message);
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
    pub array: Object<'gc>,
    pub movieclip: Object<'gc>,
    pub framelabel: Object<'gc>,
    pub scene: Object<'gc>,
}

impl<'gc> SystemPrototypes<'gc> {
    /// Construct a minimal set of system prototypes necessary for
    /// bootstrapping player globals.
    ///
    /// All other system prototypes aside from the three given here will be set
    /// to the empty object also handed to this function. It is the caller's
    /// responsibility to instantiate each class and replace the empty object
    /// with that.
    fn new(
        object: Object<'gc>,
        function: Object<'gc>,
        class: Object<'gc>,
        empty: Object<'gc>,
    ) -> Self {
        SystemPrototypes {
            object,
            function,
            class,
            string: empty,
            boolean: empty,
            number: empty,
            int: empty,
            uint: empty,
            namespace: empty,
            array: empty,
            movieclip: empty,
            framelabel: empty,
            scene: empty,
        }
    }
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

/// Add a class builtin with prototype methods to the global scope.
///
/// Since the function has to return a normal prototype object in this case, we
/// have to construct a constructor to go along with it, as if we had called
/// `install_foreign_trait` with such a class.
fn dynamic_class<'gc>(
    mc: MutationContext<'gc, '_>,
    mut global_scope: Object<'gc>,
    constr: Object<'gc>,
) {
    let name = constr
        .as_class()
        .expect("constrs have classes in them")
        .read()
        .name()
        .clone();

    global_scope.install_const(mc, name, 0, constr.into());
}

/// Add a class builtin to the global scope.
///
/// This function returns a prototype which may be stored in `SystemPrototypes`.
/// The `custom_derive` is used to select a particular `TObject` impl, or you
/// can use `None` to indicate that this class does not change host object
/// impls.
fn class<'gc, Deriver>(
    activation: &mut Activation<'_, 'gc, '_>,
    mut global: Object<'gc>,
    class_def: GcCell<'gc, Class<'gc>>,
    custom_derive: Deriver,
) -> Result<Object<'gc>, Error>
where
    Deriver: FnOnce(
        Object<'gc>,
        &mut Activation<'_, 'gc, '_>,
        GcCell<'gc, Class<'gc>>,
        Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error>,
{
    let global_scope = Scope::push_scope(global.get_scope(), global, activation.context.gc_context);
    /*let mut constr = global
    .install_foreign_trait(activation, class_trait, Some(global_scope), global)?
    .coerce_to_object(activation)?;*/

    let class_read = class_def.read();
    let super_class = if let Some(sc_name) = class_read.super_class_name() {
        let super_name = global
            .resolve_multiname(sc_name)?
            .unwrap_or_else(|| QName::dynamic_name("Object"));

        let super_class: Result<Object<'gc>, Error> = global
            .get_property(global, &super_name, activation)?
            .coerce_to_object(activation)
            .map_err(|_e| {
                format!("Could not resolve superclass {:?}", super_name.local_name()).into()
            });

        Some(super_class?)
    } else {
        None
    };

    let (mut constr, _cinit) = FunctionObject::from_class_with_deriver(
        activation,
        class_def,
        super_class,
        Some(global_scope),
        custom_derive,
    )?;
    global.install_const(
        activation.context.gc_context,
        class_read.name().clone(),
        0,
        constr.into(),
    );

    constr
        .get_property(
            constr,
            &QName::new(Namespace::public_namespace(), "prototype"),
            activation,
        )?
        .coerce_to_object(activation)
}

fn primitive_deriver<'gc>(
    base_proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    PrimitiveObject::derive(base_proto, activation.context.gc_context, class, scope)
}

fn namespace_deriver<'gc>(
    base_proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    NamespaceObject::derive(base_proto, activation.context.gc_context, class, scope)
}

fn array_deriver<'gc>(
    base_proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    ArrayObject::derive(base_proto, activation.context.gc_context, class, scope)
}

fn stage_deriver<'gc>(
    base_proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    StageObject::derive(base_proto, activation.context.gc_context, class, scope)
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

/// Initialize all remaining builtin classes.
///
/// This should be called only once, to construct the global scope of the
/// player. It will return a list of prototypes it has created, which should be
/// stored on the AVM.
pub fn load_player_globals<'gc>(activation: &mut Activation<'_, 'gc, '_>) -> Result<(), Error> {
    let gs = activation.avm2().globals();

    // public / root package
    let object_proto = object::create_proto(activation);
    let (function_constr, fn_proto) = function::create_class(activation, object_proto);
    let (class_constr, class_proto) = class::create_class(activation, object_proto, fn_proto);

    let object_constr = object::fill_proto(activation.context.gc_context, object_proto, fn_proto);

    dynamic_class(activation.context.gc_context, gs, object_constr);
    dynamic_class(activation.context.gc_context, gs, function_constr);
    dynamic_class(activation.context.gc_context, gs, class_constr);

    // At this point, we need at least a partial set of system prototypes in
    // order to continue initializing the player. The rest of the prototypes
    // are set to a bare object until we have a chance to initialize them.
    activation.context.avm2.system_prototypes = Some(SystemPrototypes::new(
        object_proto,
        fn_proto,
        class_proto,
        ScriptObject::bare_object(activation.context.gc_context),
    ));

    // Even sillier: for the sake of clarity and the borrow checker we need to
    // clone the prototypes list and modify it outside of the activation. This
    // also has the side effect that none of these classes can get at each
    // other from the activation they're handed.
    let mut sp = activation.context.avm2.system_prototypes.clone().unwrap();

    sp.string = class(
        activation,
        gs,
        string::create_class(activation.context.gc_context),
        primitive_deriver,
    )?;
    sp.boolean = class(
        activation,
        gs,
        boolean::create_class(activation.context.gc_context),
        primitive_deriver,
    )?;
    sp.number = class(
        activation,
        gs,
        number::create_class(activation.context.gc_context),
        primitive_deriver,
    )?;
    sp.int = class(
        activation,
        gs,
        int::create_class(activation.context.gc_context),
        primitive_deriver,
    )?;
    sp.uint = class(
        activation,
        gs,
        uint::create_class(activation.context.gc_context),
        primitive_deriver,
    )?;
    sp.namespace = class(
        activation,
        gs,
        namespace::create_class(activation.context.gc_context),
        namespace_deriver,
    )?;
    sp.array = class(
        activation,
        gs,
        array::create_class(activation.context.gc_context),
        array_deriver,
    )?;

    activation.context.avm2.system_prototypes = Some(sp);

    function(
        activation.context.gc_context,
        gs,
        "",
        "trace",
        trace,
        fn_proto,
    );
    constant(
        activation.context.gc_context,
        gs,
        "",
        "undefined",
        Value::Undefined,
    );
    constant(activation.context.gc_context, gs, "", "null", Value::Null);
    constant(activation.context.gc_context, gs, "", "NaN", NAN.into());
    constant(
        activation.context.gc_context,
        gs,
        "",
        "Infinity",
        f64::INFINITY.into(),
    );

    // package `flash.events`
    class(
        activation,
        gs,
        flash::events::ieventdispatcher::create_interface(activation.context.gc_context),
        implicit_deriver,
    )?;
    class(
        activation,
        gs,
        flash::events::eventdispatcher::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;

    // package `flash.display`
    class(
        activation,
        gs,
        flash::display::displayobject::create_class(activation.context.gc_context),
        stage_deriver,
    )?;
    class(
        activation,
        gs,
        flash::display::interactiveobject::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;
    class(
        activation,
        gs,
        flash::display::displayobjectcontainer::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;
    class(
        activation,
        gs,
        flash::display::shape::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;
    class(
        activation,
        gs,
        flash::display::sprite::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .movieclip = class(
        activation,
        gs,
        flash::display::movieclip::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .framelabel = class(
        activation,
        gs,
        flash::display::framelabel::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .scene = class(
        activation,
        gs,
        flash::display::scene::create_class(activation.context.gc_context),
        implicit_deriver,
    )?;

    Ok(())
}
