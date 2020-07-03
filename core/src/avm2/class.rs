//! AVM2 classes

use crate::avm2::method::Method;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::r#trait::{Trait, TraitKind};
use crate::avm2::script::TranslationUnit;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use swf::avm2::types::{Class as AbcClass, Instance as AbcInstance};

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
    instance_traits: Vec<Trait<'gc>>,

    /// The class initializer for this class.
    ///
    /// Must be called once prior to any use of this class.
    class_init: Method<'gc>,

    /// Static traits for a given class.
    ///
    /// These are accessed as constructor properties.
    class_traits: Vec<Trait<'gc>>,

    /// Whether or not this `Class` has loaded it's traits or not.
    traits_loaded: bool,
}

/// Find traits in a list of traits matching a name.
///
/// This function also enforces final/override bits on the traits, and will
/// raise `VerifyError`s as needed.
///
/// TODO: This is an O(n^2) algorithm, it sucks.
fn do_trait_lookup<'gc>(
    name: &QName,
    known_traits: &mut Vec<Trait<'gc>>,
    all_traits: &[Trait<'gc>],
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
    /// Construct a class from a `TranslationUnit` and it's class index.
    ///
    /// The returned class will be allocated, but no traits will be loaded. The
    /// caller is responsible for storing the class in the `TranslationUnit`
    /// and calling `load_traits` to complete the trait-loading process.
    pub fn from_abc_index(
        unit: TranslationUnit<'gc>,
        class_index: u32,
        mc: MutationContext<'gc, '_>,
    ) -> Result<GcCell<'gc, Self>, Error> {
        let abc = unit.abc();
        let abc_class: Result<&AbcClass, Error> = abc
            .classes
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Class index not valid".into());
        let abc_class = abc_class?;

        let abc_instance: Result<&AbcInstance, Error> = abc
            .instances
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Instance index not valid".into());
        let abc_instance = abc_instance?;

        let name = QName::from_abc_multiname(&unit.abc(), abc_instance.name.clone())?;
        let super_class = if abc_instance.super_name.0 == 0 {
            None
        } else {
            Some(Multiname::from_abc_multiname_static(
                &unit.abc(),
                abc_instance.super_name.clone(),
            )?)
        };

        let protected_namespace = if let Some(ns) = &abc_instance.protected_namespace {
            Some(Namespace::from_abc_namespace(&unit.abc(), ns.clone())?)
        } else {
            None
        };

        let mut interfaces = Vec::new();
        for interface_name in abc_instance.interfaces.iter() {
            interfaces.push(Multiname::from_abc_multiname_static(
                &unit.abc(),
                interface_name.clone(),
            )?);
        }

        let instance_init = unit.load_method(abc_instance.init_method.0, mc)?;
        let class_init = unit.load_method(abc_class.init_method.0, mc)?;

        Ok(GcCell::allocate(
            mc,
            Self {
                name,
                super_class,
                is_sealed: abc_instance.is_sealed,
                is_final: abc_instance.is_final,
                is_interface: abc_instance.is_interface,
                protected_namespace,
                interfaces,
                instance_init,
                instance_traits: Vec::new(),
                class_init,
                class_traits: Vec::new(),
                traits_loaded: false,
            },
        ))
    }

    /// Finish the class-loading process by loading traits.
    ///
    /// This process must be done after the `Class` has been stored in the
    /// `TranslationUnit`. Failing to do so runs the risk of runaway recursion
    /// or double-borrows. It should be done before the class is actually
    /// instantiated into an `Object`.
    pub fn load_traits(
        &mut self,
        unit: TranslationUnit<'gc>,
        class_index: u32,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if self.traits_loaded {
            return Ok(());
        }

        self.traits_loaded = true;

        let abc = unit.abc();
        let abc_class: Result<&AbcClass, Error> = abc
            .classes
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Class index not valid".into());
        let abc_class = abc_class?;

        let abc_instance: Result<&AbcInstance, Error> = abc
            .instances
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Instance index not valid".into());
        let abc_instance = abc_instance?;

        for abc_trait in abc_instance.traits.iter() {
            self.instance_traits
                .push(Trait::from_abc_trait(unit, &abc_trait, mc)?);
        }

        for abc_trait in abc_class.traits.iter() {
            self.class_traits
                .push(Trait::from_abc_trait(unit, &abc_trait, mc)?);
        }

        Ok(())
    }

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
        known_traits: &mut Vec<Trait<'gc>>,
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
        known_traits: &mut Vec<Trait<'gc>>,
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
        self.instance_init.clone()
    }

    /// Get this class's class initializer.
    pub fn class_init(&self) -> Method<'gc> {
        self.class_init.clone()
    }
}
