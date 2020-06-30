//! AVM2 classes

use crate::avm2::function::Executable;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::r#trait::Trait;
use gc_arena::Collect;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Class as AbcClass, Index, Instance as AbcInstance};

/// Represents a reference to an AVM2 class.
///
/// For some reason, this comes in two parts, one for static properties (called
/// the "class") and one for dynamic properties (called the "instance", even
/// though it really defines what ES3/AS2 would call a prototype)
#[derive(Collect, Clone, Debug)]
#[collect(require_static)]
pub struct Avm2ClassEntry {
    /// The ABC file this function was defined in.
    pub abc: Rc<AbcFile>,

    /// The ABC class (used to define static properties).
    ///
    /// This is also the index of the ABC instance, which holds instance
    /// properties.
    pub abc_class: u32,
}

impl Avm2ClassEntry {
    /// Construct an `Avm2MethodEntry` from an `AbcFile` and method index.
    ///
    /// This function returns `None` if the given class index does not resolve
    /// to a valid ABC class, or a valid ABC instance. As mentioned in the type
    /// documentation, ABC classes and instances are intended to be paired.
    pub fn from_class_index(abc: Rc<AbcFile>, abc_class: Index<AbcClass>) -> Option<Self> {
        if abc.classes.get(abc_class.0 as usize).is_some()
            && abc.instances.get(abc_class.0 as usize).is_some()
        {
            return Some(Self {
                abc,
                abc_class: abc_class.0,
            });
        }

        None
    }

    /// Get the underlying ABC file.
    pub fn abc(&self) -> Rc<AbcFile> {
        self.abc.clone()
    }

    /// Get a reference to the ABC class entry this refers to.
    pub fn class(&self) -> &AbcClass {
        self.abc.classes.get(self.abc_class as usize).unwrap()
    }

    /// Get a reference to the ABC class instance entry this refers to.
    pub fn instance(&self) -> &AbcInstance {
        self.abc.instances.get(self.abc_class as usize).unwrap()
    }
}

/// A loaded ABC Class which can be used to construct objects with.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Class<'gc> {
    /// The name of the class.
    name: QName,

    /// The name of this class's superclass.
    super_class: Option<Multiname>,

    /// If this class is sealed (dynamic property writes should fail)
    is_sealed: bool,

    /// If this class is final (subclassing should fail)
    is_final: bool,

    /// If this class is an interface
    is_interface: bool,

    /// The namespace that protected traits of this class are stored into.
    protected_namespace: Option<Namespace>,

    /// The list of interfaces this class implements.
    interfaces: Vec<Multiname>,

    /// The instance initializer for this class.
    ///
    /// Must be called each time a new class instance is constructed.
    instance_initializer: Executable<'gc>,

    /// Instance traits for a given class.
    ///
    /// These are accessed as normal instance properties; they should not be
    /// present on prototypes, but instead should shadow any prototype
    /// properties that would match.
    instance_traits: Vec<Trait<'gc>>,

    /// The class initializer for this class.
    ///
    /// Must be called once prior to any use of this class.
    class_init: Executable<'gc>,

    /// Static traits for a given class.
    ///
    /// These are accessed as constructor properties.
    class_traits: Vec<Trait<'gc>>,
}
