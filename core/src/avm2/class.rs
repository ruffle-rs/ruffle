//! AVM2 classes

use crate::avm2::function::Method;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::r#trait::{Trait, TraitKind};
use crate::avm2::Error;
use gc_arena::{Collect, Gc};
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
    instance_init: Method<'gc>,

    /// Instance traits for a given class.
    ///
    /// These are accessed as normal instance properties; they should not be
    /// present on prototypes, but instead should shadow any prototype
    /// properties that would match.
    instance_traits: Vec<Gc<'gc, Trait<'gc>>>,

    /// The class initializer for this class.
    ///
    /// Must be called once prior to any use of this class.
    class_init: Method<'gc>,

    /// Static traits for a given class.
    ///
    /// These are accessed as constructor properties.
    class_traits: Vec<Gc<'gc, Trait<'gc>>>,
}

/// Find traits in a list of traits matching a name.
///
/// This function also enforces final/override bits on the traits, and will
/// raise `VerifyError`s as needed.
///
/// TODO: This is an O(n^2) algorithm, it sucks.
fn do_trait_lookup<'gc>(
    name: &QName,
    known_traits: &mut Vec<Gc<'gc, Trait<'gc>>>,
    all_traits: &[Gc<'gc, Trait<'gc>>],
) -> Result<(), Error> {
    for trait_entry in all_traits {
        if name == trait_entry.name() {
            for known_trait in known_traits.iter() {
                match (&trait_entry.kind(), &known_trait.kind()) {
                    (TraitKind::Getter { .. }, TraitKind::Setter { .. }) => continue,
                    (TraitKind::Setter { .. }, TraitKind::Getter { .. }) => continue,
                    _ => {}
                };

                if known_trait.is_final() {
                    return Err("Attempting to override a final definition".into());
                }

                if !trait_entry.is_override() {
                    return Err("Definition override is not marked as override".into());
                }
            }

            known_traits.push(trait_entry.clone());
        }
    }

    Ok(())
}

impl<'gc> Class<'gc> {
    pub fn name(&self) -> &QName {
        &self.name
    }

    pub fn super_class_name(&self) -> &Option<Multiname> {
        &self.super_class
    }

    /// Given a name, append class traits matching the name to a list of known
    /// traits.
    ///
    /// This function adds it's result onto the list of known traits, with the
    /// caveat that duplicate entries will be replaced (if allowed). As such, this
    /// function should be run on the class hierarchy from top to bottom.
    ///
    /// If a given trait has an invalid name, attempts to override a final trait,
    /// or overlaps an existing trait without being an override, then this function
    /// returns an error.
    pub fn lookup_class_traits(
        &self,
        name: &QName,
        known_traits: &mut Vec<Gc<'gc, Trait<'gc>>>,
    ) -> Result<(), Error> {
        do_trait_lookup(name, known_traits, &self.class_traits)
    }

    /// Determines if this class provides a given trait on itself.
    pub fn has_class_trait(&self, name: &QName) -> bool {
        for trait_entry in self.class_traits.iter() {
            if name == trait_entry.name() {
                return true;
            }
        }

        false
    }

    /// Look for a class trait with a given local name, and return it's
    /// namespace.
    ///
    /// TODO: Matching multiple namespaces with the same local name is at least
    /// claimed by the AVM2 specification to be a `VerifyError`.
    pub fn resolve_any_class_trait(&self, local_name: &str) -> Option<Namespace> {
        for trait_entry in self.class_traits.iter() {
            if local_name == trait_entry.name().local_name() {
                return Some(trait_entry.name().namespace().clone());
            }
        }

        None
    }

    /// Given a name, append instance traits matching the name to a list of
    /// known traits.
    ///
    /// This function adds it's result onto the list of known traits, with the
    /// caveat that duplicate entries will be replaced (if allowed). As such, this
    /// function should be run on the class hierarchy from top to bottom.
    ///
    /// If a given trait has an invalid name, attempts to override a final trait,
    /// or overlaps an existing trait without being an override, then this function
    /// returns an error.
    pub fn lookup_instance_traits(
        &self,
        name: &QName,
        known_traits: &mut Vec<Gc<'gc, Trait<'gc>>>,
    ) -> Result<(), Error> {
        do_trait_lookup(name, known_traits, &self.instance_traits)
    }

    /// Determines if this class provides a given trait on it's instances.
    pub fn has_instance_trait(&self, name: &QName) -> bool {
        for trait_entry in self.instance_traits.iter() {
            if name == trait_entry.name() {
                return true;
            }
        }

        false
    }

    /// Look for an instance trait with a given local name, and return it's
    /// namespace.
    ///
    /// TODO: Matching multiple namespaces with the same local name is at least
    /// claimed by the AVM2 specification to be a `VerifyError`.
    pub fn resolve_any_instance_trait(&self, local_name: &str) -> Option<Namespace> {
        for trait_entry in self.instance_traits.iter() {
            if local_name == trait_entry.name().local_name() {
                return Some(trait_entry.name().namespace().clone());
            }
        }

        None
    }

    /// Get this class's instance initializer.
    pub fn instance_init(&self) -> Method<'gc> {
        self.instance_init
    }

    /// Get this class's class initializer.
    pub fn class_init(&self) -> Method<'gc> {
        self.class_init
    }
}
