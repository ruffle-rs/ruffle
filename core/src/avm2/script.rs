//! Whole script representation

use super::api_version::ApiVersion;
use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::error::Error;
use crate::avm2::globals::global_scope;
use crate::avm2::method::{Method, MethodAssociation};
use crate::avm2::object::{Object, ScriptObject, TObject};
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::vtable::VTable;
use crate::avm2::{Avm2, Multiname, Namespace};
use crate::context::UpdateContext;
use crate::string::{AvmAtom, AvmString, StringContext};
use crate::tag_utils::SwfMovie;
use crate::PlayerRuntime;
use gc_arena::barrier::field;
use gc_arena::lock::OnceLock;
use gc_arena::{Collect, Gc, Mutation};
use std::cell::Cell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use swf::avm2::types::{
    AbcFile, Index, Method as AbcMethod, Multiname as AbcMultiname, Namespace as AbcNamespace,
    Script as AbcScript,
};

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct TranslationUnit<'gc>(Gc<'gc, TranslationUnitData<'gc>>);

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
    classes: Box<[OnceLock<Class<'gc>>]>,

    /// All methods loaded from the ABC's method list.
    methods: Box<[OnceLock<Method<'gc>>]>,

    /// All scripts loaded from the ABC's scripts list.
    scripts: Box<[OnceLock<Script<'gc>>]>,

    /// All strings loaded from the ABC's strings list.
    /// They're lazy loaded and offset by 1, with the 0th element being always the empty string.
    strings: Box<[OnceLock<AvmAtom<'gc>>]>,

    /// All namespaces loaded from the ABC's scripts list.
    namespaces: Box<[OnceLock<Namespace<'gc>>]>,

    /// All multinames loaded from the ABC's multiname list
    /// Note that some of these may have a runtime (lazy) component.
    /// Make sure to check for that before using them.
    multinames: Box<[OnceLock<Gc<'gc, Multiname<'gc>>>]>,

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
        use std::iter::repeat_n;
        let this = TranslationUnitData {
            domain,
            name,
            classes: repeat_n(OnceLock::new(), abc.classes.len()).collect(),
            methods: repeat_n(OnceLock::new(), abc.methods.len()).collect(),
            scripts: repeat_n(OnceLock::new(), abc.scripts.len()).collect(),
            strings: repeat_n(OnceLock::new(), abc.constant_pool.strings.len() + 1).collect(),
            namespaces: repeat_n(OnceLock::new(), abc.constant_pool.namespaces.len() + 1).collect(),
            multinames: repeat_n(OnceLock::new(), abc.constant_pool.multinames.len() + 1).collect(),
            movie,
            abc: Rc::new(abc),
        };

        Self(Gc::new(mc, this))
    }

    pub fn load_classes(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        // Classes must be loaded in the order they appear in the constant pool,
        // to ensure that superclasses are loaded before subclasses

        let num_classes = self.0.classes.len();
        for i in 0..num_classes {
            let class = self.load_class(i as u32, activation)?;

            // NOTE: There are subtle differences between how a class is initially exported (here),
            // and how it's exported again when it is encountered in a trait (see `Script::load_traits`).
            // We currently don't handle them and just export it in the domain in both cases.
            self.domain()
                .export_class(class.name(), class, activation.gc());
        }

        Ok(())
    }

    /// Manually set a loaded class in this TranslationUnit. This is useful for
    /// early class setup.
    pub fn set_class(self, mc: &Mutation<'gc>, index: usize, class: Class<'gc>) {
        let classes = field!(Gc::write(mc, self.0), TranslationUnitData, classes).as_deref();
        classes[index].unlock().set(class).unwrap();
    }

    pub fn domain(self) -> Domain<'gc> {
        self.0.domain
    }

    // Retrieve the name associated with the original `DoAbc2` tag
    pub fn name(self) -> Option<AvmString<'gc>> {
        self.0.name
    }

    /// Retrieve the underlying `AbcFile` for this translation unit.
    pub fn abc(self) -> Rc<AbcFile> {
        self.0.abc.clone()
    }

    pub fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.clone()
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
        let idx = method_index.0 as usize;
        if let Some(method) = self.0.methods.get(idx).and_then(|m| m.get()) {
            return Ok(*method);
        }

        let write = Gc::write(activation.gc(), self.0);
        let methods = field!(write, TranslationUnitData, methods).as_deref();
        let method = Method::from_method_index(self, method_index, is_function, activation)?;
        methods[idx].unlock().set(method).unwrap();
        Ok(method)
    }

    /// Load a class from the ABC file and return its class definition.
    pub fn load_class(
        self,
        class_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Class<'gc>, Error<'gc>> {
        let idx = class_index as usize;
        if let Some(class) = self.0.classes.get(idx).and_then(|c| c.get()) {
            return Ok(*class);
        }

        let write = Gc::write(activation.gc(), self.0);
        let classes = field!(write, TranslationUnitData, classes).as_deref();
        let class = Class::from_abc_index(self, class_index, activation)?;
        classes[idx].unlock().set(class).unwrap();

        class.load_traits(activation, self, class_index)?;

        let c_class = class
            .c_class()
            .expect("Class::from_abc_index returns an i_class");

        class.validate_class(activation, false)?;
        c_class.validate_class(activation, false)?;

        class.init_vtable(activation)?;
        c_class.init_vtable(activation)?;

        Ok(class)
    }

    /// Load a script from the ABC file and return its script definition.
    pub fn load_script(
        self,
        script_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Script<'gc>, Error<'gc>> {
        let idx = script_index as usize;
        if let Some(scripts) = self.0.scripts.get(idx).and_then(|s| s.get()) {
            return Ok(*scripts);
        }

        let write = Gc::write(activation.gc(), self.0);
        let scripts = field!(write, TranslationUnitData, scripts).as_deref();
        let script = Script::from_abc_index(self, script_index, self.0.domain, activation)?;
        scripts[idx].unlock().set(script).unwrap();
        Ok(script)
    }

    /// Gets a script in the ABC file by index.
    pub fn get_script(self, index: usize) -> Option<Script<'gc>> {
        self.0.scripts.get(index).and_then(|s| s.get()).copied()
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
        context: &mut StringContext<'gc>,
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
        context: &mut StringContext<'gc>,
    ) -> Result<AvmAtom<'gc>, Error<'gc>> {
        let idx = string_index as usize;
        if let Some(atom) = self.0.strings.get(idx).and_then(|a| a.get()) {
            return Ok(*atom);
        }

        let raw = if string_index == 0 {
            &[]
        } else {
            self.0
                .abc
                .constant_pool
                .strings
                .get(idx - 1)
                .ok_or_else(|| format!("Unknown string constant {string_index}"))?
                .as_slice()
        };

        let atom = context.intern_wstr(ruffle_wstr::from_utf8_bytes(raw));

        let write = Gc::write(context.gc(), self.0);
        let strings = field!(write, TranslationUnitData, strings).as_deref();
        strings[idx].unlock().set(atom).unwrap();
        Ok(atom)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function treats index 0 as an error condition.
    pub fn pool_namespace(
        self,
        activation: &mut Activation<'_, 'gc>,
        ns_index: Index<AbcNamespace>,
    ) -> Result<Namespace<'gc>, Error<'gc>> {
        let idx = ns_index.0 as usize;
        if let Some(namespace) = self.0.namespaces.get(idx).and_then(|ns| ns.get()) {
            return Ok(*namespace);
        }

        let write = Gc::write(activation.gc(), self.0);
        let namespaces = field!(write, TranslationUnitData, namespaces).as_deref();
        let namespace = Namespace::from_abc_namespace(activation, self, ns_index)?;
        namespaces[idx].unlock().set(namespace).unwrap();
        Ok(namespace)
    }

    /// Retrieve a multiname from the current constant pool.
    /// The name can have a lazy component, do not pass it anywhere.
    pub fn pool_maybe_uninitialized_multiname(
        self,
        activation: &mut Activation<'_, 'gc>,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<Gc<'gc, Multiname<'gc>>, Error<'gc>> {
        let idx = multiname_index.0 as usize;
        if let Some(multiname) = self.0.multinames.get(idx).and_then(|mn| mn.get()) {
            return Ok(*multiname);
        }

        let write = Gc::write(activation.gc(), self.0);
        let multinames = field!(write, TranslationUnitData, multinames).as_deref();
        let multiname = Multiname::from_abc_index(activation, self, multiname_index)?;
        let multiname = Gc::new(activation.gc(), multiname);
        multinames[idx].unlock().set(multiname).unwrap();
        Ok(multiname)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function treats index 0 as an error condition.
    pub fn pool_multiname_static(
        self,
        activation: &mut Activation<'_, 'gc>,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<Gc<'gc, Multiname<'gc>>, Error<'gc>> {
        let multiname = self.pool_maybe_uninitialized_multiname(activation, multiname_index)?;
        if multiname.has_lazy_component() {
            return Err(format!("Multiname {} is not static", multiname_index.0).into());
        }

        Ok(multiname)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function returns None for index 0.
    pub fn pool_multiname_static_any(
        self,
        activation: &mut Activation<'_, 'gc>,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<Option<Gc<'gc, Multiname<'gc>>>, Error<'gc>> {
        if multiname_index.0 == 0 {
            Ok(None)
        } else {
            self.pool_multiname_static(activation, multiname_index)
                .map(Some)
        }
    }
}

/// A loaded Script from an ABC file.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct Script<'gc>(pub Gc<'gc, ScriptData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ScriptData<'gc> {
    /// The global object for the script.
    globals: Object<'gc>,

    /// The domain associated with this script.
    domain: Domain<'gc>,

    /// The initializer method to run for the script.
    init: Method<'gc>,

    /// Whether or not script initialization occurred.
    initialized: Cell<bool>,

    /// The `TranslationUnit` this script was loaded from.
    translation_unit: TranslationUnit<'gc>,
}

impl<'gc> Script<'gc> {
    /// Construct a script from a `TranslationUnit` and its script index.
    ///
    /// The returned script will be allocated, and its traits will be loaded.
    /// The caller is responsible for storing the class in the `TranslationUnit`.
    pub fn from_abc_index(
        unit: TranslationUnit<'gc>,
        script_index: u32,
        domain: Domain<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let abc = unit.abc();
        let script = abc
            .scripts
            .get(script_index as usize)
            .expect("Script index should be valid");

        let init = unit.load_method(script.init_method, false, activation)?;

        let globals = Script::create_globals_object(unit, script, domain, init, activation)?;

        let created_script = Self(Gc::new(
            activation.gc(),
            ScriptData {
                globals,
                domain,
                init,
                initialized: Cell::new(false),
                translation_unit: unit,
            },
        ));

        // Export script traits in domain now that the Script is created
        for trait_ in created_script.global_class().traits() {
            domain.export_definition(trait_.name(), created_script, activation.gc());
        }

        Ok(created_script)
    }

    /// Finish the script-loading process by loading traits and creating a global object.
    fn create_globals_object(
        unit: TranslationUnit<'gc>,
        script: &AbcScript,
        domain: Domain<'gc>,
        init_method: Method<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let mc = activation.gc();

        let traits: Box<[Trait<'_>]> = script
            .traits
            .iter()
            .map(|abc_trait| Trait::from_abc_trait(unit, abc_trait, activation))
            .collect::<Result<_, _>>()?;

        for newtrait in &traits {
            if let TraitKind::Class { class, .. } = newtrait.kind() {
                domain.export_class(newtrait.name(), *class, mc);
            }
        }

        // Now that we have the traits, create the global class for this script
        // and use it to initialize a vtable and global object.

        let global_class = global_scope::create_class(activation, init_method, traits)?;

        let scope = ScopeChain::new(domain);
        let object_class = activation.avm2().classes().object;

        let global_obj_vtable = VTable::new(
            global_class,
            Some(object_class),
            Some(scope),
            Some(object_class.instance_vtable()),
            mc,
        );

        // Script initializers are always run in "interpreter mode"
        let script_init_assoc = MethodAssociation::classbound(global_class, true);
        init_method.associate(activation, script_init_assoc)?;

        // Associate all the methods on the script
        global_class.bind_methods(
            activation,
            MethodAssociation::classbound(global_class, false),
        )?;

        Ok(ScriptObject::custom_object(
            mc,
            global_class,
            object_class.proto(), // Just use Object's prototype
            global_obj_vtable,
        ))
    }

    /// Return the entrypoint for the script and the scope it should run in.
    pub fn init(self) -> (Method<'gc>, Object<'gc>, Domain<'gc>) {
        (self.0.init, self.0.globals, self.0.domain)
    }

    pub fn domain(self) -> Domain<'gc> {
        self.0.domain
    }

    pub fn translation_unit(self) -> TranslationUnit<'gc> {
        self.0.translation_unit
    }

    pub fn global_class(self) -> Class<'gc> {
        self.0.globals.instance_class()
    }

    /// Return the global scope for the script.
    ///
    /// If the script has not yet been initialized, this will initialize it on
    /// the same stack.
    #[inline]
    pub fn globals(self, context: &mut UpdateContext<'gc>) -> Result<Object<'gc>, Error<'gc>> {
        if !self.0.initialized.get() {
            self.0.initialized.set(true);

            Avm2::run_script_initializer(self, context)?;
        }

        Ok(self.0.globals)
    }
}

impl Debug for Script<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Script")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}
