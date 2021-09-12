//! Whole script representation

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::method::{BytecodeMethod, Method};
use crate::avm2::object::{DomainObject, Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use crate::string::AvmString;
use fnv::FnvHashMap;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use std::cell::Ref;
use std::mem::drop;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Index, Script as AbcScript};

#[derive(Copy, Clone, Debug, Collect)]
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
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct TranslationUnitData<'gc> {
    /// The domain that all scripts in the translation unit export defs to.
    domain: Domain<'gc>,

    /// The ABC file that all of the following loaded data comes from.
    #[collect(require_static)]
    abc: Rc<AbcFile>,

    /// All classes loaded from the ABC's class list.
    classes: FnvHashMap<u32, GcCell<'gc, Class<'gc>>>,

    /// All methods loaded from the ABC's method list.
    methods: FnvHashMap<u32, Method<'gc>>,

    /// All scripts loaded from the ABC's scripts list.
    scripts: FnvHashMap<u32, Script<'gc>>,

    /// All strings loaded from the ABC's strings list.
    strings: FnvHashMap<u32, AvmString<'gc>>,
}

impl<'gc> TranslationUnit<'gc> {
    /// Construct a new `TranslationUnit` for a given ABC file intended to
    /// execute within a particular domain.
    pub fn from_abc(abc: Rc<AbcFile>, domain: Domain<'gc>, mc: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(
            mc,
            TranslationUnitData {
                domain,
                abc,
                classes: FnvHashMap::default(),
                methods: FnvHashMap::default(),
                scripts: FnvHashMap::default(),
                strings: FnvHashMap::default(),
            },
        ))
    }

    /// Retrieve the underlying `AbcFile` for this translation unit.
    pub fn abc(self) -> Rc<AbcFile> {
        self.0.read().abc.clone()
    }

    /// Load a method from the ABC file and return its method definition.
    pub fn load_method(
        self,
        method_index: u32,
        is_function: bool,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Method<'gc>, Error> {
        let read = self.0.read();
        if let Some(method) = read.methods.get(&method_index) {
            return Ok(method.clone());
        }

        drop(read);

        let method: Result<Gc<'gc, BytecodeMethod<'gc>>, Error> = BytecodeMethod::from_method_index(
            self,
            Index::new(method_index),
            is_function,
            activation,
        );
        let method: Method<'gc> = method?.into();

        self.0
            .write(activation.context.gc_context)
            .methods
            .insert(method_index, method.clone());

        Ok(method)
    }

    /// Load a class from the ABC file and return its class definition.
    pub fn load_class(
        self,
        class_index: u32,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<GcCell<'gc, Class<'gc>>, Error> {
        let read = self.0.read();
        if let Some(class) = read.classes.get(&class_index) {
            return Ok(*class);
        }

        drop(read);

        let class = Class::from_abc_index(self, class_index, activation)?;
        self.0
            .write(activation.context.gc_context)
            .classes
            .insert(class_index, class);

        class
            .write(activation.context.gc_context)
            .load_traits(self, class_index, activation)?;

        Ok(class)
    }

    /// Load a script from the ABC file and return its script definition.
    pub fn load_script(
        self,
        script_index: u32,
        uc: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Script<'gc>, Error> {
        let read = self.0.read();
        if let Some(scripts) = read.scripts.get(&script_index) {
            return Ok(*scripts);
        }

        let mut domain = read.domain;

        drop(read);

        let mut activation = Activation::from_nothing(uc.reborrow());
        let global = DomainObject::script_global(&mut activation, domain)?;

        let mut script = Script::from_abc_index(self, script_index, global, &mut activation)?;
        self.0
            .write(activation.context.gc_context)
            .scripts
            .insert(script_index, script);

        script.load_traits(self, script_index, &mut activation)?;

        for traitdef in script.traits()?.iter() {
            domain.export_definition(
                traitdef.name().clone(),
                script,
                activation.context.gc_context,
            )?;
        }

        Ok(script)
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
        mc: MutationContext<'gc, '_>,
    ) -> Result<Option<AvmString<'gc>>, Error> {
        let mut write = self.0.write(mc);
        if let Some(string) = write.strings.get(&string_index) {
            return Ok(Some(*string));
        }

        if string_index == 0 {
            return Ok(None);
        }

        let avm_string = AvmString::new(
            mc,
            write
                .abc
                .constant_pool
                .strings
                .get(string_index as usize - 1)
                .ok_or_else(|| format!("Unknown string constant {}", string_index))?,
        );
        write.strings.insert(string_index, avm_string);

        Ok(Some(avm_string))
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
        mc: MutationContext<'gc, '_>,
    ) -> Result<AvmString<'gc>, Error> {
        Ok(self
            .pool_string_option(string_index, mc)?
            .unwrap_or_default())
    }
}

/// A loaded Script from an ABC file.
#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Script<'gc>(GcCell<'gc, ScriptData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
struct ScriptData<'gc> {
    /// The global scope for the script.
    globals: Object<'gc>,

    /// The initializer method to run for the script.
    init: Method<'gc>,

    /// Traits that this script uses.
    traits: Vec<Trait<'gc>>,

    /// Whether or not we loaded our traits.
    traits_loaded: bool,

    /// Whether or not script initialization occurred.
    initialized: bool,
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
    pub fn empty_script(mc: MutationContext<'gc, '_>, globals: Object<'gc>) -> Self {
        Self(GcCell::allocate(
            mc,
            ScriptData {
                globals,
                init: Method::from_builtin(
                    |_, _, _| Ok(Value::Undefined),
                    "<Built-in script initializer>",
                    mc,
                ),
                traits: Vec::new(),
                traits_loaded: true,
                initialized: false,
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
        globals: Object<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let abc = unit.abc();
        let script: Result<&AbcScript, Error> = abc
            .scripts
            .get(script_index as usize)
            .ok_or_else(|| "LoadError: Script index not valid".into());
        let script = script?;

        let init = unit.load_method(script.init_method.0, false, activation)?;

        Ok(Self(GcCell::allocate(
            activation.context.gc_context,
            ScriptData {
                globals,
                init,
                traits: Vec::new(),
                traits_loaded: false,
                initialized: false,
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
        &mut self,
        unit: TranslationUnit<'gc>,
        script_index: u32,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(activation.context.gc_context);

        if write.traits_loaded {
            return Ok(());
        }

        write.traits_loaded = true;

        let abc = unit.abc();
        let script: Result<_, Error> = abc
            .scripts
            .get(script_index as usize)
            .ok_or_else(|| "LoadError: Script index not valid".into());
        let script = script?;

        for abc_trait in script.traits.iter() {
            drop(write);

            let newtrait = Trait::from_abc_trait(unit, abc_trait, activation)?;

            write = self.0.write(activation.context.gc_context);
            write.traits.push(newtrait);
        }

        Ok(())
    }

    /// Return the entrypoint for the script and the scope it should run in.
    pub fn init(self) -> (Method<'gc>, Object<'gc>) {
        let read = self.0.read();
        (read.init.clone(), read.globals)
    }

    /// Return the global scope for the script.
    ///
    /// If the script has not yet been initialized, this will initialize it on
    /// the same stack.
    pub fn globals(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        let mut write = self.0.write(context.gc_context);
        if !write.initialized {
            write.initialized = true;

            let mut globals = write.globals;
            let mut null_activation = Activation::from_nothing(context.reborrow());

            drop(write);

            globals.install_traits(&mut null_activation, &self.traits()?)?;

            Avm2::run_script_initializer(*self, context)?;

            Ok(globals)
        } else {
            Ok(write.globals)
        }
    }

    /// Return traits for this script.
    ///
    /// This function will return an error if it is incorrectly called before
    /// traits are loaded.
    pub fn traits<'a>(&'a self) -> Result<Ref<'a, [Trait<'gc>]>, Error> {
        let read = self.0.read();

        if !read.traits_loaded {
            return Err("LoadError: Script traits accessed before they were loaded!".into());
        }

        Ok(Ref::map(read, |read| &read.traits[..]))
    }
}
