//! Global scope built-ins

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{
    ClassObject, DomainObject, FunctionObject, Object, ScriptObject, TObject,
};
use crate::avm2::scope::Scope;
use crate::avm2::script::Script;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
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
mod vector;
mod xml;
mod xml_list;

const NS_RUFFLE_INTERNAL: &str = "https://ruffle.rs/AS3/impl/";
const NS_VECTOR: &str = "__AS3__.vec";

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
    pub rectangle: Object<'gc>,
    pub textfield: Object<'gc>,
    pub textformat: Object<'gc>,
    pub graphics: Object<'gc>,
    pub loaderinfo: Object<'gc>,
    pub bytearray: Object<'gc>,
    pub stage: Object<'gc>,
    pub sprite: Object<'gc>,
    pub simplebutton: Object<'gc>,
    pub regexp: Object<'gc>,
    pub vector: Object<'gc>,
    pub soundtransform: Object<'gc>,
    pub soundchannel: Object<'gc>,
    pub bitmap: Object<'gc>,
    pub bitmapdata: Object<'gc>,
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
            rectangle: empty,
            textfield: empty,
            textformat: empty,
            graphics: empty,
            loaderinfo: empty,
            bytearray: empty,
            stage: empty,
            sprite: empty,
            simplebutton: empty,
            regexp: empty,
            vector: empty,
            soundtransform: empty,
            soundchannel: empty,
            bitmap: empty,
            bitmapdata: empty,
        }
    }
}

/// This structure represents all system builtin classes.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SystemClasses<'gc> {
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
    pub rectangle: Object<'gc>,
    pub textfield: Object<'gc>,
    pub textformat: Object<'gc>,
    pub graphics: Object<'gc>,
    pub loaderinfo: Object<'gc>,
    pub bytearray: Object<'gc>,
    pub stage: Object<'gc>,
    pub sprite: Object<'gc>,
    pub simplebutton: Object<'gc>,
    pub regexp: Object<'gc>,
    pub vector: Object<'gc>,
    pub soundtransform: Object<'gc>,
    pub soundchannel: Object<'gc>,
    pub bitmap: Object<'gc>,
    pub bitmapdata: Object<'gc>,
}

impl<'gc> SystemClasses<'gc> {
    /// Construct a minimal set of system classes necessary for bootstrapping
    /// player globals.
    ///
    /// All other system classes aside from the three given here will be set to
    /// the empty object also handed to this function. It is the caller's
    /// responsibility to instantiate each class and replace the empty object
    /// with that.
    fn new(
        object: Object<'gc>,
        function: Object<'gc>,
        class: Object<'gc>,
        empty: Object<'gc>,
    ) -> Self {
        SystemClasses {
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
            rectangle: empty,
            textfield: empty,
            textformat: empty,
            graphics: empty,
            loaderinfo: empty,
            bytearray: empty,
            stage: empty,
            sprite: empty,
            simplebutton: empty,
            regexp: empty,
            vector: empty,
            soundtransform: empty,
            soundchannel: empty,
            bitmap: empty,
            bitmapdata: empty,
        }
    }
}

/// Add a free-function builtin to the global scope.
fn function<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: &'static str,
    nf: NativeMethodImpl,
    mut domain: Domain<'gc>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let mc = activation.context.gc_context;
    let qname = QName::new(Namespace::package(package), name);
    let method = Method::from_builtin(nf, name, mc);
    let as3fn = FunctionObject::from_method(activation, method, None, None).into();
    domain.export_definition(qname.clone(), script, mc)?;
    script
        .init()
        .1
        .install_dynamic_property(mc, qname, as3fn)
        .unwrap();

    Ok(())
}

/// Add a fully-formed class object builtin to the global scope.
///
/// This allows the caller to pre-populate the class's prototype with dynamic
/// properties, if necessary.
fn dynamic_class<'gc>(
    mc: MutationContext<'gc, '_>,
    class_object: Object<'gc>,
    mut domain: Domain<'gc>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let class = class_object
        .as_class_definition()
        .ok_or("Attempted to create builtin dynamic class without class on it's constructor!")?;
    let name = class.read().name().clone();

    script
        .init()
        .1
        .install_const(mc, name.clone(), 0, class_object.into(), false);
    domain.export_definition(name, script, mc)
}

/// Add a class builtin to the global scope.
///
/// This function returns the class object and class prototype as a pair, which
/// may be stored in `SystemClasses` and `SystemPrototypes`, respectively.
fn class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    class_def: GcCell<'gc, Class<'gc>>,
    mut domain: Domain<'gc>,
    script: Script<'gc>,
) -> Result<(Object<'gc>, Object<'gc>), Error> {
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

    let class_name = class_read.name().clone();
    drop(class_read);

    let class_object =
        ClassObject::from_class(activation, class_def, super_class, Some(global_scope))?;
    global.install_const(
        activation.context.gc_context,
        class_name.clone(),
        0,
        class_object.into(),
        false,
    );
    domain.export_definition(class_name, script, activation.context.gc_context)?;

    let proto = class_object
        .get_property(
            class_object,
            &QName::new(Namespace::public(), "prototype"),
            activation,
        )?
        .coerce_to_object(activation)?;

    Ok((class_object, proto))
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
    script.init().1.install_const(mc, name, 0, value, false);

    Ok(())
}

macro_rules! avm2_system_class {
    ($field:ident, $activation:ident, $class:expr, $domain:expr, $script:expr) => {
        let (class_object, proto) = class($activation, $class, $domain, $script)?;

        let sc = $activation.avm2().system_classes.as_mut().unwrap();
        sc.$field = class_object;

        let sp = $activation.avm2().system_prototypes.as_mut().unwrap();
        sp.$field = proto;
    };
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
    let gs = DomainObject::from_early_domain(mc, domain);
    let script = Script::empty_script(mc, gs);

    // public / root package
    //
    // This part of global initialization is very complicated, because
    // everything has to circularly reference everything else:
    //
    //  - Object is an instance of itself, as well as it's prototype
    //  - All other types are instances of Class, which is an instance of
    //    itself
    //  - Function's prototype is an instance of itself
    //  - All methods created by the above-mentioned classes are also instances
    //    of Function
    //
    // Hence, this ridiculously complicated dance of classdef, type allocation,
    // and partial initialization.
    let object_scope = Some(Scope::push_scope(gs.get_scope(), gs, mc));
    let object_classdef = object::create_class(mc);
    let object_class =
        ClassObject::from_class_partial(activation, object_classdef, None, object_scope)?;
    let object_proto = ScriptObject::bare_object(mc);

    let fn_scope = Some(Scope::push_scope(gs.get_scope(), gs, mc));
    let fn_classdef = function::create_class(mc);
    let fn_class = ClassObject::from_class_partial(
        activation,
        fn_classdef,
        Some(object_class.into()),
        fn_scope,
    )?;
    let fn_proto = ScriptObject::object(mc, object_proto);

    let class_scope = Some(Scope::push_scope(gs.get_scope(), gs, mc));
    let class_classdef = class::create_class(mc);
    let class_class = ClassObject::from_class_partial(
        activation,
        class_classdef,
        Some(object_class.into()),
        class_scope,
    )?;
    let class_proto = ScriptObject::object(mc, object_proto);

    // Now to weave the Gordian knot...
    object_class.link_prototype(activation, object_proto)?;
    object_class.link_type(activation, class_proto, class_class.into());

    fn_class.link_prototype(activation, fn_proto)?;
    fn_class.link_type(activation, class_proto, class_class.into());

    class_class.link_prototype(activation, class_proto)?;
    class_class.link_type(activation, class_proto, class_class.into());

    // At this point, we need at least a partial set of system prototypes in
    // order to continue initializing the player. The rest of the prototypes
    // are set to a bare object until we have a chance to initialize them.
    activation.context.avm2.system_prototypes = Some(SystemPrototypes::new(
        object_proto,
        fn_proto,
        class_proto,
        ScriptObject::bare_object(mc),
    ));

    activation.context.avm2.system_classes = Some(SystemClasses::new(
        object_class.into(),
        fn_class.into(),
        class_class.into(),
        ScriptObject::bare_object(mc),
    ));

    // Our activation environment is now functional enough to finish
    // initializing the core class weave. The order of initialization shouldn't
    // matter here, as long as all the initialization machinery can see and
    // link the various system types together correctly.
    let object_class = object_class.into_finished_class(activation)?;
    let fn_class = fn_class.into_finished_class(activation)?;
    let class_class = class_class.into_finished_class(activation)?;

    dynamic_class(mc, object_class, domain, script)?;
    dynamic_class(mc, fn_class, domain, script)?;
    dynamic_class(mc, class_class, domain, script)?;

    // After this point, it is safe to initialize any other classes.
    // Make sure to initialize superclasses *before* their subclasses!

    avm2_system_class!(
        global,
        activation,
        global_scope::create_class(mc),
        domain,
        script
    );

    // Oh, one more small hitch: the domain everything gets put into was
    // actually made *before* the core class weave, so let's fix that up now
    // that the global class actually exists.
    gs.set_proto(mc, activation.avm2().prototypes().global);

    avm2_system_class!(string, activation, string::create_class(mc), domain, script);
    avm2_system_class!(
        boolean,
        activation,
        boolean::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(number, activation, number::create_class(mc), domain, script);
    avm2_system_class!(int, activation, int::create_class(mc), domain, script);
    avm2_system_class!(uint, activation, uint::create_class(mc), domain, script);
    avm2_system_class!(
        namespace,
        activation,
        namespace::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(array, activation, array::create_class(mc), domain, script);

    function(activation, "", "trace", trace, domain, script)?;
    function(activation, "", "isFinite", is_finite, domain, script)?;
    function(activation, "", "isNaN", is_nan, domain, script)?;
    constant(mc, "", "undefined", Value::Undefined, domain, script)?;
    constant(mc, "", "null", Value::Null, domain, script)?;
    constant(mc, "", "NaN", f64::NAN.into(), domain, script)?;
    constant(mc, "", "Infinity", f64::INFINITY.into(), domain, script)?;

    class(activation, math::create_class(mc), domain, script)?;
    avm2_system_class!(regexp, activation, regexp::create_class(mc), domain, script);
    avm2_system_class!(vector, activation, vector::create_class(mc), domain, script);
    avm2_system_class!(xml, activation, xml::create_class(mc), domain, script);
    avm2_system_class!(
        xml_list,
        activation,
        xml_list::create_class(mc),
        domain,
        script
    );

    // package `flash.system`
    avm2_system_class!(
        application_domain,
        activation,
        flash::system::application_domain::create_class(mc),
        domain,
        script
    );
    class(
        activation,
        flash::system::capabilities::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::system::security::create_class(mc),
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
    avm2_system_class!(
        event,
        activation,
        flash::events::event::create_class(mc),
        domain,
        script
    );
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
    class(
        activation,
        flash::events::mouseevent::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::events::keyboardevent::create_class(mc),
        domain,
        script,
    )?;
    class(
        activation,
        flash::events::progressevent::create_class(mc),
        domain,
        script,
    )?;
    // package `flash.utils`
    avm2_system_class!(
        bytearray,
        activation,
        flash::utils::bytearray::create_class(mc),
        domain,
        script
    );

    //We also have to do this to the global scope, too.
    gs.as_application_domain()
        .unwrap()
        .init_default_domain_memory(activation)?;

    class(
        activation,
        flash::utils::endian::create_class(mc),
        domain,
        script,
    )?;

    class(
        activation,
        flash::utils::compression_algorithm::create_class(mc),
        domain,
        script,
    )?;

    function(
        activation,
        "flash.utils",
        "getTimer",
        flash::utils::get_timer,
        domain,
        script,
    )?;

    function(
        activation,
        "flash.utils",
        "getQualifiedClassName",
        flash::utils::get_qualified_class_name,
        domain,
        script,
    )?;

    function(
        activation,
        "flash.utils",
        "getQualifiedSuperclassName",
        flash::utils::get_qualified_super_class_name,
        domain,
        script,
    )?;

    function(
        activation,
        "flash.utils",
        "getDefinitionByName",
        flash::utils::get_definition_by_name,
        domain,
        script,
    )?;

    // package `flash.display`
    class(
        activation,
        flash::display::ibitmapdrawable::create_interface(mc),
        domain,
        script,
    )?;
    avm2_system_class!(
        display_object,
        activation,
        flash::display::displayobject::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        shape,
        activation,
        flash::display::shape::create_class(mc),
        domain,
        script
    );
    class(
        activation,
        flash::display::interactiveobject::create_class(mc),
        domain,
        script,
    )?;
    avm2_system_class!(
        simplebutton,
        activation,
        flash::display::simplebutton::create_class(mc),
        domain,
        script
    );
    class(
        activation,
        flash::display::displayobjectcontainer::create_class(mc),
        domain,
        script,
    )?;
    avm2_system_class!(
        sprite,
        activation,
        flash::display::sprite::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        movieclip,
        activation,
        flash::display::movieclip::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        framelabel,
        activation,
        flash::display::framelabel::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        scene,
        activation,
        flash::display::scene::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        graphics,
        activation,
        flash::display::graphics::create_class(mc),
        domain,
        script
    );
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
    avm2_system_class!(
        loaderinfo,
        activation,
        flash::display::loaderinfo::create_class(mc),
        domain,
        script
    );
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
    avm2_system_class!(
        stage,
        activation,
        flash::display::stage::create_class(mc),
        domain,
        script
    );
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
    avm2_system_class!(
        bitmap,
        activation,
        flash::display::bitmap::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        bitmapdata,
        activation,
        flash::display::bitmapdata::create_class(mc),
        domain,
        script
    );

    // package `flash.geom`
    avm2_system_class!(
        point,
        activation,
        flash::geom::point::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        rectangle,
        activation,
        flash::geom::rectangle::create_class(mc),
        domain,
        script
    );

    // package `flash.media`
    avm2_system_class!(
        video,
        activation,
        flash::media::video::create_class(mc),
        domain,
        script
    );
    class(
        activation,
        flash::media::sound::create_class(mc),
        domain,
        script,
    )?;
    avm2_system_class!(
        soundtransform,
        activation,
        flash::media::soundtransform::create_class(mc),
        domain,
        script
    );
    class(
        activation,
        flash::media::soundmixer::create_class(mc),
        domain,
        script,
    )?;
    avm2_system_class!(
        soundchannel,
        activation,
        flash::media::soundchannel::create_class(mc),
        domain,
        script
    );

    // package `flash.text`
    avm2_system_class!(
        textfield,
        activation,
        flash::text::textfield::create_class(mc),
        domain,
        script
    );
    avm2_system_class!(
        textformat,
        activation,
        flash::text::textformat::create_class(mc),
        domain,
        script
    );
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
    class(
        activation,
        flash::text::font::create_class(mc),
        domain,
        script,
    )?;

    // package `flash.crypto`
    function(
        activation,
        "flash.crypto",
        "generateRandomBytes",
        flash::crypto::generate_random_bytes,
        domain,
        script,
    )?;

    Ok(())
}
