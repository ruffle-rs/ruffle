//! AVM2 classes

use crate::avm2::method::Method;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::script::TranslationUnit;
use crate::avm2::string::AvmString;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::{Avm2, Error};
use crate::collect::CollectWrapper;
use bitflags::bitflags;
use gc_arena::{Collect, GcCell, MutationContext};
use swf::avm2::types::{
    Class as AbcClass, Instance as AbcInstance, Method as AbcMethod, MethodBody as AbcMethodBody,
};

bitflags! {
    /// All possible attributes for a given class.
    pub struct ClassAttributes: u8 {
        /// Class is sealed, attempts to set or init dynamic properties on an
        /// object will generate a runtime error.
        const SEALED    = 1 << 0;

        /// Class is final, attempts to construct child classes from it will
        /// generate a verification error.
        const FINAL     = 1 << 1;

        /// Class is an interface.
        const INTERFACE = 1 << 2;
    }
}

/// A loaded ABC Class which can be used to construct objects with.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Class<'gc> {
    /// The name of the class.
    name: QName<'gc>,

    /// The name of this class's superclass.
    super_class: Option<Multiname<'gc>>,

    /// Attributes of the given class.
    attributes: CollectWrapper<ClassAttributes>,

    /// The namespace that protected traits of this class are stored into.
    protected_namespace: Option<Namespace<'gc>>,

    /// The list of interfaces this class implements.
    interfaces: Vec<Multiname<'gc>>,

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

    /// Whether or not this `Class` has loaded its traits or not.
    traits_loaded: bool,
}

/// Find traits in a list of traits matching a name.
///
/// This function also enforces final/override bits on the traits, and will
/// raise `VerifyError`s as needed.
///
/// TODO: This is an O(n^2) algorithm, it sucks.
fn do_trait_lookup<'gc>(
    name: &QName<'gc>,
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

/// Find traits in a list of traits matching a slot ID.
///
/// This function also enforces final/override bits on the traits, and will
/// raise `VerifyError`s as needed.
///
/// TODO: This is an O(n^2) algorithm, it sucks.
fn do_trait_lookup_by_slot<'gc>(
    id: u32,
    known_traits: &mut Vec<Trait<'gc>>,
    all_traits: &[Trait<'gc>],
) -> Result<(), Error> {
    for trait_entry in all_traits {
        let trait_id = match trait_entry.kind() {
            TraitKind::Slot { slot_id, .. } => slot_id,
            TraitKind::Const { slot_id, .. } => slot_id,
            TraitKind::Class { slot_id, .. } => slot_id,
            TraitKind::Function { slot_id, .. } => slot_id,
            _ => continue,
        };

        if id == *trait_id {
            for known_trait in known_traits.iter() {
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
    /// Create a new class.
    ///
    /// This function is primarily intended for use by native code to define
    /// builtin classes. The absolute minimum necessary to define a class is
    /// required here; further methods allow further changes to the class.
    ///
    /// Classes created in this way cannot have traits loaded from an ABC file
    /// using `load_traits`.
    pub fn new(
        name: QName<'gc>,
        super_class: Option<Multiname<'gc>>,
        instance_init: Method<'gc>,
        class_init: Method<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> GcCell<'gc, Self> {
        GcCell::allocate(
            mc,
            Self {
                name,
                super_class,
                attributes: CollectWrapper(ClassAttributes::empty()),
                protected_namespace: None,
                interfaces: Vec::new(),
                instance_init,
                instance_traits: Vec::new(),
                class_init,
                class_traits: Vec::new(),
                traits_loaded: true,
            },
        )
    }

    /// Set the attributes of the class (sealed/final/interface status).
    pub fn set_attributes(&mut self, attributes: ClassAttributes) {
        self.attributes = CollectWrapper(attributes);
    }

    /// Add a protected namespace to this class.
    pub fn set_protected_namespace(&mut self, ns: Namespace<'gc>) {
        self.protected_namespace = Some(ns)
    }

    /// Construct a class from a `TranslationUnit` and its class index.
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

        let name = QName::from_abc_multiname(unit, abc_instance.name.clone(), mc)?;
        let super_class = if abc_instance.super_name.0 == 0 {
            None
        } else {
            Some(Multiname::from_abc_multiname_static(
                unit,
                abc_instance.super_name.clone(),
                mc,
            )?)
        };

        let protected_namespace = if let Some(ns) = &abc_instance.protected_namespace {
            Some(Namespace::from_abc_namespace(unit, ns.clone(), mc)?)
        } else {
            None
        };

        let mut interfaces = Vec::new();
        for interface_name in abc_instance.interfaces.iter() {
            interfaces.push(Multiname::from_abc_multiname_static(
                unit,
                interface_name.clone(),
                mc,
            )?);
        }

        let instance_init = unit.load_method(abc_instance.init_method.0, mc)?;
        let class_init = unit.load_method(abc_class.init_method.0, mc)?;

        let mut attributes = ClassAttributes::empty();
        attributes.set(ClassAttributes::SEALED, abc_instance.is_sealed);
        attributes.set(ClassAttributes::FINAL, abc_instance.is_final);
        attributes.set(ClassAttributes::INTERFACE, abc_instance.is_interface);

        Ok(GcCell::allocate(
            mc,
            Self {
                name,
                super_class,
                attributes: CollectWrapper(attributes),
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
        avm2: &mut Avm2<'gc>,
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
                .push(Trait::from_abc_trait(unit, &abc_trait, avm2, mc)?);
        }

        for abc_trait in abc_class.traits.iter() {
            self.class_traits
                .push(Trait::from_abc_trait(unit, &abc_trait, avm2, mc)?);
        }

        Ok(())
    }

    pub fn from_method_body(
        avm2: &mut Avm2<'gc>,
        mc: MutationContext<'gc, '_>,
        translation_unit: TranslationUnit<'gc>,
        method: &AbcMethod,
        body: &AbcMethodBody,
    ) -> Result<GcCell<'gc, Self>, Error> {
        let name = translation_unit.pool_string(method.name.as_u30(), mc)?;
        let mut traits = Vec::new();

        for trait_entry in body.traits.iter() {
            traits.push(Trait::from_abc_trait(
                translation_unit,
                &trait_entry,
                avm2,
                mc,
            )?);
        }

        Ok(GcCell::allocate(
            mc,
            Self {
                name: QName::dynamic_name(name),
                super_class: None,
                attributes: CollectWrapper(ClassAttributes::empty()),
                protected_namespace: None,
                interfaces: Vec::new(),
                instance_init: Method::from_builtin(|_, _, _| {
                    Err("Do not call activation initializers!".into())
                }),
                instance_traits: traits,
                class_init: Method::from_builtin(|_, _, _| {
                    Err("Do not call activation class initializers!".into())
                }),
                class_traits: Vec::new(),
                traits_loaded: true,
            },
        ))
    }

    pub fn name(&self) -> &QName<'gc> {
        &self.name
    }

    pub fn super_class_name(&self) -> &Option<Multiname<'gc>> {
        &self.super_class
    }

    /// Define a trait on the class.
    ///
    /// Class traits will be accessible as properties on the class constructor
    /// function.
    pub fn define_class_trait(&mut self, my_trait: Trait<'gc>) {
        self.class_traits.push(my_trait);
    }

    /// Given a name, append class traits matching the name to a list of known
    /// traits.
    ///
    /// This function adds its result onto the list of known traits, with the
    /// caveat that duplicate entries will be replaced (if allowed). As such, this
    /// function should be run on the class hierarchy from top to bottom.
    ///
    /// If a given trait has an invalid name, attempts to override a final trait,
    /// or overlaps an existing trait without being an override, then this function
    /// returns an error.
    pub fn lookup_class_traits(
        &self,
        name: &QName<'gc>,
        known_traits: &mut Vec<Trait<'gc>>,
    ) -> Result<(), Error> {
        do_trait_lookup(name, known_traits, &self.class_traits)
    }

    /// Given a slot ID, append class traits matching the slot to a list of
    /// known traits.
    ///
    /// This function adds its result onto the list of known traits, with the
    /// caveat that duplicate entries will be replaced (if allowed). As such, this
    /// function should be run on the class hierarchy from top to bottom.
    ///
    /// If a given trait has an invalid name, attempts to override a final trait,
    /// or overlaps an existing trait without being an override, then this function
    /// returns an error.
    pub fn lookup_class_traits_by_slot(
        &self,
        id: u32,
        known_traits: &mut Vec<Trait<'gc>>,
    ) -> Result<(), Error> {
        do_trait_lookup_by_slot(id, known_traits, &self.class_traits)
    }

    /// Determines if this class provides a given trait on itself.
    pub fn has_class_trait(&self, name: &QName<'gc>) -> bool {
        for trait_entry in self.class_traits.iter() {
            if name == trait_entry.name() {
                return true;
            }
        }

        false
    }

    /// Look for a class trait with a given local name, and return its
    /// namespace.
    ///
    /// TODO: Matching multiple namespaces with the same local name is at least
    /// claimed by the AVM2 specification to be a `VerifyError`.
    pub fn resolve_any_class_trait(&self, local_name: AvmString<'gc>) -> Option<Namespace<'gc>> {
        for trait_entry in self.class_traits.iter() {
            if local_name == trait_entry.name().local_name() {
                return Some(trait_entry.name().namespace().clone());
            }
        }

        None
    }

    /// Define a trait on instances of the class.
    ///
    /// Instance traits will be accessible as properties on instances of the
    /// class. They will not be accessible on the class prototype, and any
    /// properties defined on the prototype will be shadowed by these traits.
    pub fn define_instance_trait(&mut self, my_trait: Trait<'gc>) {
        self.instance_traits.push(my_trait);
    }

    /// Given a name, append instance traits matching the name to a list of
    /// known traits.
    ///
    /// This function adds its result onto the list of known traits, with the
    /// caveat that duplicate entries will be replaced (if allowed). As such, this
    /// function should be run on the class hierarchy from top to bottom.
    ///
    /// If a given trait has an invalid name, attempts to override a final trait,
    /// or overlaps an existing trait without being an override, then this function
    /// returns an error.
    pub fn lookup_instance_traits(
        &self,
        name: &QName<'gc>,
        known_traits: &mut Vec<Trait<'gc>>,
    ) -> Result<(), Error> {
        do_trait_lookup(name, known_traits, &self.instance_traits)
    }

    /// Given a slot ID, append instance traits matching the slot to a list of
    /// known traits.
    ///
    /// This function adds its result onto the list of known traits, with the
    /// caveat that duplicate entries will be replaced (if allowed). As such, this
    /// function should be run on the class hierarchy from top to bottom.
    ///
    /// If a given trait has an invalid name, attempts to override a final trait,
    /// or overlaps an existing trait without being an override, then this function
    /// returns an error.
    pub fn lookup_instance_traits_by_slot(
        &self,
        id: u32,
        known_traits: &mut Vec<Trait<'gc>>,
    ) -> Result<(), Error> {
        do_trait_lookup_by_slot(id, known_traits, &self.instance_traits)
    }

    /// Determines if this class provides a given trait on its instances.
    pub fn has_instance_trait(&self, name: &QName<'gc>) -> bool {
        for trait_entry in self.instance_traits.iter() {
            if name == trait_entry.name() {
                return true;
            }
        }

        false
    }

    /// Look for an instance trait with a given local name, and return its
    /// namespace.
    ///
    /// TODO: Matching multiple namespaces with the same local name is at least
    /// claimed by the AVM2 specification to be a `VerifyError`.
    pub fn resolve_any_instance_trait(&self, local_name: AvmString<'gc>) -> Option<Namespace<'gc>> {
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

    pub fn interfaces(&self) -> &[Multiname<'gc>] {
        &self.interfaces
    }

    pub fn implements(&mut self, iface: Multiname<'gc>) {
        self.interfaces.push(iface)
    }

    /// Determine if this class is sealed (no dynamic properties)
    pub fn is_sealed(&self) -> bool {
        self.attributes.0.contains(ClassAttributes::SEALED)
    }
}
