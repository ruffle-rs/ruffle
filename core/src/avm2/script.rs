//! Whole script representation

use crate::avm2::class::Class;
use crate::avm2::function::{Avm2MethodEntry, Method};
use crate::avm2::r#trait::Trait;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashMap;
use std::mem::drop;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Index, Script as AbcScript};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct CollectWrapper<T>(T);

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
    /// The ABC file that all of the following loaded data comes from.
    abc: CollectWrapper<Rc<AbcFile>>,

    /// All classes loaded from the ABC's class list.
    classes: HashMap<u32, GcCell<'gc, Class<'gc>>>,

    /// All methods loaded from the ABC's method list.
    methods: HashMap<u32, Method<'gc>>,

    /// All scripts loaded from the ABC's scripts list.
    scripts: HashMap<u32, GcCell<'gc, Script<'gc>>>,
}

impl<'gc> TranslationUnit<'gc> {
    pub fn from_abc(abc: Rc<AbcFile>, mc: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(
            mc,
            TranslationUnitData {
                abc: CollectWrapper(abc),
                classes: HashMap::new(),
                methods: HashMap::new(),
                scripts: HashMap::new(),
            },
        ))
    }

    /// Retrieve the underlying `AbcFile` for this translation unit.
    pub fn abc(self) -> Rc<AbcFile> {
        self.0.read().abc.0.clone()
    }

    /// Load a method from the ABC file and return it's method definition.
    pub fn load_method(
        self,
        method_index: u32,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Method<'gc>, Error> {
        let write = self.0.write(mc);
        if let Some(method) = write.methods.get(&method_index) {
            return Ok(method.clone());
        }

        drop(write);

        let method: Result<Avm2MethodEntry<'gc>, Error> =
            Avm2MethodEntry::from_method_index(self, Index::new(method_index))
                .ok_or_else(|| "Method index does not exist".into());
        let method: Method<'gc> = method?.into();

        self.0
            .write(mc)
            .methods
            .insert(method_index, method.clone());

        Ok(method)
    }

    /// Load a class from the ABC file and return it's class definition.
    pub fn load_class(
        self,
        class_index: u32,
        mc: MutationContext<'gc, '_>,
    ) -> Result<GcCell<'gc, Class<'gc>>, Error> {
        let write = self.0.write(mc);
        if let Some(class) = write.classes.get(&class_index) {
            return Ok(*class);
        }

        drop(write);

        let class = Class::from_abc_index(self, class_index, mc)?;
        self.0.write(mc).classes.insert(class_index, class);

        class.write(mc).load_traits(self, class_index, mc)?;

        Ok(class)
    }

    /// Load a script from the ABC file and return it's script definition.
    pub fn load_script(
        self,
        script_index: u32,
        mc: MutationContext<'gc, '_>,
    ) -> Result<GcCell<'gc, Script<'gc>>, Error> {
        let write = self.0.write(mc);
        if let Some(scripts) = write.scripts.get(&script_index) {
            return Ok(*scripts);
        }

        drop(write);

        let script = Script::from_abc_index(self, script_index, mc)?;
        self.0.write(mc).scripts.insert(script_index, script);

        script.write(mc).load_traits(self, script_index, mc)?;

        Ok(script)
    }
}

/// A loaded Script from an ABC file.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Script<'gc> {
    /// The initializer method to run for the script.
    init: Method<'gc>,

    /// Traits that this script uses.
    traits: Vec<Trait<'gc>>,

    /// Whether or not we loaded our traits.
    traits_loaded: bool,
}

impl<'gc> Script<'gc> {
    /// Construct a script from a `TranslationUnit` and it's script index.
    ///
    /// The returned script will be allocated, but no traits will be loaded.
    /// The caller is responsible for storing the class in the
    /// `TranslationUnit` and calling `load_traits` to complete the
    /// trait-loading process.
    pub fn from_abc_index(
        unit: TranslationUnit<'gc>,
        script_index: u32,
        mc: MutationContext<'gc, '_>,
    ) -> Result<GcCell<'gc, Self>, Error> {
        let abc = unit.abc();
        let script: Result<&AbcScript, Error> = abc
            .scripts
            .get(script_index as usize)
            .ok_or_else(|| "LoadError: Script index not valid".into());
        let script = script?;

        let init = unit.load_method(script.init_method.0, mc)?;

        Ok(GcCell::allocate(
            mc,
            Self {
                init,
                traits: Vec::new(),
                traits_loaded: false,
            },
        ))
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
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if self.traits_loaded {
            return Ok(());
        }

        self.traits_loaded = true;

        let abc = unit.abc();
        let script: Result<&AbcScript, Error> = abc
            .scripts
            .get(script_index as usize)
            .ok_or_else(|| "LoadError: Script index not valid".into());
        let script = script?;

        for abc_trait in script.traits.iter() {
            self.traits
                .push(Trait::from_abc_trait(unit, &abc_trait, mc)?);
        }

        Ok(())
    }

    /// Return the entrypoint for the script.
    pub fn init(&self) -> Method<'gc> {
        self.init.clone()
    }

    /// Return traits for this script.
    ///
    /// This function will return an error if it is incorrectly called before
    /// traits are loaded.
    pub fn traits(&self) -> Result<&[Trait<'gc>], Error> {
        if !self.traits_loaded {
            return Err("LoadError: Script traits accessed before they were loaded!".into());
        }

        Ok(&self.traits)
    }
}
