//! Whole script representation

use crate::avm2::class::Class;
use crate::avm2::method::{BytecodeMethod, Method};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::{Avm2, Error};
use crate::collect::CollectWrapper;
use fnv::FnvHashMap;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
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
    /// The ABC file that all of the following loaded data comes from.
    abc: CollectWrapper<Rc<AbcFile>>,

    /// All classes loaded from the ABC's class list.
    classes: FnvHashMap<u32, GcCell<'gc, Class<'gc>>>,

    /// All methods loaded from the ABC's method list.
    methods: FnvHashMap<u32, Method<'gc>>,

    /// All scripts loaded from the ABC's scripts list.
    scripts: FnvHashMap<u32, GcCell<'gc, Script<'gc>>>,

    /// All strings loaded from the ABC's strings list.
    strings: FnvHashMap<u32, AvmString<'gc>>,
}

impl<'gc> TranslationUnit<'gc> {
    pub fn from_abc(abc: Rc<AbcFile>, mc: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(
            mc,
            TranslationUnitData {
                abc: CollectWrapper(abc),
                classes: FnvHashMap::default(),
                methods: FnvHashMap::default(),
                scripts: FnvHashMap::default(),
                strings: FnvHashMap::default(),
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
        let read = self.0.read();
        if let Some(method) = read.methods.get(&method_index) {
            return Ok(method.clone());
        }

        drop(read);

        let method: Result<Gc<'gc, BytecodeMethod<'gc>>, Error> =
            BytecodeMethod::from_method_index(self, Index::new(method_index), mc)
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
        avm2: &mut Avm2<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<GcCell<'gc, Class<'gc>>, Error> {
        let read = self.0.read();
        if let Some(class) = read.classes.get(&class_index) {
            return Ok(*class);
        }

        drop(read);

        let class = Class::from_abc_index(self, class_index, mc)?;
        self.0.write(mc).classes.insert(class_index, class);

        class.write(mc).load_traits(self, class_index, avm2, mc)?;

        Ok(class)
    }

    /// Load a script from the ABC file and return it's script definition.
    pub fn load_script(
        self,
        script_index: u32,
        avm2: &mut Avm2<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<GcCell<'gc, Script<'gc>>, Error> {
        let read = self.0.read();
        if let Some(scripts) = read.scripts.get(&script_index) {
            return Ok(*scripts);
        }

        drop(read);

        let script = Script::from_abc_index(self, script_index, mc)?;
        self.0.write(mc).scripts.insert(script_index, script);

        script.write(mc).load_traits(self, script_index, avm2, mc)?;

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
                .0
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
        avm2: &mut Avm2<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if self.traits_loaded {
            return Ok(());
        }

        self.traits_loaded = true;

        let abc = unit.abc();
        let script: Result<_, Error> = abc
            .scripts
            .get(script_index as usize)
            .ok_or_else(|| "LoadError: Script index not valid".into());
        let script = script?;

        for abc_trait in script.traits.iter() {
            self.traits
                .push(Trait::from_abc_trait(unit, &abc_trait, avm2, mc)?);
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
