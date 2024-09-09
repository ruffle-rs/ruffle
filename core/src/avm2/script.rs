//! Whole script representation

use super::api_version::ApiVersion;
use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::globals::global_scope;
use crate::avm2::method::{BytecodeMethod, Method};
use crate::avm2::object::{Object, ScriptObject, TObject};
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::value::Value;
use crate::avm2::vtable::VTable;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::{Avm2, Error};
use crate::context::{GcContext, UpdateContext};
use crate::string::{AvmAtom, AvmString};
use crate::tag_utils::SwfMovie;
use crate::PlayerRuntime;
use gc_arena::{Collect, Gc, GcCell, Mutation};
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use swf::avm2::types::{
    AbcFile, Index, Method as AbcMethod, Multiname as AbcMultiname, Namespace as AbcNamespace,
    Script as AbcScript,
};

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct TranslationUnit<'gc>(GcCell<'gc, TranslationUnitData<'gc>>);

/// A loaded ABC file, with any loaded ABC items alongside it.
///
/// A `TranslationUnit` is constructed when ABC loading begins, and it stores
/// all loaded ABC items (classes, methods, and scripts) as they are loaded.
/// Unit items are loaded lazily and retained in the `TranslationUnit` for
/// later retrieval.
///
/// Loaded versions of ABC items consist of the types `Class`, `Method`, and
/// `Script`, all of which correspond to their `swf` equivalents, but with
/// names preloaded. This roughly corresponds to the logical "loading" phase of
/// ABC execution as documented in the AVM2 Overview. "Linking" takes place by
/// constructing the appropriate runtime object for that item.
#[derive(Clone, Collect)]
#[collect(no_drop)]
struct TranslationUnitData<'gc> {
    /// The domain that all scripts in the translation unit export defs to.
    domain: Domain<'gc>,

    /// The name from the original `DoAbc2` tag, or `None` if this came from a `DoAbc` tag
    name: Option<AvmString<'gc>>,

    /// The ABC file that all of the following loaded data comes from.
    #[collect(require_static)]
    abc: Rc<AbcFile>,

    /// All classes loaded from the ABC's class list.
    classes: Vec<Option<Class<'gc>>>,

    /// All methods loaded from the ABC's method list.
    methods: Vec<Option<Method<'gc>>>,

    /// All scripts loaded from the ABC's scripts list.
    scripts: Vec<Option<Script<'gc>>>,

    /// All strings loaded from the ABC's strings list.
    /// They're lazy loaded and offset by 1, with the 0th element being always the empty string.
    strings: Vec<Option<AvmAtom<'gc>>>,

    /// All namespaces loaded from the ABC's scripts list.
    namespaces: Vec<Option<Namespace<'gc>>>,

    /// All multinames loaded from the ABC's multiname list
    /// Note that some of these may have a runtime (lazy) component.
    /// Make sure to check for that before using them.
    multinames: Vec<Option<Gc<'gc, Multiname<'gc>>>>,

    /// The movie that this TranslationUnit was loaded from.
    movie: Arc<SwfMovie>,
}

impl<'gc> TranslationUnit<'gc> {
    /// Construct a new `TranslationUnit` for a given ABC file intended to
    /// execute within a particular domain.
    pub fn from_abc(
        abc: AbcFile,
        domain: Domain<'gc>,
        name: Option<AvmString<'gc>>,
        movie: Arc<SwfMovie>,
        mc: &Mutation<'gc>,
    ) -> Self {
        let classes = vec![None; abc.classes.len()];
        let methods = vec![None; abc.methods.len()];
        let scripts = vec![None; abc.scripts.len()];
        let strings = vec![None; abc.constant_pool.strings.len() + 1];
        let namespaces = vec![None; abc.constant_pool.namespaces.len() + 1];
        let multinames = vec![None; abc.constant_pool.multinames.len() + 1];

        Self(GcCell::new(
            mc,
            TranslationUnitData {
                domain,
                name,
                abc: Rc::new(abc),
                classes,
                methods,
                scripts,
                strings,
                namespaces,
                multinames,
                movie,
            },
        ))
    }

    pub fn load_classes(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        // Classes must be loaded in the order they appear in the constant pool,
        // to ensure that superclasses are loaded before subclasses

        let num_classes = self.0.read().classes.len();
        for i in 0..num_classes {
            let class = self.load_class(i as u32, activation)?;

            // NOTE: There are subtle differences between how a class is initially exported (here),
            // and how it's exported again when it is encountered in a trait (see `Script::load_traits`).
            // We currently don't handle them and just export it in the domain in both cases.
            self.domain()
                .export_class(class.name(), class, activation.context.gc_context);
        }

        Ok(())
    }

    pub fn domain(self) -> Domain<'gc> {
        self.0.read().domain
    }

    // Retrieve the name associated with the original `DoAbc2` tag
    pub fn name(self) -> Option<AvmString<'gc>> {
        self.0.read().name
    }

    /// Retrieve the underlying `AbcFile` for this translation unit.
    pub fn abc(self) -> Rc<AbcFile> {
        self.0.read().abc.clone()
    }

    pub fn movie(self) -> Arc<SwfMovie> {
        self.0.read().movie.clone()
    }

    pub fn api_version(self, avm2: &Avm2<'gc>) -> ApiVersion {
        if self.domain().is_playerglobals_domain(avm2) {
            // FIXME: get this from the player version we're emulating
            match avm2.player_runtime {
                PlayerRuntime::FlashPlayer => ApiVersion::SWF_31,
                PlayerRuntime::AIR => ApiVersion::AIR_20_0,
            }
        } else {
            avm2.root_api_version
        }
    }

    /// Load a method from the ABC file and return its method definition.
    pub fn load_method(
        self,
        method_index: Index<AbcMethod>,
        is_function: bool,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Method<'gc>, Error<'gc>> {
        let read = self.0.read();
        if let Some(Some(method)) = read.methods.get(method_index.0 as usize) {
            return Ok(*method);
        }

        let is_global = read.domain.is_playerglobals_domain(activation.avm2());
        drop(read);

        let bc_method =
            BytecodeMethod::from_method_index(self, method_index, is_function, activation)?;

        // This closure lets us move out of 'bc_method.signature' and then return,
        // allowing us to use 'bc_method' later on without a borrow-checker error.
        let method = (|| {
            if is_global {
                if let Some((name, native)) =
                    activation.avm2().native_method_table[method_index.0 as usize]
                {
                    assert_eq!(
                        bc_method.abc_method_body, None,
                        "Method in native method table has a bytecode body!"
                    );
                    let variadic = bc_method.is_variadic();
                    // Set the method name and function pointer from the table.
                    return Method::from_builtin_and_params(
                        native,
                        name,
                        bc_method.signature,
                        bc_method.return_type,
                        variadic,
                        activation.context.gc_context,
                    );
                }
            }
            Gc::new(activation.context.gc_context, bc_method).into()
        })();

        self.0.write(activation.context.gc_context).methods[method_index.0 as usize] = Some(method);

        Ok(method)
    }

    /// Load a class from the ABC file and return its class definition.
    pub fn load_class(
        self,
        class_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Class<'gc>, Error<'gc>> {
        let read = self.0.read();
        if let Some(Some(class)) = read.classes.get(class_index as usize) {
            return Ok(*class);
        }

        drop(read);

        let class = Class::from_abc_index(self, class_index, activation)?;
        self.0.write(activation.context.gc_context).classes[class_index as usize] = Some(class);

        class.load_traits(activation, self, class_index)?;

        class.init_vtable(activation.context)?;
        class
            .c_class()
            .expect("Class::from_abc_index returns an i_class")
            .init_vtable(activation.context)?;

        Ok(class)
    }

    /// Load a script from the ABC file and return its script definition.
    pub fn load_script(
        self,
        script_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Script<'gc>, Error<'gc>> {
        let read = self.0.read();
        if let Some(Some(scripts)) = read.scripts.get(script_index as usize) {
            return Ok(*scripts);
        }

        let domain = read.domain;

        drop(read);

        let script = Script::from_abc_index(self, script_index, domain, activation)?;
        self.0.write(activation.context.gc_context).scripts[script_index as usize] = Some(script);

        script.load_traits(self, script_index, activation)?;

        Ok(script)
    }

    /// Gets a script in the ABC file by index.
    pub fn get_script(&self, index: usize) -> Option<Script<'gc>> {
        self.0.read().scripts.get(index).copied().flatten()
    }

    /// Load a string from the ABC's constant pool.
    ///
    /// This function yields an error if no such string index exists.
    ///
    /// This function yields `None` to signal string index zero, which callers
    /// are free to interpret as the context demands.
    pub fn pool_string_option(
        self,
        string_index: u32,
        context: &mut GcContext<'_, 'gc>,
    ) -> Result<Option<AvmAtom<'gc>>, Error<'gc>> {
        if string_index == 0 {
            Ok(None)
        } else {
            self.pool_string(string_index, context).map(Some)
        }
    }

    /// Load a string from the ABC's constant pool.
    ///
    /// This function yields an error if no such string index exists.
    ///
    /// String index 0 is always `""`. If you need to instead treat 0 as
    /// something else, then please use `pool_string_option`.
    pub fn pool_string(
        self,
        string_index: u32,
        context: &mut GcContext<'_, 'gc>,
    ) -> Result<AvmAtom<'gc>, Error<'gc>> {
        let mut write = self.0.write(context.gc_context);
        if let Some(Some(atom)) = write.strings.get(string_index as usize) {
            return Ok(*atom);
        }

        let raw = if string_index == 0 {
            &[]
        } else {
            write
                .abc
                .constant_pool
                .strings
                .get(string_index as usize - 1)
                .ok_or_else(|| format!("Unknown string constant {string_index}"))?
                .as_slice()
        };

        let atom = context
            .interner
            .intern_wstr(context.gc_context, ruffle_wstr::from_utf8_bytes(raw));

        write.strings[string_index as usize] = Some(atom);
        Ok(atom)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function treats index 0 as an error condition.
    pub fn pool_namespace(
        self,
        ns_index: Index<AbcNamespace>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<Namespace<'gc>, Error<'gc>> {
        let read = self.0.read();
        if let Some(Some(namespace)) = read.namespaces.get(ns_index.0 as usize) {
            return Ok(*namespace);
        }

        drop(read);

        let namespace = Namespace::from_abc_namespace(self, ns_index, context)?;
        self.0.write(context.gc_context).namespaces[ns_index.0 as usize] = Some(namespace);

        Ok(namespace)
    }

    /// Retrieve a multiname from the current constant pool.
    /// The name can have a lazy component, do not pass it anywhere.
    pub fn pool_maybe_uninitialized_multiname(
        self,
        multiname_index: Index<AbcMultiname>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<Gc<'gc, Multiname<'gc>>, Error<'gc>> {
        let mc = context.gc_context;
        let read = self.0.read();
        if let Some(Some(multiname)) = read.multinames.get(multiname_index.0 as usize) {
            return Ok(*multiname);
        }

        drop(read);

        let multiname = Multiname::from_abc_index(self, multiname_index, context)?;
        let multiname = Gc::new(mc, multiname);
        self.0.write(mc).multinames[multiname_index.0 as usize] = Some(multiname);

        Ok(multiname)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function treats index 0 as an error condition.
    pub fn pool_multiname_static(
        self,
        multiname_index: Index<AbcMultiname>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<Gc<'gc, Multiname<'gc>>, Error<'gc>> {
        let multiname = self.pool_maybe_uninitialized_multiname(multiname_index, context)?;
        if multiname.has_lazy_component() {
            return Err(format!("Multiname {} is not static", multiname_index.0).into());
        }

        Ok(multiname)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function treats index 0 as the any-type `*`.
    pub fn pool_multiname_static_any(
        self,
        multiname_index: Index<AbcMultiname>,
        context: &mut UpdateContext<'gc>,
    ) -> Result<Gc<'gc, Multiname<'gc>>, Error<'gc>> {
        if multiname_index.0 == 0 {
            let mc = context.gc_context;
            Ok(Gc::new(mc, Multiname::any()))
        } else {
            self.pool_multiname_static(multiname_index, context)
        }
    }
}

/// A loaded Script from an ABC file.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct Script<'gc>(pub GcCell<'gc, ScriptData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ScriptData<'gc> {
    /// The global object for the script.
    globals: Option<Object<'gc>>,

    /// The domain associated with this script.
    domain: Domain<'gc>,

    /// The initializer method to run for the script.
    init: Method<'gc>,

    /// Whether or not we loaded our traits.
    traits_loaded: bool,

    /// Whether or not script initialization occurred.
    initialized: bool,

    /// The `TranslationUnit` this script was loaded from.
    translation_unit: Option<TranslationUnit<'gc>>,

    pub abc_index: Option<u32>,
}

impl<'gc> Script<'gc> {
    /// Create an empty script.
    ///
    /// This method is intended for builtin script initialization, such as our
    /// implementation of player globals. The builtin script initializer will
    /// be responsible for actually installing traits into both the script
    /// globals as well as the domain that this script is supposed to be a part
    /// of.
    ///
    /// The `globals` object should be constructed using the `global`
    /// prototype.
    pub fn empty_script(mc: &Mutation<'gc>, globals: Object<'gc>, domain: Domain<'gc>) -> Self {
        Self(GcCell::new(
            mc,
            ScriptData {
                globals: Some(globals),
                domain,
                init: Method::from_builtin(
                    |_, _, _| Ok(Value::Undefined),
                    "<Built-in script initializer>",
                    mc,
                ),
                traits_loaded: true,
                initialized: false,
                translation_unit: None,
                abc_index: None,
            },
        ))
    }

    /// Construct a script from a `TranslationUnit` and its script index.
    ///
    /// The returned script will be allocated, but no traits will be loaded.
    /// The caller is responsible for storing the class in the
    /// `TranslationUnit` and calling `load_traits` to complete the
    /// trait-loading process.
    ///
    /// The given `globals` should be an empty object of the `global` hidden
    /// type. The initializer script will create and store traits on it.
    pub fn from_abc_index(
        unit: TranslationUnit<'gc>,
        script_index: u32,
        domain: Domain<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let abc = unit.abc();
        let script: Result<&AbcScript, Error<'gc>> = abc
            .scripts
            .get(script_index as usize)
            .ok_or_else(|| "LoadError: Script index not valid".into());
        let script = script?;

        let init = unit.load_method(script.init_method, false, activation)?;

        Ok(Self(GcCell::new(
            activation.context.gc_context,
            ScriptData {
                globals: None,
                domain,
                init,
                traits_loaded: false,
                initialized: false,
                translation_unit: Some(unit),
                abc_index: Some(script_index),
            },
        )))
    }

    /// Finish the class-loading process by loading traits.
    ///
    /// This process must be done after the `Script` has been stored in the
    /// `TranslationUnit`. Failing to do so runs the risk of runaway recursion
    /// or double-borrows. It should be done before the script is actually
    /// executed.
    pub fn load_traits(
        self,
        unit: TranslationUnit<'gc>,
        script_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let mc = activation.gc();

        let mut write = self.0.write(mc);

        if write.traits_loaded {
            return Ok(());
        }

        write.traits_loaded = true;

        let abc = unit.abc();
        let script: Result<_, Error<'gc>> = abc
            .scripts
            .get(script_index as usize)
            .ok_or_else(|| "LoadError: Script index not valid".into());
        let script = script?;
        let mut domain = write.domain;

        let mut traits = Vec::new();

        for abc_trait in script.traits.iter() {
            let newtrait = Trait::from_abc_trait(unit, abc_trait, activation)?;
            domain.export_definition(newtrait.name(), self, mc);
            if let TraitKind::Class { class, .. } = newtrait.kind() {
                domain.export_class(newtrait.name(), *class, mc);
            }

            traits.push(newtrait);
        }

        drop(write);

        // Now that we have the traits, create the global class for this script
        // and use it to initialize a vtable and global object.

        let global_class = global_scope::create_class(activation, traits);

        let scope = ScopeChain::new(domain);
        let object_class = activation.avm2().classes().object;

        let global_obj_vtable = VTable::empty(mc);
        global_obj_vtable.init_vtable(
            global_class,
            Some(object_class),
            Some(scope),
            Some(object_class.instance_vtable()),
            mc,
        );

        let global_object = ScriptObject::custom_object(
            mc,
            global_class,
            object_class.proto(), // Just use Object's prototype
            global_obj_vtable,
        );

        self.0.write(mc).globals = Some(global_object);

        Ok(())
    }

    /// Return the entrypoint for the script and the scope it should run in.
    pub fn init(self) -> (Method<'gc>, Object<'gc>, Domain<'gc>) {
        let read = self.0.read();
        let globals = read.globals.expect("Global object should be initialized");

        (read.init, globals, read.domain)
    }

    pub fn domain(self) -> Domain<'gc> {
        self.0.read().domain
    }

    pub fn translation_unit(self) -> Option<TranslationUnit<'gc>> {
        self.0.read().translation_unit
    }

    pub fn traits_loaded(self) -> bool {
        self.0.read().traits_loaded
    }

    pub fn global_class(self) -> Class<'gc> {
        self.0
            .read()
            .globals
            .expect("Global object should be initialized")
            .instance_class()
    }

    /// Return the global scope for the script.
    ///
    /// If the script has not yet been initialized, this will initialize it on
    /// the same stack.
    pub fn globals(self, context: &mut UpdateContext<'gc>) -> Result<Object<'gc>, Error<'gc>> {
        let mut write = self.0.write(context.gc_context);

        let globals = write.globals.expect("Global object should be initialized");

        if !write.initialized {
            write.initialized = true;
            drop(write);

            Avm2::run_script_initializer(self, context)?;
        }

        Ok(globals)
    }
}

impl<'gc> Debug for Script<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Script")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}
