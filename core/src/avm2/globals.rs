//! Global scope built-ins

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::method::NativeMethod;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{
    ClassObject, DomainObject, FunctionObject, Object, ScriptObject, TObject,
};
use crate::avm2::scope::Scope;
use crate::avm2::script::Script;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};

mod array;
mod boolean;
mod class;
mod flash;
mod function;
mod global_scope;
mod int;
mod math;
mod namespace;
mod number;
mod object;
mod regexp;
mod string;
mod r#uint;
mod xml;
mod xml_list;

const NS_RUFFLE_INTERNAL: &str = "https://ruffle.rs/AS3/impl/";

fn trace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let mut message = String::new();
    if !args.is_empty() {
        message.push_str(&args[0].clone().coerce_to_string(activation)?);
        for arg in &args[1..] {
            message.push(' ');
            message.push_str(&arg.clone().coerce_to_string(activation)?);
        }
    }

    activation.context.log.avm_trace(&message);

    Ok(Value::Undefined)
}

fn is_finite<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(val) = args.get(0) {
        Ok(val.coerce_to_number(activation)?.is_finite().into())
    } else {
        Ok(false.into())
    }
}

fn is_nan<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(val) = args.get(0) {
        Ok(val.coerce_to_number(activation)?.is_nan().into())
    } else {
        Ok(true.into())
    }
}

/// This structure represents all system builtins' prototypes.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SystemPrototypes<'gc> {
    pub object: Object<'gc>,
    pub function: Object<'gc>,
    pub class: Object<'gc>,
    pub global: Object<'gc>,
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
    pub application_domain: Object<'gc>,
    pub event: Object<'gc>,
    pub video: Object<'gc>,
    pub xml: Object<'gc>,
    pub xml_list: Object<'gc>,
    pub display_object: Object<'gc>,
    pub shape: Object<'gc>,
    pub point: Object<'gc>,
    pub textfield: Object<'gc>,
    pub textformat: Object<'gc>,
    pub graphics: Object<'gc>,
    pub loaderinfo: Object<'gc>,
    pub bytearray: Object<'gc>,
    pub stage: Object<'gc>,
    pub sprite: Object<'gc>,
    pub simplebutton: Object<'gc>,
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
            global: empty,
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
            application_domain: empty,
            event: empty,
            video: empty,
            xml: empty,
            xml_list: empty,
            display_object: empty,
            shape: empty,
            point: empty,
            textfield: empty,
            textformat: empty,
            graphics: empty,
            loaderinfo: empty,
            bytearray: empty,
            stage: empty,
            sprite: empty,
            simplebutton: empty,
        }
    }
}

/// Add a free-function builtin to the global scope.
fn function<'gc>(
    mc: MutationContext<'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    nf: NativeMethod,
    fn_proto: Object<'gc>,
    mut domain: Domain<'gc>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let name = QName::new(Namespace::package(package), name);
    let as3fn = FunctionObject::from_builtin(mc, nf, fn_proto).into();
    domain.export_definition(name.clone(), script, mc)?;
    script
        .init()
        .1
        .install_dynamic_property(mc, name, as3fn)
        .unwrap();

    Ok(())
}

/// Add a class builtin with prototype methods to the global scope.
///
/// Since the function has to return a normal prototype object in this case, we
/// have to construct a constructor to go along with it, as if we had called
/// `install_foreign_trait` with such a class.
fn dynamic_class<'gc>(
    mc: MutationContext<'gc, '_>,
    constr: Object<'gc>,
    class: GcCell<'gc, Class<'gc>>,
    mut domain: Domain<'gc>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let name = class.read().name().clone();

    script
        .init()
        .1
        .install_const(mc, name.clone(), 0, constr.into());
    domain.export_definition(name, script, mc)
}

/// Add a class builtin to the global scope.
///
/// This function returns a prototype which may be stored in `SystemPrototypes`.
/// The `custom_derive` is used to select a particular `TObject` impl, or you
/// can use `None` to indicate that this class does not change host object
/// impls.
fn class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    class_def: GcCell<'gc, Class<'gc>>,
    mut domain: Domain<'gc>,
    script: Script<'gc>,
) -> Result<Object<'gc>, Error> {
    let mut global = script.init().1;
    let global_scope = Scope::push_scope(global.get_scope(), global, activation.context.gc_context);

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

    let (mut constr, _cinit) =
        ClassObject::from_class(activation, class_def, super_class, Some(global_scope))?;
    global.install_const(
        activation.context.gc_context,
        class_read.name().clone(),
        0,
        constr.into(),
    );
    domain.export_definition(
        class_read.name().clone(),
        script,
        activation.context.gc_context,
    )?;

    constr
        .get_property(
            constr,
            &QName::new(Namespace::public(), "prototype"),
            activation,
        )?
        .coerce_to_object(activation)
}

/// Add a builtin constant to the global scope.
fn constant<'gc>(
    mc: MutationContext<'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    value: Value<'gc>,
    mut domain: Domain<'gc>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let name = QName::new(Namespace::package(package), name);
    domain.export_definition(name.clone(), script, mc)?;
    script.init().1.install_const(mc, name, 0, value);

    Ok(())
}

/// Initialize the player global domain.
///
/// This should be called only once, to construct the global scope of the
/// player. It will return a list of prototypes it has created, which should be
/// stored on the AVM. All relevant declarations will also be attached to the
/// given domain.
pub fn load_player_globals<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    domain: Domain<'gc>,
) -> Result<(), Error> {
    let mc = activation.context.gc_context;
    let gs = DomainObject::from_domain(mc, None, domain);
    let script = Script::empty_script(mc, gs);

    // public / root package
    //
    // We have to do this particular dance so that we have Object methods whose
    // functions have call/apply in their prototypes, and that Function is also
    // a subclass of Object.
    let (object_proto, object_class) = object::create_proto(activation, gs);
    let (fn_proto, fn_class) = function::create_proto(activation, gs, object_proto);

    let object_constr = object::fill_proto(activation, object_proto, fn_proto)?;
    let function_constr = function::fill_proto(activation, fn_proto, object_constr);

    let (class_constr, class_proto, class_class) =
        class::create_class(activation, gs, object_constr, object_proto, fn_proto);

    dynamic_class(mc, object_constr, object_class, domain, script)?;
    dynamic_class(mc, function_constr, fn_class, domain, script)?;
    dynamic_class(mc, class_constr, class_class, domain, script)?;

    // At this point, we need at least a partial set of system prototypes in
    // order to continue initializing the player. The rest of the prototypes
    // are set to a bare object until we have a chance to initialize them.
    activation.context.avm2.system_prototypes = Some(SystemPrototypes::new(
        object_proto,
        fn_proto,
        class_proto,
        ScriptObject::bare_object(mc),
    ));

    // Even sillier: for the sake of clarity and the borrow checker we need to
    // clone the prototypes list and modify it outside of the activation. This
    // also has the side effect that none of these classes can get at each
    // other from the activation they're handed.
    let mut sp = activation.context.avm2.system_prototypes.clone().unwrap();

    sp.global = class(activation, global_scope::create_class(mc), domain, script)?;
    sp.string = class(activation, string::create_class(mc), domain, script)?;
    sp.boolean = class(activation, boolean::create_class(mc), domain, script)?;
    sp.number = class(activation, number::create_class(mc), domain, script)?;
    sp.int = class(activation, int::create_class(mc), domain, script)?;
    sp.uint = class(activation, uint::create_class(mc), domain, script)?;
    sp.namespace = class(activation, namespace::create_class(mc), domain, script)?;
    sp.array = class(activation, array::create_class(mc), domain, script)?;

    // At this point we have to hide the fact that we had to create the player
    // globals scope *before* the `Object` class
    gs.set_proto(mc, sp.global);

    activation.context.avm2.system_prototypes = Some(sp);

    function(mc, "", "trace", trace, fn_proto, domain, script)?;
    function(mc, "", "isFinite", is_finite, fn_proto, domain, script)?;
    function(mc, "", "isNaN", is_nan, fn_proto, domain, script)?;
    constant(mc, "", "undefined", Value::Undefined, domain, script)?;
    constant(mc, "", "null", Value::Null, domain, script)?;
    constant(mc, "", "NaN", f64::NAN.into(), domain, script)?;
    constant(mc, "", "Infinity", f64::INFINITY.into(), domain, script)?;

    class(activation, math::create_class(mc), domain, script)?;
    class(activation, regexp::create_class(mc), domain, script)?;

    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .xml = class(activation, xml::create_class(mc), domain, script)?;

    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .xml_list = class(activation, xml_list::create_class(mc), domain, script)?;

    // package `flash.system`
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .application_domain = class(
        activation,
        flash::system::application_domain::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::system::system::create_class(mc),
        domain,
        script,
    )?;

    // package `flash.events`
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .event = class(
        activation,
        flash::events::event::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::events::ieventdispatcher::create_interface(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::events::eventdispatcher::create_class(mc),
        domain,
        script,
    )?;
    // package `flash.utils`
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .bytearray = class(
        activation,
        flash::utils::bytearray::create_class(mc),
        domain,
        script,
    )?;

    class(
        activation,
        flash::utils::endian::create_class(mc),
        domain,
        script,
    )?;

    class(
        activation,
        flash::utils::compression_algorithm::create_class(mc),
        implicit_deriver,
        domain,
        script,
    )?;

    function(
        mc,
        "flash.utils",
        "getTimer",
        flash::utils::get_timer,
        fn_proto,
        domain,
        script,
    )?;

    // package `flash.display`
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .display_object = class(
        activation,
        flash::display::displayobject::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .shape = class(
        activation,
        flash::display::shape::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::interactiveobject::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .simplebutton = class(
        activation,
        flash::display::simplebutton::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::displayobjectcontainer::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .sprite = class(
        activation,
        flash::display::sprite::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .movieclip = class(
        activation,
        flash::display::movieclip::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .framelabel = class(
        activation,
        flash::display::framelabel::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .scene = class(
        activation,
        flash::display::scene::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .graphics = class(
        activation,
        flash::display::graphics::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::jointstyle::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::linescalemode::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::capsstyle::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .loaderinfo = class(
        activation,
        flash::display::loaderinfo::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::actionscriptversion::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::swfversion::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .stage = class(
        activation,
        flash::display::stage::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::stagescalemode::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::stagealign::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::stagedisplaystate::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::display::stagequality::create_class(mc),
        domain,
        script,
    )?;

    // package `flash.geom`
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .point = class(
        activation,
        flash::geom::point::create_class(mc),
        domain,
        script,
    )?;

    // package `flash.media`
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .video = class(
        activation,
        flash::media::video::create_class(mc),
        domain,
        script,
    )?;

    // package `flash.text`
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .textfield = class(
        activation,
        flash::text::textfield::create_class(mc),
        domain,
        script,
    )?;
    activation
        .context
        .avm2
        .system_prototypes
        .as_mut()
        .unwrap()
        .textformat = class(
        activation,
        flash::text::textformat::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::text::textfieldautosize::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::text::textformatalign::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::text::textfieldtype::create_class(mc),
        domain,
        script,
    )?;

    Ok(())
}
