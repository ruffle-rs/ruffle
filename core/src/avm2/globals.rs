use crate::avm2::activation::Activation;
use crate::avm2::api_version::ApiVersion;
use crate::avm2::class::{BuiltinType, Class};
use crate::avm2::domain::Domain;
use crate::avm2::object::{ClassObject, ScriptObject};
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::script::TranslationUnit;
use crate::avm2::{Avm2, Error, Multiname, Namespace, QName};
use crate::string::WStr;
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
    pub definitionerror: ClassObject<'gc>,
    pub uninitializederror: ClassObject<'gc>,
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
    pub worker: ClassObject<'gc>,
    pub workerdomain: ClassObject<'gc>,
    pub messagechannel: ClassObject<'gc>,
    pub securitydomain: ClassObject<'gc>,
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

    // Vector.<Number> aka Vector$double
    pub number_vector: Class<'gc>,
    // Vector.<int> aka Vector$int
    pub int_vector: Class<'gc>,
    // Vector.<uint> aka Vector$uint
    pub uint_vector: Class<'gc>,
    // Vector.<*> aka Vector$object
    pub object_vector: Class<'gc>,

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
    pub urlrequestheader: Class<'gc>,
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
            definitionerror: object,
            uninitializederror: object,
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
            worker: object,
            workerdomain: object,
            messagechannel: object,
            securitydomain: object,
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

            number_vector: object,
            int_vector: object,
            uint_vector: object,
            object_vector: object,

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
            urlrequestheader: object,
            contextmenuitem: object,
        }
    }
}

/// Setup the `Object`, `Class`, and `void` classes, which are special "early
/// classes". This step of VM initialization must be done before everything else,
/// including the construction of the first `Script` and loading of any other classes.
pub fn init_early_classes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    tunit: TranslationUnit<'gc>,
) -> Result<(), Error<'gc>> {
    // We know that Object is class #0 and Class is class #1 in the builtin ABC
    const OBJECT_IDX: u32 = 0;
    const CLASS_IDX: u32 = 1;

    let mc = activation.gc();

    // We need to load `Object`'s `i_class` before we do anything else, even
    // initialize the script.
    // Object's i_class has no superclass, so we load it first.
    let object_i_class = Class::instance_from_abc_index(tunit, OBJECT_IDX, activation)?;
    object_i_class.load_instance_traits(activation, tunit, OBJECT_IDX)?;
    object_i_class.init_vtable(activation)?;

    // We're going to need the `Object` class registered in the domain for the
    // `Class` class to load. These will be overwritten when we properly load
    // the rest of the classes, but it doesn't matter since it'll be overwritten
    // with the same class anyways.
    activation
        .domain()
        .export_class(object_i_class.name(), object_i_class, mc);

    // Now we can load `Class`'s `i_class`:
    let class_i_class = Class::instance_from_abc_index(tunit, CLASS_IDX, activation)?;
    class_i_class.load_instance_traits(activation, tunit, CLASS_IDX)?;
    class_i_class.init_vtable(activation)?;

    // Register the `Class` class in the domain
    activation
        .domain()
        .export_class(class_i_class.name(), class_i_class, mc);

    // Now we can load the `c_class`es for `Object` and `Class` safely.
    let object_c_class = Class::class_from_abc_index(tunit, OBJECT_IDX, class_i_class, activation)?;
    object_c_class.load_class_traits(activation, tunit, OBJECT_IDX)?;
    object_c_class.init_vtable(activation)?;

    let class_c_class = Class::class_from_abc_index(tunit, CLASS_IDX, class_i_class, activation)?;
    class_c_class.load_class_traits(activation, tunit, CLASS_IDX)?;
    class_c_class.init_vtable(activation)?;

    // Now we link the i_classes and c_classes with each other:
    object_i_class.link_with_c_class(mc, object_c_class);
    class_i_class.link_with_c_class(mc, class_c_class);

    // Set the classes on the TranslationUnit to prevent `TranslationUnit::load_class`
    // from creating duplicate classes for them
    tunit.set_class(mc, OBJECT_IDX as usize, object_i_class);
    tunit.set_class(mc, CLASS_IDX as usize, class_i_class);

    // Set up the `null` and `void` classes and initialize `SystemClasses`

    // This is a weird internal class in avmplus, but it allows for implementing
    // `describeType(null)` in a cleaner way
    let null_def = null::create_class(activation);

    // void doesn't have a ClassObject
    let void_def = void::create_class(activation);
    activation
        .domain()
        .export_class(void_def.name(), void_def, mc);

    activation.avm2().system_class_defs = Some(SystemClassDefs::new(
        object_i_class,
        class_i_class,
        null_def,
        void_def,
    ));

    // NOTE: We don't create correct outer ScopeChains for `Object` and `Class`
    // here. This could cause unexpected behavior in their code, but the current
    // AS method implementations don't ever use properties from the outer ScopeChain.
    // However, we do need to ensure that the outer ScopeChain isn't zero-sized.
    // We will need to replace this ScopeChain once the script is created.
    let dummy_object =
        ScriptObject::custom_object(mc, object_i_class, None, object_i_class.vtable());

    let empty_scope = ScopeChain::new(tunit.domain());
    let dummy_scope = empty_scope.chain(mc, &[Scope::new(dummy_object.into())]);
    activation.set_outer(dummy_scope);

    // Finally, we can actually create the ClassObjects for `Object` and `Class`.

    let object_class = ClassObject::from_class_minimal(activation, object_i_class, None);
    let object_proto =
        ScriptObject::custom_object(mc, object_i_class, None, object_class.instance_vtable());

    let class_class =
        ClassObject::from_class_minimal(activation, class_i_class, Some(object_class));
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

    // At this point, we need both early classes to be available in `SystemClasses`
    // in order to call `into_finished_class` on both ClassObjects.

    activation.avm2().system_classes = Some(SystemClasses::new(object_class, class_class));

    // Construct the `ClassObject`s. We will run the class initializers later.
    class_class.into_finished_class(activation);
    object_class.into_finished_class(activation);

    // We don't need to validate the classes, as we already know that the
    // `Object` and `Class` classes are valid

    // However, we do need to bind their methods
    object_class.bind_methods(activation)?;
    class_class.bind_methods(activation)?;

    // Reset the Activation's outer scope.
    activation.set_outer(empty_scope);

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

pub fn init_builtin_system_classes(activation: &mut Activation<'_, '_>) {
    // We don't include `Function` here because it registers itself manually
    // in its class initializer
    avm2_system_classes_playerglobal!(
        &mut *activation,
        [
            ("", "ArgumentError", argumenterror),
            ("", "Array", array),
            ("", "Boolean", boolean),
            ("", "DefinitionError", definitionerror),
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
            ("", "UninitializedError", uninitializederror),
            ("", "URIError", urierror),
            ("", "VerifyError", verifyerror),
            ("", "XML", xml),
            ("", "XMLList", xml_list),
            ("__AS3__.vec", "Vector", generic_vector),
        ]
    );

    crate::avm2::globals::vector::init_vector_class_objects(activation);
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

    // Mark all the special builtin classes; see the documentation on
    // `Class.builtin_type` for more information
    let class_defs = activation.avm2().class_defs();
    class_defs.int.mark_builtin_type(BuiltinType::Int);
    class_defs.uint.mark_builtin_type(BuiltinType::Uint);
    class_defs.number.mark_builtin_type(BuiltinType::Number);
    class_defs.boolean.mark_builtin_type(BuiltinType::Boolean);
    class_defs.object.mark_builtin_type(BuiltinType::Object);
    class_defs.string.mark_builtin_type(BuiltinType::String);
    class_defs.void.mark_builtin_type(BuiltinType::Void);

    crate::avm2::globals::vector::init_vector_class_defs(activation);
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
            ("flash.system", "MessageChannel", messagechannel),
            ("flash.system", "SecurityDomain", securitydomain),
            ("flash.system", "Worker", worker),
            ("flash.system", "WorkerDomain", workerdomain),
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
            ("flash.net", "URLRequestHeader", urlrequestheader),
            ("flash.ui", "ContextMenuItem", contextmenuitem),
        ]
    );
}

/// Loads classes from our custom 'playerglobal' (which are written in ActionScript)
/// into the environment. See 'core/src/avm2/globals/README.md' for more information
pub fn load_playerglobal<'gc>(activation: &mut Activation<'_, 'gc>, domain: Domain<'gc>) {
    activation.avm2().native_method_table = native::NATIVE_METHOD_TABLE;
    activation.avm2().native_instance_allocator_table = native::NATIVE_INSTANCE_ALLOCATOR_TABLE;
    activation.avm2().native_call_handler_table = native::NATIVE_CALL_HANDLER_TABLE;
    activation.avm2().native_custom_constructor_table = native::NATIVE_CUSTOM_CONSTRUCTOR_TABLE;
    activation.avm2().native_fast_call_list = native::NATIVE_FAST_CALL_LIST;

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
    domain.init_default_domain_memory(activation);
    activation
        .avm2()
        .stage_domain()
        .init_default_domain_memory(activation);
}
