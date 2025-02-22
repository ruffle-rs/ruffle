use crate::avm2::activation::Activation;
use crate::avm2::api_version::ApiVersion;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::domain::Domain;
use crate::avm2::object::{ClassObject, ScriptObject, TObject};
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::script::Script;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::vtable::VTable;
use crate::avm2::{Avm2, Error, Multiname, Namespace, QName};
use crate::string::{AvmString, WStr};
use crate::tag_utils::{self, ControlFlow, SwfMovie, SwfSlice, SwfStream};
use gc_arena::Collect;
use std::sync::Arc;
use swf::TagCode;

mod __ruffle__;
mod array;
mod avmplus;
mod boolean;
mod class;
mod date;
mod error;
pub mod flash;
mod function;
pub mod global_scope;
mod int;
mod json;
mod math;
mod namespace;
mod null;
mod number;
mod object;
mod q_name;
mod reg_exp;
mod string;
mod toplevel;
mod r#uint;
mod vector;
mod vector_double;
mod vector_int;
mod vector_object;
mod vector_uint;
mod void;
mod xml;
mod xml_list;

pub use toplevel::decode_uri;
pub use toplevel::decode_uri_component;
pub use toplevel::encode_uri;
pub use toplevel::encode_uri_component;
pub use toplevel::escape;
pub use toplevel::is_finite;
pub use toplevel::is_na_n;
pub use toplevel::is_xml_name;
pub use toplevel::parse_float;
pub use toplevel::parse_int;
pub use toplevel::trace;
pub use toplevel::unescape;

/// This structure represents all system builtin classes.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SystemClasses<'gc> {
    pub object: ClassObject<'gc>,
    pub function: ClassObject<'gc>,
    pub class: ClassObject<'gc>,
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
    pub loader: ClassObject<'gc>,
    pub loaderinfo: ClassObject<'gc>,
    pub bytearray: ClassObject<'gc>,
    pub stage: ClassObject<'gc>,
    pub sprite: ClassObject<'gc>,
    pub simplebutton: ClassObject<'gc>,
    pub regexp: ClassObject<'gc>,
    // the generic Vector class, useless until you .apply() type arg onto it
    pub generic_vector: ClassObject<'gc>,
    // Vector.<*>, NOT Vector.<Object>. Used as base class for new Vector<T>.
    pub object_vector: ClassObject<'gc>,
    pub soundtransform: ClassObject<'gc>,
    pub soundchannel: ClassObject<'gc>,
    pub bitmap: ClassObject<'gc>,
    pub bitmapdata: ClassObject<'gc>,
    pub date: ClassObject<'gc>,
    pub qname: ClassObject<'gc>,
    pub mouseevent: ClassObject<'gc>,
    pub progressevent: ClassObject<'gc>,
    pub httpstatusevent: ClassObject<'gc>,
    pub textevent: ClassObject<'gc>,
    pub errorevent: ClassObject<'gc>,
    pub ioerrorevent: ClassObject<'gc>,
    pub securityerrorevent: ClassObject<'gc>,
    pub transform: ClassObject<'gc>,
    pub colortransform: ClassObject<'gc>,
    pub matrix: ClassObject<'gc>,
    pub matrix3d: ClassObject<'gc>,
    pub perspectiveprojection: ClassObject<'gc>,
    pub illegaloperationerror: ClassObject<'gc>,
    pub eventdispatcher: ClassObject<'gc>,
    pub rectangle: ClassObject<'gc>,
    pub keyboardevent: ClassObject<'gc>,
    pub point: ClassObject<'gc>,
    pub evalerror: ClassObject<'gc>,
    pub rangeerror: ClassObject<'gc>,
    pub referenceerror: ClassObject<'gc>,
    pub argumenterror: ClassObject<'gc>,
    pub syntaxerror: ClassObject<'gc>,
    pub typeerror: ClassObject<'gc>,
    pub verifyerror: ClassObject<'gc>,
    pub ioerror: ClassObject<'gc>,
    pub eoferror: ClassObject<'gc>,
    pub urierror: ClassObject<'gc>,
    pub securityerror: ClassObject<'gc>,
    pub error: ClassObject<'gc>,
    pub uncaughterrorevents: ClassObject<'gc>,
    pub statictext: ClassObject<'gc>,
    pub textlinemetrics: ClassObject<'gc>,
    pub stage3d: ClassObject<'gc>,
    pub context3d: ClassObject<'gc>,
    pub indexbuffer3d: ClassObject<'gc>,
    pub vertexbuffer3d: ClassObject<'gc>,
    pub program3d: ClassObject<'gc>,
    pub urlvariables: ClassObject<'gc>,
    pub bevelfilter: ClassObject<'gc>,
    pub bitmapfilter: ClassObject<'gc>,
    pub blurfilter: ClassObject<'gc>,
    pub colormatrixfilter: ClassObject<'gc>,
    pub convolutionfilter: ClassObject<'gc>,
    pub displacementmapfilter: ClassObject<'gc>,
    pub dropshadowfilter: ClassObject<'gc>,
    pub glowfilter: ClassObject<'gc>,
    pub gradientbevelfilter: ClassObject<'gc>,
    pub gradientglowfilter: ClassObject<'gc>,
    pub texture: ClassObject<'gc>,
    pub cubetexture: ClassObject<'gc>,
    pub rectangletexture: ClassObject<'gc>,
    pub morphshape: ClassObject<'gc>,
    pub shader: ClassObject<'gc>,
    pub shaderinput: ClassObject<'gc>,
    pub shaderparameter: ClassObject<'gc>,
    pub netstatusevent: ClassObject<'gc>,
    pub shaderfilter: ClassObject<'gc>,
    pub statusevent: ClassObject<'gc>,
    pub asyncerrorevent: ClassObject<'gc>,
    pub contextmenuevent: ClassObject<'gc>,
    pub filereference: ClassObject<'gc>,
    pub filefilter: ClassObject<'gc>,
    pub font: ClassObject<'gc>,
    pub textline: ClassObject<'gc>,
    pub sampledataevent: ClassObject<'gc>,
    pub avm1movie: ClassObject<'gc>,
    pub focusevent: ClassObject<'gc>,
    pub dictionary: ClassObject<'gc>,
    pub id3info: ClassObject<'gc>,
    pub textrun: ClassObject<'gc>,
    pub sharedobject: ClassObject<'gc>,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SystemClassDefs<'gc> {
    pub object: Class<'gc>,
    pub class: Class<'gc>,
    pub function: Class<'gc>,
    pub null: Class<'gc>,
    pub void: Class<'gc>,

    pub array: Class<'gc>,
    pub boolean: Class<'gc>,
    pub int: Class<'gc>,
    pub generic_vector: Class<'gc>,
    pub namespace: Class<'gc>,
    pub number: Class<'gc>,
    pub string: Class<'gc>,
    pub uint: Class<'gc>,
    pub xml: Class<'gc>,
    pub xml_list: Class<'gc>,

    pub bitmap: Class<'gc>,
    pub bitmapdata: Class<'gc>,
    pub igraphicsdata: Class<'gc>,
    pub graphicsbitmapfill: Class<'gc>,
    pub graphicsendfill: Class<'gc>,
    pub graphicsgradientfill: Class<'gc>,
    pub graphicspath: Class<'gc>,
    pub graphicstrianglepath: Class<'gc>,
    pub graphicssolidfill: Class<'gc>,
    pub graphicsshaderfill: Class<'gc>,
    pub graphicsstroke: Class<'gc>,
    pub cubetexture: Class<'gc>,
    pub rectangletexture: Class<'gc>,
    pub display_object: Class<'gc>,
    pub sprite: Class<'gc>,
    pub contextmenuitem: Class<'gc>,
}

impl<'gc> SystemClasses<'gc> {
    /// Construct a minimal set of system classes necessary for bootstrapping
    /// player globals.
    ///
    /// All other system classes aside from the two given here will be set to
    /// the empty object also handed to this function. It is the caller's
    /// responsibility to instantiate each class and replace the empty object
    /// with that.
    fn new(object: ClassObject<'gc>, class: ClassObject<'gc>) -> Self {
        SystemClasses {
            object,
            class,

            // temporary initialization
            function: object,
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
            loader: object,
            loaderinfo: object,
            bytearray: object,
            stage: object,
            sprite: object,
            simplebutton: object,
            regexp: object,
            generic_vector: object,
            object_vector: object,
            soundtransform: object,
            soundchannel: object,
            bitmap: object,
            bitmapdata: object,
            date: object,
            qname: object,
            mouseevent: object,
            progressevent: object,
            httpstatusevent: object,
            textevent: object,
            errorevent: object,
            ioerrorevent: object,
            securityerrorevent: object,
            transform: object,
            colortransform: object,
            matrix: object,
            matrix3d: object,
            perspectiveprojection: object,
            illegaloperationerror: object,
            eventdispatcher: object,
            rectangle: object,
            keyboardevent: object,
            point: object,
            evalerror: object,
            rangeerror: object,
            referenceerror: object,
            argumenterror: object,
            syntaxerror: object,
            typeerror: object,
            verifyerror: object,
            ioerror: object,
            eoferror: object,
            urierror: object,
            securityerror: object,
            error: object,
            uncaughterrorevents: object,
            statictext: object,
            textlinemetrics: object,
            stage3d: object,
            context3d: object,
            indexbuffer3d: object,
            vertexbuffer3d: object,
            program3d: object,
            urlvariables: object,
            bevelfilter: object,
            bitmapfilter: object,
            blurfilter: object,
            colormatrixfilter: object,
            convolutionfilter: object,
            displacementmapfilter: object,
            dropshadowfilter: object,
            glowfilter: object,
            gradientbevelfilter: object,
            gradientglowfilter: object,
            texture: object,
            cubetexture: object,
            rectangletexture: object,
            morphshape: object,
            shader: object,
            shaderinput: object,
            shaderparameter: object,
            netstatusevent: object,
            shaderfilter: object,
            statusevent: object,
            asyncerrorevent: object,
            contextmenuevent: object,
            filereference: object,
            filefilter: object,
            font: object,
            textline: object,
            sampledataevent: object,
            avm1movie: object,
            focusevent: object,
            dictionary: object,
            id3info: object,
            textrun: object,
            sharedobject: object,
        }
    }
}

impl<'gc> SystemClassDefs<'gc> {
    fn new(object: Class<'gc>, class: Class<'gc>, null: Class<'gc>, void: Class<'gc>) -> Self {
        SystemClassDefs {
            object,
            class,
            null,
            void,

            // temporary initialization
            array: object,
            boolean: object,
            int: object,
            function: object,
            generic_vector: object,
            namespace: object,
            number: object,
            string: object,
            uint: object,
            xml: object,
            xml_list: object,

            bitmap: object,
            bitmapdata: object,
            igraphicsdata: object,
            graphicsbitmapfill: object,
            graphicsendfill: object,
            graphicsgradientfill: object,
            graphicspath: object,
            graphicstrianglepath: object,
            graphicssolidfill: object,
            graphicsshaderfill: object,
            graphicsstroke: object,
            cubetexture: object,
            rectangletexture: object,
            display_object: object,
            sprite: object,
            contextmenuitem: object,
        }
    }
}

/// Looks up a function defined in the script domain, and defines it on the global object.
///
/// This expects the looked-up value to be a function.
fn define_fn_on_global<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &'static str,
    script: Script<'gc>,
) {
    let (_, global, domain) = script.init();
    let qname = QName::new(activation.avm2().namespaces.public_all(), name);
    let func = domain
        .get_defined_value(activation, qname)
        .expect("Function being defined on global should be defined in domain!");

    Value::from(global)
        .init_property(&qname.into(), func, activation)
        .expect("Should set property");
}

/// Add a fully-formed class object builtin to the global scope.
///
/// This allows the caller to pre-populate the class's prototype with dynamic
/// properties, if necessary.
fn dynamic_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class_object: ClassObject<'gc>,
    script: Script<'gc>,
) {
    let (_, global, domain) = script.init();
    let class = class_object.inner_class_definition();
    let name = class.name();

    Value::from(global)
        .init_property(&name.into(), class_object.into(), activation)
        .expect("Should set property");

    domain.export_definition(name, script, activation.gc())
}

/// Initialize the player global domain.
///
/// This should be called only once, to construct the global scope of the
/// player. It will return a list of prototypes it has created, which should be
/// stored on the AVM. All relevant declarations will also be attached to the
/// given domain.
pub fn load_player_globals<'gc>(
    activation: &mut Activation<'_, 'gc>,
    domain: Domain<'gc>,
) -> Result<(), Error<'gc>> {
    let mc = activation.gc();

    // Set the outer scope of this activation to the global scope.

    // public / root package
    //
    // This part of global initialization is very complicated, because
    // everything has to circularly reference everything else:
    //
    //  - Object is an instance of itself, as well as its prototype
    //  - All types are instances of Class, which is an instance of itself
    //  - Object has prototype methods, but creating them requires the Function
    //    class, and creating the Function class requires the Object class to exist
    //
    // Hence, this ridiculously complicated dance of classdef, type allocation,
    // and partial initialization.

    // Object extends nothing
    let object_i_class = object::create_i_class(activation);

    // Class extends Object
    let class_i_class = class::create_i_class(activation, object_i_class);

    // Object$ extends Class
    let object_c_class = object::create_c_class(activation, class_i_class);
    object_i_class.set_c_class(mc, object_c_class);
    object_c_class.set_i_class(mc, object_i_class);

    // Class$ extends Class
    let class_c_class = class::create_c_class(activation, class_i_class);
    class_i_class.set_c_class(mc, class_c_class);
    class_c_class.set_i_class(mc, class_i_class);

    // This is a weird internal class in avmplus, but it allows for implementing
    // `describeType(null)` in a cleaner way
    let null_def = null::create_class(activation);

    // void doesn't have a ClassObject
    let void_def = void::create_class(activation);

    // Register the classes in the domain, now (except for the global and null classes)
    domain.export_class(object_i_class.name(), object_i_class, mc);
    domain.export_class(class_i_class.name(), class_i_class, mc);
    domain.export_class(void_def.name(), void_def, mc);

    activation.context.avm2.system_class_defs = Some(SystemClassDefs::new(
        object_i_class,
        class_i_class,
        null_def,
        void_def,
    ));

    // Unfortunately we need to specify the global traits manually, at least until
    // all the builtin classes are defined in AS.
    let mut global_traits = Vec::new();

    let public_ns = activation.avm2().namespaces.public_all();

    let class_trait_list = &[
        (public_ns, "Object", object_i_class),
        (public_ns, "Class", class_i_class),
    ];

    // "trace" is the only builtin function not defined on the toplevel global object
    let function_trait_list = &[
        "decodeURI",
        "decodeURIComponent",
        "encodeURI",
        "encodeURIComponent",
        "escape",
        "unescape",
        "isXMLName",
        "isFinite",
        "isNaN",
        "parseFloat",
        "parseInt",
    ];

    for (namespace, name, class) in class_trait_list {
        let qname = QName::new(*namespace, *name);

        global_traits.push(Trait::from_class(qname, *class));
    }

    for function_name in function_trait_list {
        let qname = QName::new(public_ns, *function_name);

        // FIXME: These should be TraitKind::Methods, to match how they are when
        // defined on the AS global object, but we don't have the actual Methods
        // right now.
        global_traits.push(Trait::from_const(
            qname,
            Some(activation.avm2().multinames.function),
            Some(Value::Null),
        ));
    }

    // Create the builtin globals' classdef
    let global_classdef = global_scope::create_class(activation, global_traits);

    // Initialize the global object. This gives it a temporary vtable until the
    // global ClassObject is constructed and we have the true vtable; nothing actually
    // operates on the global object until it gets its true vtable, so this should
    // be fine.
    let globals = ScriptObject::custom_object(mc, global_classdef, None, global_classdef.vtable());

    let scope = ScopeChain::new(domain);
    let gs = scope.chain(mc, &[Scope::new(globals.into())]);
    activation.set_outer(gs);

    let object_class = ClassObject::from_class_partial(activation, object_i_class, None);
    let object_proto =
        ScriptObject::custom_object(mc, object_i_class, None, object_class.instance_vtable());

    let class_class =
        ClassObject::from_class_partial(activation, class_i_class, Some(object_class));
    let class_proto = ScriptObject::custom_object(
        mc,
        object_i_class,
        Some(object_proto),
        object_class.instance_vtable(),
    );

    // Now to weave the Gordian knot...
    object_class.link_prototype(activation, object_proto);
    object_class.link_type(mc, class_proto);

    class_class.link_prototype(activation, class_proto);
    class_class.link_type(mc, class_proto);

    // At this point, we need at least a partial set of system classes in
    // order to continue initializing the player. The rest of the classes
    // are set to a temporary class until we have a chance to initialize them.

    activation.context.avm2.system_classes = Some(SystemClasses::new(object_class, class_class));

    // Our activation environment is now functional enough to finish
    // initializing the core class weave. We need to initialize superclasses
    // (e.g. `Object`) before subclasses, so that `into_finished_class` can
    // copy traits from the initialized superclass vtable.

    // Construct the `ClassObject`s, starting with `Class`. This ensures
    // that the `prototype` property of `Class` gets copied into the *class*
    // vtable for `Object`.
    let class_class = class_class.into_finished_class(activation)?;
    let object_class = object_class.into_finished_class(activation)?;

    // Object prototype is enough
    globals.set_proto(mc, object_class.prototype());

    // Create a new, full vtable for globals.
    let global_obj_vtable = VTable::empty(mc);
    global_obj_vtable.init_vtable(
        global_classdef,
        Some(object_class),
        Some(scope),
        Some(object_class.instance_vtable()),
        mc,
    );

    globals.set_vtable(mc, global_obj_vtable);

    activation.context.avm2.toplevel_global_object = Some(globals);

    // Initialize the script
    let script = Script::empty_script(mc, globals, domain);

    // From this point, `globals` is safe to be modified

    dynamic_class(activation, object_class, script);
    dynamic_class(activation, class_class, script);

    // After this point, it is safe to initialize any other classes.

    // Inside this call, the macro `avm2_system_classes_playerglobal`
    // triggers classloading. Therefore, we run `load_playerglobal`
    // relatively late, so that it can access classes defined before
    // this call.
    load_playerglobal(activation, domain)?;

    for function_name in function_trait_list {
        // Now copy those functions to the global object.
        define_fn_on_global(activation, function_name, script);
    }

    Ok(())
}

/// This file is built by 'core/build_playerglobal/'
/// See that tool, and 'core/src/avm2/globals/README.md', for more details
const PLAYERGLOBAL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/playerglobal.swf"));

mod native {
    include!(concat!(env!("OUT_DIR"), "/native_table.rs"));
}

// Allow accessing slots that are meant to be accessed natively by using
// avm2::globals::slots::*;
pub mod slots {
    pub use super::native::slots::*;
}

// Do the same for methods
pub mod methods {
    pub use super::native::methods::*;
}

// This acts the same way as 'avm2_system_class', but for classes
// declared in 'playerglobal'. Classes are declared as ("package", "class", field_name),
// and are stored in 'avm2().system_classes'
macro_rules! avm2_system_classes_playerglobal {
    ($activation:expr, [$(($package:expr, $class_name:expr, $field:ident)),* $(,)?]) => {
        let activation = $activation;
        $(
            // Package and class names are ASCII
            let package = WStr::from_units($package.as_bytes());
            let class_name = WStr::from_units($class_name.as_bytes());

            let package = activation.strings().intern_static(package);
            let class_name = activation.strings().intern_static(class_name);

            // Lookup with the highest version, so we we see all defined classes here
            let ns = Namespace::package(package, ApiVersion::VM_INTERNAL, activation.strings());
            let name = QName::new(ns, class_name);
            let class_object = activation.domain().get_defined_value(activation, name).unwrap_or_else(|e| panic!("Failed to lookup {name:?}: {e:?}"));
            let class_object = class_object.as_object().unwrap().as_class_object().unwrap();
            let sc = activation.avm2().system_classes.as_mut().unwrap();
            sc.$field = class_object;
        )*
    }
}

macro_rules! avm2_system_class_defs_playerglobal {
    ($activation:expr, [$(($package:expr, $class_name:expr, $field:ident)),* $(,)?]) => {
        let activation = $activation;
        $(
            // Package and class names are ASCII
            let package = WStr::from_units($package.as_bytes());
            let class_name = WStr::from_units($class_name.as_bytes());

            let package = activation.strings().intern_static(package);
            let class_name = activation.strings().intern_static(class_name);

            let domain = activation.domain();

            // Lookup with the highest version, so we we see all defined classes here
            let ns = Namespace::package(package, ApiVersion::VM_INTERNAL, activation.strings());
            let name = Multiname::new(ns, class_name);
            let class_def = domain.get_class(activation.context, &name).unwrap_or_else(|| panic!("Failed to lookup {name:?}"));
            let sc = activation.avm2().system_class_defs.as_mut().unwrap();
            sc.$field = class_def;
        )*
    }
}

/// Set up a builtin vector's Class. This will change its name, mark it as a
/// specialization of Vector, and set its class parameter to the passed
/// `param_class`. This function returns the vector Class.
fn setup_vector_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    old_name: &'static str,
    new_name: &'static str,
    param_class: Option<Class<'gc>>,
) -> Class<'gc> {
    let generic_vector_cls = activation.avm2().class_defs().generic_vector;

    let vector_ns = activation.avm2().namespaces.vector_internal;

    // First, lookup the class
    let old_name = activation
        .strings()
        .intern_static(WStr::from_units(old_name.as_bytes()));

    let vector_cls = activation
        .domain()
        .get_class(activation.context, &Multiname::new(vector_ns, old_name))
        .expect("Vector class should be defined");

    // Set its name to Vector.<T>
    let new_name = AvmString::new_utf8(activation.gc(), new_name);

    vector_cls.set_name(activation.gc(), QName::new(vector_ns, new_name));

    // Set its parameter to the given parameter and add it to the map of
    // applications on the generic vector Class
    vector_cls.set_param(activation.gc(), Some(param_class));
    generic_vector_cls.add_application(activation.gc(), param_class, vector_cls);

    vector_cls
}

/// Set up a builtin vector's ClassObject. This marks it as a specialization of
/// Vector. This function returns the vector ClassObject.
fn setup_vector_class_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    vector_name: &'static str,
    param_class: Option<Class<'gc>>,
) -> ClassObject<'gc> {
    let generic_vector_cls = activation.avm2().classes().generic_vector;

    let vector_ns = activation.avm2().namespaces.vector_internal;

    // `vector_name` should be ASCII
    let class_name = activation
        .strings()
        .intern_static(WStr::from_units(vector_name.as_bytes()));

    let value = activation
        .domain()
        .get_defined_value(activation, QName::new(vector_ns, class_name))
        .expect("Vector class should be defined");

    let vector_cls = value.as_object().unwrap().as_class_object().unwrap();

    generic_vector_cls.add_application(activation.gc(), param_class, vector_cls);

    vector_cls
}

pub fn init_builtin_system_classes(activation: &mut Activation<'_, '_>) {
    // We don't include `Function` here because it registers itself manually
    // in its class initializer
    avm2_system_classes_playerglobal!(
        &mut *activation,
        [
            ("", "ArgumentError", argumenterror),
            ("", "Array", array),
            ("", "Boolean", boolean),
            ("", "Error", error),
            ("", "EvalError", evalerror),
            ("", "int", int),
            ("", "Namespace", namespace),
            ("", "Number", number),
            ("", "QName", qname),
            ("", "RangeError", rangeerror),
            ("", "ReferenceError", referenceerror),
            ("", "SecurityError", securityerror),
            ("", "String", string),
            ("", "SyntaxError", syntaxerror),
            ("", "TypeError", typeerror),
            ("", "uint", uint),
            ("", "URIError", urierror),
            ("", "VerifyError", verifyerror),
            ("", "XML", xml),
            ("", "XMLList", xml_list),
            ("__AS3__.vec", "Vector", generic_vector),
        ]
    );

    // Register Vector$int/uint/Number/Object as being applications of the Vector ClassObject
    let number_cls = activation.avm2().class_defs().number;
    setup_vector_class_object(activation, "Vector$double", Some(number_cls));

    let int_cls = activation.avm2().class_defs().int;
    setup_vector_class_object(activation, "Vector$int", Some(int_cls));

    let uint_cls = activation.avm2().class_defs().uint;
    setup_vector_class_object(activation, "Vector$uint", Some(uint_cls));

    let object_vector = setup_vector_class_object(activation, "Vector$object", None);

    // Manually set the object vector class since it's in an internal namespace
    // (`avm2_system_classes_playerglobal` only works for classes in public namespaces)
    activation
        .avm2()
        .system_classes
        .as_mut()
        .unwrap()
        .object_vector = object_vector;
}

pub fn init_builtin_system_class_defs(activation: &mut Activation<'_, '_>) {
    avm2_system_class_defs_playerglobal!(
        &mut *activation,
        [
            ("", "Array", array),
            ("", "Boolean", boolean),
            ("", "Function", function),
            ("", "int", int),
            ("", "Namespace", namespace),
            ("", "Number", number),
            ("", "String", string),
            ("", "uint", uint),
            ("", "XML", xml),
            ("", "XMLList", xml_list),
            ("__AS3__.vec", "Vector", generic_vector),
        ]
    );

    // Mark Vector as a generic class
    let generic_vector = activation.avm2().class_defs().generic_vector;
    generic_vector.set_attributes(
        activation.gc(),
        ClassAttributes::GENERIC | ClassAttributes::FINAL,
    );

    // Setup the four builtin vector classes

    let number_cls = activation.avm2().class_defs().number;
    setup_vector_class(
        activation,
        "Vector$double",
        "Vector.<Number>",
        Some(number_cls),
    );

    let int_cls = activation.avm2().class_defs().int;
    setup_vector_class(activation, "Vector$int", "Vector.<int>", Some(int_cls));

    let uint_cls = activation.avm2().class_defs().uint;
    setup_vector_class(activation, "Vector$uint", "Vector.<uint>", Some(uint_cls));

    setup_vector_class(activation, "Vector$object", "Vector.<*>", None);
}

pub fn init_native_system_classes(activation: &mut Activation<'_, '_>) {
    avm2_system_classes_playerglobal!(
        &mut *activation,
        [
            ("", "Date", date),
            ("", "RegExp", regexp),
            ("flash.display", "AVM1Movie", avm1movie),
            ("flash.display", "Bitmap", bitmap),
            ("flash.display", "BitmapData", bitmapdata),
            ("flash.display", "Scene", scene),
            ("flash.display", "FrameLabel", framelabel),
            ("flash.display", "Graphics", graphics),
            ("flash.display", "Loader", loader),
            ("flash.display", "LoaderInfo", loaderinfo),
            ("flash.display", "MorphShape", morphshape),
            ("flash.display", "MovieClip", movieclip),
            ("flash.display", "ShaderInput", shaderinput),
            ("flash.display", "ShaderParameter", shaderparameter),
            ("flash.display", "Shape", shape),
            ("flash.display", "SimpleButton", simplebutton),
            ("flash.display", "Sprite", sprite),
            ("flash.display", "Stage", stage),
            ("flash.display", "Stage3D", stage3d),
            ("flash.display3D", "Context3D", context3d),
            ("flash.display3D", "IndexBuffer3D", indexbuffer3d),
            ("flash.display3D", "Program3D", program3d),
            ("flash.display3D.textures", "CubeTexture", cubetexture),
            ("flash.display3D.textures", "Texture", texture),
            (
                "flash.display3D.textures",
                "RectangleTexture",
                rectangletexture
            ),
            ("flash.display3D", "VertexBuffer3D", vertexbuffer3d),
            (
                "flash.errors",
                "IllegalOperationError",
                illegaloperationerror
            ),
            ("flash.errors", "IOError", ioerror),
            ("flash.errors", "EOFError", eoferror),
            ("flash.events", "Event", event),
            ("flash.events", "EventDispatcher", eventdispatcher),
            ("flash.events", "TextEvent", textevent),
            ("flash.events", "ErrorEvent", errorevent),
            ("flash.events", "KeyboardEvent", keyboardevent),
            ("flash.events", "ProgressEvent", progressevent),
            ("flash.events", "HTTPStatusEvent", httpstatusevent),
            ("flash.events", "SecurityErrorEvent", securityerrorevent),
            ("flash.events", "IOErrorEvent", ioerrorevent),
            ("flash.events", "MouseEvent", mouseevent),
            ("flash.events", "FullScreenEvent", fullscreenevent),
            ("flash.events", "UncaughtErrorEvents", uncaughterrorevents),
            ("flash.events", "NetStatusEvent", netstatusevent),
            ("flash.events", "StatusEvent", statusevent),
            ("flash.events", "AsyncErrorEvent", asyncerrorevent),
            ("flash.events", "ContextMenuEvent", contextmenuevent),
            ("flash.events", "FocusEvent", focusevent),
            ("flash.geom", "Matrix", matrix),
            ("flash.geom", "Matrix3D", matrix3d),
            ("flash.geom", "PerspectiveProjection", perspectiveprojection),
            ("flash.geom", "Point", point),
            ("flash.geom", "Rectangle", rectangle),
            ("flash.geom", "Transform", transform),
            ("flash.geom", "ColorTransform", colortransform),
            ("flash.media", "ID3Info", id3info),
            ("flash.media", "SoundChannel", soundchannel),
            ("flash.media", "SoundTransform", soundtransform),
            ("flash.media", "Video", video),
            ("flash.net", "URLVariables", urlvariables),
            ("flash.net", "FileReference", filereference),
            ("flash.net", "FileFilter", filefilter),
            ("flash.net", "SharedObject", sharedobject),
            ("flash.utils", "ByteArray", bytearray),
            ("flash.utils", "Dictionary", dictionary),
            ("flash.system", "ApplicationDomain", application_domain),
            ("flash.text", "Font", font),
            ("flash.text", "StaticText", statictext),
            ("flash.text", "TextFormat", textformat),
            ("flash.text", "TextField", textfield),
            ("flash.text", "TextLineMetrics", textlinemetrics),
            ("flash.text", "TextRun", textrun),
            ("flash.text.engine", "TextLine", textline),
            ("flash.filters", "BevelFilter", bevelfilter),
            ("flash.filters", "BitmapFilter", bitmapfilter),
            ("flash.filters", "BlurFilter", blurfilter),
            ("flash.filters", "ColorMatrixFilter", colormatrixfilter),
            ("flash.filters", "ConvolutionFilter", convolutionfilter),
            (
                "flash.filters",
                "DisplacementMapFilter",
                displacementmapfilter
            ),
            ("flash.filters", "DropShadowFilter", dropshadowfilter),
            ("flash.filters", "GlowFilter", glowfilter),
            ("flash.filters", "GradientBevelFilter", gradientbevelfilter),
            ("flash.filters", "GradientGlowFilter", gradientglowfilter),
            ("flash.filters", "ShaderFilter", shaderfilter),
            ("flash.events", "SampleDataEvent", sampledataevent),
        ]
    );

    avm2_system_class_defs_playerglobal!(
        &mut *activation,
        [
            ("flash.display", "Bitmap", bitmap),
            ("flash.display", "BitmapData", bitmapdata),
            ("flash.display", "DisplayObject", display_object),
            ("flash.display", "IGraphicsData", igraphicsdata),
            ("flash.display", "GraphicsBitmapFill", graphicsbitmapfill),
            ("flash.display", "GraphicsEndFill", graphicsendfill),
            (
                "flash.display",
                "GraphicsGradientFill",
                graphicsgradientfill
            ),
            ("flash.display", "GraphicsPath", graphicspath),
            (
                "flash.display",
                "GraphicsTrianglePath",
                graphicstrianglepath
            ),
            ("flash.display", "GraphicsSolidFill", graphicssolidfill),
            ("flash.display", "GraphicsStroke", graphicsstroke),
            ("flash.display", "Sprite", sprite),
            ("flash.display3D.textures", "CubeTexture", cubetexture),
            (
                "flash.display3D.textures",
                "RectangleTexture",
                rectangletexture
            ),
            ("flash.ui", "ContextMenuItem", contextmenuitem),
        ]
    );
}

/// Loads classes from our custom 'playerglobal' (which are written in ActionScript)
/// into the environment. See 'core/src/avm2/globals/README.md' for more information
fn load_playerglobal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    domain: Domain<'gc>,
) -> Result<(), Error<'gc>> {
    activation.avm2().native_method_table = native::NATIVE_METHOD_TABLE;
    activation.avm2().native_instance_allocator_table = native::NATIVE_INSTANCE_ALLOCATOR_TABLE;
    activation.avm2().native_call_handler_table = native::NATIVE_CALL_HANDLER_TABLE;
    activation.avm2().native_custom_constructor_table = native::NATIVE_CUSTOM_CONSTRUCTOR_TABLE;

    let movie = Arc::new(
        SwfMovie::from_data(PLAYERGLOBAL, "file:///".into(), None)
            .expect("playerglobal.swf should be valid"),
    );

    let slice = SwfSlice::from(movie.clone());

    let mut reader = slice.read_from(0);

    let tag_callback = |reader: &mut SwfStream<'_>, tag_code, _tag_len| {
        if tag_code == TagCode::DoAbc2 {
            let do_abc = reader
                .read_do_abc_2()
                .expect("playerglobal.swf should be valid");
            Avm2::load_builtin_abc(activation.context, do_abc.data, domain, movie.clone());
        } else if tag_code != TagCode::End {
            panic!("playerglobal should only contain `DoAbc2` tag - found tag {tag_code:?}")
        }
        Ok(ControlFlow::Continue)
    };

    let _ = tag_utils::decode_tags(&mut reader, tag_callback);

    // Domain memory must be initialized after playerglobals is loaded because it relies on ByteArray.
    domain.init_default_domain_memory(activation)?;
    activation
        .avm2()
        .stage_domain()
        .init_default_domain_memory(activation)?;
    Ok(())
}
