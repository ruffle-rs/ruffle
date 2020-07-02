//! Whole script representation

use crate::avm2::activation::Avm2ScriptEntry;
use crate::avm2::class::Class;
use crate::avm2::function::{Avm2MethodEntry, Method};
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashMap;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Index};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct CollectWrapper<T>(T);

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
pub struct TranslationUnit<'gc> {
    /// The ABC file that all of the following loaded data comes from.
    abc: CollectWrapper<Rc<AbcFile>>,

    /// All classes loaded from the ABC's class list.
    classes: HashMap<u32, GcCell<'gc, Class<'gc>>>,

    /// All methods loaded from the ABC's method list.
    methods: HashMap<u32, Method<'gc>>,

    /// All scripts loaded from the ABC's scripts list.
    scripts: HashMap<u32, GcCell<'gc, Avm2ScriptEntry>>,
}

impl<'gc> TranslationUnit<'gc> {
    pub fn from_abc(abc: Rc<AbcFile>) -> Self {
        Self {
            abc: CollectWrapper(abc),
            classes: HashMap::new(),
            methods: HashMap::new(),
            scripts: HashMap::new(),
        }
    }

    /// Retrieve the underlying `AbcFile` for this translation unit.
    pub fn abc(&self) -> Rc<AbcFile> {
        self.abc.0
    }

    /// Load a method from the ABC file and return it's method definition.
    pub fn load_method(&mut self, method_index: u32) -> Result<Method<'gc>, Error> {
        if let Some(method) = self.methods.get(&method_index) {
            return Ok(method.clone());
        }

        let method: Result<Avm2MethodEntry, Error> =
            Avm2MethodEntry::from_method_index(self.abc.0, Index::new(method_index))
                .ok_or_else(|| "Method index does not exist".into());
        let method = method?.into();

        self.methods.insert(method_index, method);

        return Ok(method);
    }

    /// Load a class from the ABC file and return it's class definition.
    pub fn load_class(
        &mut self,
        class_index: u32,
        mc: MutationContext<'gc, '_>,
    ) -> Result<GcCell<'gc, Class<'gc>>, Error> {
        if let Some(class) = self.classes.get(&class_index) {
            return Ok(class.clone());
        }

        let class = Class::from_abc_index(&mut self, class_index, mc)?;
        self.classes.insert(class_index, class);

        class.write(mc).load_traits(&mut self, class_index, mc)?;

        return Ok(class);
    }
}
