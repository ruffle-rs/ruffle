use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{
    ClassObject, FunctionObject, NamespaceObject, Object, ScriptObject, TObject,
};
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::script::Script;
use crate::avm2::value::Value;
use crate::avm2::Avm2;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::string::AvmString;
use crate::tag_utils::{self, ControlFlow, SwfMovie, SwfSlice, SwfStream};
use gc_arena::{Collect, GcCell, MutationContext};
use std::sync::Arc;
use swf::TagCode;

mod array;
mod boolean;
mod class;
mod date;
mod error;
pub mod flash;
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
pub(crate) const NS_VECTOR: &str = "__AS3__.vec";

pub use flash::utils::NS_FLASH_PROXY;

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
    pub mouseevent: ClassObject<'gc>,
    pub progressevent: ClassObject<'gc>,
    pub textevent: ClassObject<'gc>,
    pub errorevent: ClassObject<'gc>,
    pub ioerrorevent: ClassObject<'gc>,
    pub securityerrorevent: ClassObject<'gc>,
    pub transform: ClassObject<'gc>,
    pub colortransform: ClassObject<'gc>,
    pub matrix: ClassObject<'gc>,
    pub illegaloperationerror: ClassObject<'gc>,
    pub eventdispatcher: ClassObject<'gc>,
    pub rectangle: ClassObject<'gc>,
    pub keyboardevent: ClassObject<'gc>,
    pub point: ClassObject<'gc>,
    pub rangeerror: ClassObject<'gc>,
    pub referenceerror: ClassObject<'gc>,
    pub argumenterror: ClassObject<'gc>,
    pub typeerror: ClassObject<'gc>,
    pub verifyerror: ClassObject<'gc>,
    pub ioerror: ClassObject<'gc>,
    pub eoferror: ClassObject<'gc>,
    pub uncaughterrorevents: ClassObject<'gc>,
    pub statictext: ClassObject<'gc>,
    pub textlinemetrics: ClassObject<'gc>,
    pub stage3d: ClassObject<'gc>,
    pub context3d: ClassObject<'gc>,
    pub indexbuffer3d: ClassObject<'gc>,
    pub vertexbuffer3d: ClassObject<'gc>,
    pub program3d: ClassObject<'gc>,
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
            mouseevent: object,
            progressevent: object,
            textevent: object,
            errorevent: object,
            ioerrorevent: object,
            securityerrorevent: object,
            transform: object,
            colortransform: object,
            matrix: object,
            illegaloperationerror: object,
            eventdispatcher: object,
            rectangle: object,
            keyboardevent: object,
            point: object,
            rangeerror: object,
            referenceerror: object,
            argumenterror: object,
            typeerror: object,
            verifyerror: object,
            ioerror: object,
            eoferror: object,
            uncaughterrorevents: object,
            statictext: object,
            textlinemetrics: object,
            stage3d: object,
            context3d: object,
            indexbuffer3d: object,
            vertexbuffer3d: object,
            program3d: object,
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
) -> Result<(), Error<'gc>> {
    let (_, mut global, mut domain) = script.init();
    let mc = activation.context.gc_context;
    let scope = activation.create_scopechain();
    let qname = QName::new(Namespace::package(package), name);
    let method = Method::from_builtin(nf, name, mc);
    let as3fn = FunctionObject::from_method(activation, method, scope, None, None).into();
    domain.export_definition(qname, script, mc)?;
    global.install_const_late(mc, qname, as3fn, activation.avm2().classes().function);

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
    // The `ClassObject` of the `Class` class
    class_class: ClassObject<'gc>,
) -> Result<(), Error<'gc>> {
    let (_, mut global, mut domain) = script.init();
    let class = class_object.inner_class_definition();
    let name = class.read().name();

    global.install_const_late(mc, name, class_object.into(), class_class);
    domain.export_definition(name, script, mc)
}

/// Add a class builtin to the global scope.
///
/// This function returns the class object and class prototype as a class, which
/// may be stored in `SystemClasses`
fn class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    class_def: GcCell<'gc, Class<'gc>>,
    script: Script<'gc>,
) -> Result<ClassObject<'gc>, Error<'gc>> {
    let (_, mut global, mut domain) = script.init();

    let class_read = class_def.read();
    let super_class = if let Some(sc_name) = class_read.super_class_name() {
        let super_class: Result<Object<'gc>, Error<'gc>> = activation
            .resolve_definition(sc_name)
            .ok()
            .and_then(|v| v)
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
        activation.avm2().classes().class,
    );
    domain.export_definition(class_name, script, activation.context.gc_context)?;

    Ok(class_object)
}

/// Add a builtin constant to the global scope.
fn constant<'gc>(
    mc: MutationContext<'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    value: Value<'gc>,
    script: Script<'gc>,
    class: ClassObject<'gc>,
) -> Result<(), Error<'gc>> {
    let (_, mut global, mut domain) = script.init();
    let name = QName::new(Namespace::package(package), name);
    domain.export_definition(name, script, mc)?;
    global.install_const_late(mc, name, value, class);

    Ok(())
}

fn namespace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    package: impl Into<AvmString<'gc>>,
    name: impl Into<AvmString<'gc>>,
    uri: impl Into<AvmString<'gc>>,
    script: Script<'gc>,
) -> Result<(), Error<'gc>> {
    let namespace = NamespaceObject::from_namespace(activation, Namespace::Namespace(uri.into()))?;
    constant(
        activation.context.gc_context,
        package,
        name,
        namespace.into(),
        script,
        activation.avm2().classes().namespace,
    )
}

macro_rules! avm2_system_class {
    ($field:ident, $activation:ident, $class:expr, $script:expr) => {
        let class_object = class($activation, $class, $script)?;

        let sc = $activation.avm2().system_classes.as_mut().unwrap();
        sc.$field = class_object;
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
) -> Result<(), Error<'gc>> {
    let mc = activation.context.gc_context;

    let globals = ScriptObject::custom_object(activation.context.gc_context, None, None);
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
    let object_proto = ScriptObject::custom_object(mc, Some(object_class), None);

    let fn_classdef = function::create_class(mc);
    let fn_class = ClassObject::from_class_partial(activation, fn_classdef, Some(object_class))?;
    let fn_proto = ScriptObject::custom_object(mc, Some(fn_class), Some(object_proto));

    let class_classdef = class::create_class(mc);
    let class_class =
        ClassObject::from_class_partial(activation, class_classdef, Some(object_class))?;
    let class_proto = ScriptObject::custom_object(mc, Some(object_class), Some(object_proto));

    let global_classdef = global_scope::create_class(mc);
    let global_class =
        ClassObject::from_class_partial(activation, global_classdef, Some(object_class))?;
    let global_proto = ScriptObject::custom_object(mc, Some(object_class), Some(object_proto));

    // Now to weave the Gordian knot...
    object_class.link_prototype(activation, object_proto)?;
    object_class.link_type(activation, class_proto, class_class);

    fn_class.link_prototype(activation, fn_proto)?;
    fn_class.link_type(activation, class_proto, class_class);

    class_class.link_prototype(activation, class_proto)?;
    class_class.link_type(activation, class_proto, class_class);

    global_class.link_prototype(activation, global_proto)?;
    global_class.link_type(activation, class_proto, class_class);

    // At this point, we need at least a partial set of system classes in
    // order to continue initializing the player. The rest of the classes
    // are set to a temporary class until we have a chance to initialize them.

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

    globals.set_proto(mc, global_proto);
    globals.set_instance_of(mc, global_class);
    globals.fork_vtable(activation.context.gc_context);

    // From this point, `globals` is safe to be modified

    dynamic_class(mc, object_class, script, class_class)?;
    dynamic_class(mc, fn_class, script, class_class)?;
    dynamic_class(mc, class_class, script, class_class)?;

    // After this point, it is safe to initialize any other classes.
    // Make sure to initialize superclasses *before* their subclasses!

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

    class(activation, json::create_class(mc), script)?;
    avm2_system_class!(regexp, activation, regexp::create_class(mc), script);
    avm2_system_class!(vector, activation, vector::create_class(mc), script);

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
        flash::events::ieventdispatcher::create_interface(mc),
        script,
    )?;
    avm2_system_class!(
        eventdispatcher,
        activation,
        flash::events::eventdispatcher::create_class(mc),
        script
    );

    // package `flash.utils`

    class(
        activation,
        flash::utils::dictionary::create_class(mc),
        script,
    )?;

    namespace(
        activation,
        "flash.utils",
        "flash_proxy",
        flash::utils::NS_FLASH_PROXY,
        script,
    )?;
    class(activation, flash::utils::proxy::create_class(mc), script)?;

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

    // package `flash.geom`

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

    // package `flash.net`
    avm2_system_class!(
        sharedobject,
        activation,
        flash::net::sharedobject::create_class(mc),
        script
    );

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

    // package `flash.external`
    class(
        activation,
        flash::external::externalinterface::create_class(mc),
        script,
    )?;

    // Inside this call, the macro `avm2_system_classes_playerglobal`
    // triggers classloading. Therefore, we run `load_playerglobal`
    // relative late, so that it can access classes defined before
    // this call.
    load_playerglobal(activation, domain)?;

    Ok(())
}

/// This file is built by 'core/build_playerglobal/'
/// See that tool, and 'core/src/avm2/globals/README.md', for more details
const PLAYERGLOBAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/playerglobal.swf"));

mod native {
    include!(concat!(env!("OUT_DIR"), "/native_table.rs"));
}

/// Loads classes from our custom 'playerglobal' (which are written in ActionScript)
/// into the environment. See 'core/src/avm2/globals/README.md' for more information
fn load_playerglobal<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    domain: Domain<'gc>,
) -> Result<(), Error<'gc>> {
    activation.avm2().native_method_table = native::NATIVE_METHOD_TABLE;
    activation.avm2().native_instance_allocator_table = native::NATIVE_INSTANCE_ALLOCATOR_TABLE;
    activation.avm2().native_instance_init_table = native::NATIVE_INSTANCE_INIT_TABLE;

    let movie =
        SwfMovie::from_data(PLAYERGLOBAL, None, None).expect("playerglobal.swf should be valid");

    let slice = SwfSlice::from(Arc::new(movie));

    let mut reader = slice.read_from(0);

    let tag_callback = |reader: &mut SwfStream<'_>, tag_code, _tag_len| {
        if tag_code == TagCode::DoAbc {
            let do_abc = reader
                .read_do_abc()
                .expect("playerglobal.swf should be valid");
            Avm2::do_abc(&mut activation.context, do_abc, domain)
                .expect("playerglobal.swf should be valid");
        } else if tag_code != TagCode::End {
            panic!(
                "playerglobal should only contain `DoAbc` tag - found tag {:?}",
                tag_code
            )
        }
        Ok(ControlFlow::Continue)
    };

    let _ = tag_utils::decode_tags(&mut reader, tag_callback);
    macro_rules! avm2_system_classes_playerglobal {
        ($activation:expr, $script:expr, [$(($package:expr, $class_name:expr, $field:ident)),* $(,)?]) => {
            $(
                let name = Multiname::new(Namespace::package($package), $class_name);
                let class_object = activation.resolve_class(&name)?;
                let sc = $activation.avm2().system_classes.as_mut().unwrap();
                sc.$field = class_object;
            )*
        }
    }

    // This acts the same way as 'avm2_system_class', but for classes
    // declared in 'playerglobal'. Classes are declared as ("package", "class", field_name),
    // and are stored in 'avm2().system_classes'
    avm2_system_classes_playerglobal!(
        activation,
        script,
        [
            ("", "ArgumentError", argumenterror),
            ("", "RangeError", rangeerror),
            ("", "ReferenceError", referenceerror),
            ("", "TypeError", typeerror),
            ("", "VerifyError", verifyerror),
            ("", "XML", xml),
            ("", "XMLList", xml_list),
            ("flash.display", "Scene", scene),
            ("flash.display", "Stage3D", stage3d),
            ("flash.display3D", "Context3D", context3d),
            ("flash.display3D", "IndexBuffer3D", indexbuffer3d),
            ("flash.display3D", "Program3D", program3d),
            ("flash.display3D", "VertexBuffer3D", vertexbuffer3d),
            (
                "flash.errors",
                "IllegalOperationError",
                illegaloperationerror
            ),
            ("flash.errors", "IOError", ioerror),
            ("flash.errors", "EOFError", eoferror),
            ("flash.events", "Event", event),
            ("flash.events", "TextEvent", textevent),
            ("flash.events", "ErrorEvent", errorevent),
            ("flash.events", "KeyboardEvent", keyboardevent),
            ("flash.events", "ProgressEvent", progressevent),
            ("flash.events", "SecurityErrorEvent", securityerrorevent),
            ("flash.events", "IOErrorEvent", ioerrorevent),
            ("flash.events", "MouseEvent", mouseevent),
            ("flash.events", "FullScreenEvent", fullscreenevent),
            ("flash.events", "UncaughtErrorEvents", uncaughterrorevents),
            ("flash.geom", "Matrix", matrix),
            ("flash.geom", "Point", point),
            ("flash.geom", "Rectangle", rectangle),
            ("flash.geom", "Transform", transform),
            ("flash.geom", "ColorTransform", colortransform),
            ("flash.utils", "ByteArray", bytearray),
            ("flash.text", "StaticText", statictext),
            ("flash.text", "TextLineMetrics", textlinemetrics),
        ]
    );

    // Domain memory must be initialized after playerglobals is loaded because it relies on ByteArray.
    domain.init_default_domain_memory(activation)?;
    Ok(())
}
