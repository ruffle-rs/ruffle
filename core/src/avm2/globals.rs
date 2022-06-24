use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{
    ClassObject, FunctionObject, NamespaceObject, Object, ScriptObject, TObject,
};
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::script::Script;
use crate::avm2::value::Value;
use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::string::AvmString;
use crate::tag_utils::{self, SwfMovie, SwfSlice, SwfStream};
use gc_arena::{Collect, GcCell, MutationContext};
use std::sync::Arc;
use swf::TagCode;

mod array;
mod boolean;
mod class;
mod date;
mod flash;
mod function;
mod global_scope;
mod int;
mod json;
mod math;
mod namespace;
mod number;
mod object;
mod qname;
mod regexp;
mod string;
mod toplevel;
mod r#uint;
mod vector;
mod xml;
mod xml_list;

pub(crate) const NS_RUFFLE_INTERNAL: &str = "https://ruffle.rs/AS3/impl/";
const NS_VECTOR: &str = "__AS3__.vec";

pub use flash::utils::NS_FLASH_PROXY;

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
    pub fullscreenevent: Object<'gc>,
    pub video: Object<'gc>,
    pub xml: Object<'gc>,
    pub xml_list: Object<'gc>,
    pub display_object: Object<'gc>,
    pub shape: Object<'gc>,
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
    pub date: Object<'gc>,
    pub qname: Object<'gc>,
    pub sharedobject: Object<'gc>,
    pub nativemenu: Object<'gc>,
    pub contextmenu: Object<'gc>,
    pub mouseevent: Object<'gc>,
    pub textevent: Object<'gc>,
    pub errorevent: Object<'gc>,
    pub ioerrorevent: Object<'gc>,
    pub securityerrorevent: Object<'gc>,
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
        global: Object<'gc>,
        empty: Object<'gc>,
    ) -> Self {
        SystemPrototypes {
            object,
            function,
            class,
            global,
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
            fullscreenevent: empty,
            video: empty,
            xml: empty,
            xml_list: empty,
            display_object: empty,
            shape: empty,
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
            date: empty,
            qname: empty,
            sharedobject: empty,
            nativemenu: empty,
            contextmenu: empty,
            mouseevent: empty,
            textevent: empty,
            errorevent: empty,
            ioerrorevent: empty,
            securityerrorevent: empty,
        }
    }
}

/// This structure represents all system builtin classes.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SystemClasses<'gc> {
    pub object: ClassObject<'gc>,
    pub function: ClassObject<'gc>,
    pub class: ClassObject<'gc>,
    pub global: ClassObject<'gc>,
    pub string: ClassObject<'gc>,
    pub boolean: ClassObject<'gc>,
    pub number: ClassObject<'gc>,
    pub int: ClassObject<'gc>,
    pub uint: ClassObject<'gc>,
    pub namespace: ClassObject<'gc>,
    pub array: ClassObject<'gc>,
    pub movieclip: ClassObject<'gc>,
    pub framelabel: ClassObject<'gc>,
    pub scene: ClassObject<'gc>,
    pub application_domain: ClassObject<'gc>,
    pub event: ClassObject<'gc>,
    pub fullscreenevent: ClassObject<'gc>,
    pub video: ClassObject<'gc>,
    pub xml: ClassObject<'gc>,
    pub xml_list: ClassObject<'gc>,
    pub display_object: ClassObject<'gc>,
    pub shape: ClassObject<'gc>,
    pub textfield: ClassObject<'gc>,
    pub textformat: ClassObject<'gc>,
    pub graphics: ClassObject<'gc>,
    pub loaderinfo: ClassObject<'gc>,
    pub bytearray: ClassObject<'gc>,
    pub stage: ClassObject<'gc>,
    pub sprite: ClassObject<'gc>,
    pub simplebutton: ClassObject<'gc>,
    pub regexp: ClassObject<'gc>,
    pub vector: ClassObject<'gc>,
    pub soundtransform: ClassObject<'gc>,
    pub soundchannel: ClassObject<'gc>,
    pub bitmap: ClassObject<'gc>,
    pub bitmapdata: ClassObject<'gc>,
    pub date: ClassObject<'gc>,
    pub qname: ClassObject<'gc>,
    pub sharedobject: ClassObject<'gc>,
    pub nativemenu: ClassObject<'gc>,
    pub contextmenu: ClassObject<'gc>,
    pub mouseevent: ClassObject<'gc>,
    pub textevent: ClassObject<'gc>,
    pub errorevent: ClassObject<'gc>,
    pub ioerrorevent: ClassObject<'gc>,
    pub securityerrorevent: ClassObject<'gc>,
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
        object: ClassObject<'gc>,
        function: ClassObject<'gc>,
        class: ClassObject<'gc>,
        global: ClassObject<'gc>,
    ) -> Self {
        SystemClasses {
            object,
            function,
            class,
            global,
            // temporary initialization
            string: object,
            boolean: object,
            number: object,
            int: object,
            uint: object,
            namespace: object,
            array: object,
            movieclip: object,
            framelabel: object,
            scene: object,
            application_domain: object,
            event: object,
            fullscreenevent: object,
            video: object,
            xml: object,
            xml_list: object,
            display_object: object,
            shape: object,
            textfield: object,
            textformat: object,
            graphics: object,
            loaderinfo: object,
            bytearray: object,
            stage: object,
            sprite: object,
            simplebutton: object,
            regexp: object,
            vector: object,
            soundtransform: object,
            soundchannel: object,
            bitmap: object,
            bitmapdata: object,
            date: object,
            qname: object,
            sharedobject: object,
            nativemenu: object,
            contextmenu: object,
            mouseevent: object,
            textevent: object,
            errorevent: object,
            ioerrorevent: object,
            securityerrorevent: object,
        }
    }
}

/// Add a free-function builtin to the global scope.
fn function<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: &'static str,
    nf: NativeMethodImpl,
    script: Script<'gc>,
) -> Result<(), Error> {
    let (_, mut global, mut domain) = script.init();
    let mc = activation.context.gc_context;
    let scope = activation.create_scopechain();
    let qname = QName::new(Namespace::package(package), name);
    let method = Method::from_builtin(nf, name, mc);
    let as3fn = FunctionObject::from_method(activation, method, scope, None, None).into();
    domain.export_definition(qname, script, mc)?;
    global.install_const_late(mc, qname, as3fn);

    Ok(())
}

/// Add a fully-formed class object builtin to the global scope.
///
/// This allows the caller to pre-populate the class's prototype with dynamic
/// properties, if necessary.
fn dynamic_class<'gc>(
    mc: MutationContext<'gc, '_>,
    class_object: ClassObject<'gc>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let (_, mut global, mut domain) = script.init();
    let class = class_object.inner_class_definition();
    let name = class.read().name();

    global.install_const_late(mc, name, class_object.into());
    domain.export_definition(name, script, mc)
}

/// Add a class builtin to the global scope.
///
/// This function returns the class object and class prototype as a pair, which
/// may be stored in `SystemClasses` and `SystemPrototypes`, respectively.
fn class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    class_def: GcCell<'gc, Class<'gc>>,
    script: Script<'gc>,
) -> Result<(ClassObject<'gc>, Object<'gc>), Error> {
    let (_, mut global, mut domain) = script.init();

    let class_read = class_def.read();
    let super_class = if let Some(sc_name) = class_read.super_class_name() {
        let super_class: Result<Object<'gc>, Error> = global
            .get_property(sc_name, activation)
            .ok()
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                format!(
                    "Could not resolve superclass {} when defining global class {}",
                    sc_name.to_qualified_name(activation.context.gc_context),
                    class_read
                        .name()
                        .to_qualified_name(activation.context.gc_context)
                )
                .into()
            });
        let super_class = super_class?
            .as_class_object()
            .ok_or_else(|| Error::from("Base class of a global class is not a class"))?;

        Some(super_class)
    } else {
        None
    };

    let class_name = class_read.name();
    drop(class_read);

    let class_object = ClassObject::from_class(activation, class_def, super_class)?;
    global.install_const_late(
        activation.context.gc_context,
        class_name,
        class_object.into(),
    );
    domain.export_definition(class_name, script, activation.context.gc_context)?;

    Ok((class_object, class_object.prototype()))
}

/// Add a builtin constant to the global scope.
fn constant<'gc>(
    mc: MutationContext<'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    value: Value<'gc>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let (_, mut global, mut domain) = script.init();
    let name = QName::new(Namespace::package(package), name);
    domain.export_definition(name, script, mc)?;
    global.install_const_late(mc, name, value);

    Ok(())
}

fn namespace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    uri: impl Into<AvmString<'gc>>,
    script: Script<'gc>,
) -> Result<(), Error> {
    let namespace = NamespaceObject::from_namespace(activation, Namespace::Namespace(uri.into()))?;
    constant(
        activation.context.gc_context,
        package,
        name,
        namespace.into(),
        script,
    )
}

macro_rules! avm2_system_class {
    ($field:ident, $activation:ident, $class:expr, $script:expr) => {
        let (class_object, proto) = class($activation, $class, $script)?;

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

    let globals = ScriptObject::bare_object(activation.context.gc_context);
    let gs = ScopeChain::new(domain).chain(mc, &[Scope::new(globals)]);
    let script = Script::empty_script(mc, globals, domain);

    // Set the outer scope of this activation to the global scope.
    activation.set_outer(gs);

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
    //  - All classes are put on Global's trait list, but Global needs
    //    to be initialized first, but you can't do that until Object/Class are ready.
    //
    // Hence, this ridiculously complicated dance of classdef, type allocation,
    // and partial initialization.
    let object_classdef = object::create_class(mc);
    let object_class = ClassObject::from_class_partial(activation, object_classdef, None)?;
    let object_proto = ScriptObject::bare_instance(mc, object_class);

    let fn_classdef = function::create_class(mc);
    let fn_class = ClassObject::from_class_partial(activation, fn_classdef, Some(object_class))?;
    let fn_proto = ScriptObject::instance(mc, fn_class, object_proto);

    let class_classdef = class::create_class(mc);
    let class_class =
        ClassObject::from_class_partial(activation, class_classdef, Some(object_class))?;
    let class_proto = ScriptObject::instance(mc, object_class, object_proto);

    let global_classdef = global_scope::create_class(mc);
    let global_class =
        ClassObject::from_class_partial(activation, global_classdef, Some(object_class))?;
    let global_proto = ScriptObject::instance(mc, object_class, object_proto);

    // Now to weave the Gordian knot...
    object_class.link_prototype(activation, object_proto)?;
    object_class.link_type(activation, class_proto, class_class);

    fn_class.link_prototype(activation, fn_proto)?;
    fn_class.link_type(activation, class_proto, class_class);

    class_class.link_prototype(activation, class_proto)?;
    class_class.link_type(activation, class_proto, class_class);

    global_class.link_prototype(activation, global_proto)?;
    global_class.link_type(activation, class_proto, class_class);

    // At this point, we need at least a partial set of system prototypes in
    // order to continue initializing the player. The rest of the prototypes
    // are set to a bare object until we have a chance to initialize them.
    activation.context.avm2.system_prototypes = Some(SystemPrototypes::new(
        object_proto,
        fn_proto,
        class_proto,
        global_proto,
        ScriptObject::bare_object(mc),
    ));

    activation.context.avm2.system_classes = Some(SystemClasses::new(
        object_class,
        fn_class,
        class_class,
        global_class,
    ));

    // Our activation environment is now functional enough to finish
    // initializing the core class weave. The order of initialization shouldn't
    // matter here, as long as all the initialization machinery can see and
    // link the various system types together correctly.
    let class_class = class_class.into_finished_class(activation)?;
    let fn_class = fn_class.into_finished_class(activation)?;
    let object_class = object_class.into_finished_class(activation)?;
    let _global_class = global_class.into_finished_class(activation)?;

    globals.set_proto(mc, activation.avm2().prototypes().global);
    globals.set_instance_of(mc, activation.avm2().classes().global);
    globals.fork_vtable(activation.context.gc_context);

    // From this point, `globals` is safe to be modified

    dynamic_class(mc, object_class, script)?;
    dynamic_class(mc, fn_class, script)?;
    dynamic_class(mc, class_class, script)?;

    // After this point, it is safe to initialize any other classes.
    // Make sure to initialize superclasses *before* their subclasses!

    load_playerglobal(activation, domain)?;

    avm2_system_class!(string, activation, string::create_class(mc), script);
    avm2_system_class!(boolean, activation, boolean::create_class(mc), script);
    avm2_system_class!(number, activation, number::create_class(mc), script);
    avm2_system_class!(int, activation, int::create_class(mc), script);
    avm2_system_class!(uint, activation, uint::create_class(mc), script);
    avm2_system_class!(namespace, activation, namespace::create_class(mc), script);
    avm2_system_class!(qname, activation, qname::create_class(mc), script);
    avm2_system_class!(array, activation, array::create_class(mc), script);

    function(activation, "", "trace", toplevel::trace, script)?;
    function(activation, "", "isFinite", toplevel::is_finite, script)?;
    function(activation, "", "isNaN", toplevel::is_nan, script)?;
    function(activation, "", "parseInt", toplevel::parse_int, script)?;
    function(activation, "", "parseFloat", toplevel::parse_float, script)?;
    function(activation, "", "escape", toplevel::escape, script)?;
    constant(mc, "", "undefined", Value::Undefined, script)?;
    constant(mc, "", "null", Value::Null, script)?;
    constant(mc, "", "NaN", f64::NAN.into(), script)?;
    constant(mc, "", "Infinity", f64::INFINITY.into(), script)?;

    class(activation, math::create_class(mc), script)?;
    class(activation, json::create_class(mc), script)?;
    avm2_system_class!(regexp, activation, regexp::create_class(mc), script);
    avm2_system_class!(vector, activation, vector::create_class(mc), script);
    avm2_system_class!(xml, activation, xml::create_class(mc), script);
    avm2_system_class!(xml_list, activation, xml_list::create_class(mc), script);

    avm2_system_class!(date, activation, date::create_class(mc), script);

    // package `flash.system`
    avm2_system_class!(
        application_domain,
        activation,
        flash::system::application_domain::create_class(mc),
        script
    );
    class(
        activation,
        flash::system::capabilities::create_class(mc),
        script,
    )?;
    class(
        activation,
        flash::system::security::create_class(mc),
        script,
    )?;
    class(activation, flash::system::system::create_class(mc), script)?;

    // package `flash.events`
    avm2_system_class!(
        event,
        activation,
        flash::events::event::create_class(mc),
        script
    );
    class(
        activation,
        flash::events::ieventdispatcher::create_interface(mc),
        script,
    )?;
    class(
        activation,
        flash::events::eventdispatcher::create_class(mc),
        script,
    )?;
    avm2_system_class!(
        mouseevent,
        activation,
        flash::events::mouseevent::create_class(mc),
        script
    );
    avm2_system_class!(
        textevent,
        activation,
        flash::events::textevent::create_class(mc),
        script
    );
    avm2_system_class!(
        errorevent,
        activation,
        flash::events::errorevent::create_class(mc),
        script
    );
    avm2_system_class!(
        securityerrorevent,
        activation,
        flash::events::securityerrorevent::create_class(mc),
        script
    );
    avm2_system_class!(
        ioerrorevent,
        activation,
        flash::events::ioerrorevent::create_class(mc),
        script
    );
    class(
        activation,
        flash::events::contextmenuevent::create_class(mc),
        script,
    )?;
    class(
        activation,
        flash::events::keyboardevent::create_class(mc),
        script,
    )?;
    class(
        activation,
        flash::events::progressevent::create_class(mc),
        script,
    )?;
    class(
        activation,
        flash::events::activityevent::create_class(mc),
        script,
    )?;
    avm2_system_class!(
        fullscreenevent,
        activation,
        flash::events::fullscreenevent::create_class(mc),
        script
    );
    class(
        activation,
        flash::events::eventphase::create_class(mc),
        script,
    )?;

    // package `flash.utils`
    avm2_system_class!(
        bytearray,
        activation,
        flash::utils::bytearray::create_class(mc),
        script
    );

    domain.init_default_domain_memory(activation)?;

    class(
        activation,
        flash::utils::dictionary::create_class(mc),
        script,
    )?;

    class(activation, flash::utils::timer::create_class(mc), script)?;

    namespace(
        activation,
        "flash.utils",
        "flash_proxy",
        flash::utils::NS_FLASH_PROXY,
        script,
    )?;
    class(activation, flash::utils::proxy::create_class(mc), script)?;

    function(
        activation,
        "flash.utils",
        "getTimer",
        flash::utils::get_timer,
        script,
    )?;

    function(
        activation,
        "flash.utils",
        "getQualifiedClassName",
        flash::utils::get_qualified_class_name,
        script,
    )?;

    function(
        activation,
        "flash.utils",
        "getQualifiedSuperclassName",
        flash::utils::get_qualified_super_class_name,
        script,
    )?;

    function(
        activation,
        "flash.utils",
        "getDefinitionByName",
        flash::utils::get_definition_by_name,
        script,
    )?;

    // package `flash.display`
    class(
        activation,
        flash::display::ibitmapdrawable::create_interface(mc),
        script,
    )?;
    avm2_system_class!(
        display_object,
        activation,
        flash::display::displayobject::create_class(mc),
        script
    );
    avm2_system_class!(
        shape,
        activation,
        flash::display::shape::create_class(mc),
        script
    );
    class(
        activation,
        flash::display::interactiveobject::create_class(mc),
        script,
    )?;
    avm2_system_class!(
        simplebutton,
        activation,
        flash::display::simplebutton::create_class(mc),
        script
    );
    class(
        activation,
        flash::display::displayobjectcontainer::create_class(mc),
        script,
    )?;
    avm2_system_class!(
        sprite,
        activation,
        flash::display::sprite::create_class(mc),
        script
    );
    avm2_system_class!(
        movieclip,
        activation,
        flash::display::movieclip::create_class(mc),
        script
    );
    avm2_system_class!(
        framelabel,
        activation,
        flash::display::framelabel::create_class(mc),
        script
    );
    avm2_system_class!(
        graphics,
        activation,
        flash::display::graphics::create_class(mc),
        script
    );
    class(activation, flash::display::loader::create_class(mc), script)?;
    avm2_system_class!(
        loaderinfo,
        activation,
        flash::display::loaderinfo::create_class(mc),
        script
    );
    avm2_system_class!(
        stage,
        activation,
        flash::display::stage::create_class(mc),
        script
    );
    avm2_system_class!(
        bitmap,
        activation,
        flash::display::bitmap::create_class(mc),
        script
    );
    avm2_system_class!(
        bitmapdata,
        activation,
        flash::display::bitmapdata::create_class(mc),
        script
    );
    avm2_system_class!(
        nativemenu,
        activation,
        flash::display::nativemenu::create_class(mc),
        script
    );
    class(
        activation,
        flash::display::nativemenuitem::create_class(mc),
        script,
    )?;

    // package `flash.filters`
    class(
        activation,
        flash::filters::bitmapfilter::create_class(mc),
        script,
    )?;
    class(
        activation,
        flash::filters::blurfilter::create_class(mc),
        script,
    )?;
    class(
        activation,
        flash::filters::glowfilter::create_class(mc),
        script,
    )?;

    // package `flash.geom`
    class(activation, flash::geom::matrix::create_class(mc), script)?;

    // package `flash.media`
    avm2_system_class!(
        video,
        activation,
        flash::media::video::create_class(mc),
        script
    );
    class(activation, flash::media::sound::create_class(mc), script)?;
    avm2_system_class!(
        soundtransform,
        activation,
        flash::media::soundtransform::create_class(mc),
        script
    );
    class(
        activation,
        flash::media::soundmixer::create_class(mc),
        script,
    )?;
    avm2_system_class!(
        soundchannel,
        activation,
        flash::media::soundchannel::create_class(mc),
        script
    );

    // package `flash.ui`
    avm2_system_class!(
        contextmenu,
        activation,
        flash::ui::contextmenu::create_class(mc),
        script
    );
    class(
        activation,
        flash::ui::contextmenuitem::create_class(mc),
        script,
    )?;
    class(activation, flash::ui::mouse::create_class(mc), script)?;
    class(activation, flash::ui::keyboard::create_class(mc), script)?;

    // package `flash.net`
    avm2_system_class!(
        sharedobject,
        activation,
        flash::net::sharedobject::create_class(mc),
        script
    );

    class(
        activation,
        flash::net::object_encoding::create_class(mc),
        script,
    )?;
    class(activation, flash::net::url_loader::create_class(mc), script)?;
    class(
        activation,
        flash::net::url_loader_data_format::create_class(mc),
        script,
    )?;
    class(
        activation,
        flash::net::url_request::create_class(mc),
        script,
    )?;

    // package `flash.text`
    avm2_system_class!(
        textfield,
        activation,
        flash::text::textfield::create_class(mc),
        script
    );
    avm2_system_class!(
        textformat,
        activation,
        flash::text::textformat::create_class(mc),
        script
    );
    class(activation, flash::text::font::create_class(mc), script)?;

    // package `flash.crypto`
    function(
        activation,
        "flash.crypto",
        "generateRandomBytes",
        flash::crypto::generate_random_bytes,
        script,
    )?;

    // package `flash.external`
    class(
        activation,
        flash::external::externalinterface::create_class(mc),
        script,
    )?;

    Ok(())
}

/// This file is built by 'core/build_playerglobal/'
/// See that tool, and 'core/src/avm2/globals/README.md', for more details
const PLAYERGLOBAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/playerglobal.swf"));

/// Loads classes from our custom 'playerglobal' (which are written in ActionScript)
/// into the environment. See 'core/src/avm2/globals/README.md' for more information
fn load_playerglobal<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    domain: Domain<'gc>,
) -> Result<(), Error> {
    let movie = Arc::new(SwfMovie::from_data(PLAYERGLOBAL, None, None)?);

    let slice = SwfSlice::from(movie);

    let mut reader = slice.read_from(0);

    let tag_callback = |reader: &mut SwfStream<'_>, tag_code, tag_len| {
        if tag_code == TagCode::DoAbc {
            Avm2::load_abc_from_do_abc(&mut activation.context, &slice, domain, reader, tag_len)?;
        } else if tag_code != TagCode::End {
            panic!(
                "playerglobal should only contain `DoAbc` tag - found tag {:?}",
                tag_code
            )
        }
        Ok(())
    };

    let _ = tag_utils::decode_tags(&mut reader, tag_callback, TagCode::End);
    macro_rules! avm2_system_classes_playerglobal {
        ($activation:expr, $script:expr, [$(($package:expr, $class_name:expr, $field:ident)),* $(,)?]) => {
            $(
                let qname = QName::new(Namespace::package($package), $class_name);
                let class_object = activation.resolve_class(&qname.into())?;
                let sc = $activation.avm2().system_classes.as_mut().unwrap();
                sc.$field = class_object;

                let sp = $activation.avm2().system_prototypes.as_mut().unwrap();
                sp.$field = class_object.prototype();
            )*
        }
    }

    // This acts the same way as 'avm2_system_class', but for classes
    // declared in 'playerglobal'. Classes are declared as ("package", "class", field_name),
    // and are stored in 'avm2().system_classes' and 'avm2().system_prototypes'
    avm2_system_classes_playerglobal!(activation, script, [("flash.display", "Scene", scene)]);

    Ok(())
}
