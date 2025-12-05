//! AVM2 classes

use crate::avm2::activation::Activation;
use crate::avm2::error::{
    make_error_1014, make_error_1053, make_error_1059, make_error_1103, make_error_1107,
    make_error_1110, make_error_1111, Error1014Type,
};
use crate::avm2::method::{Method, MethodAssociation, NativeMethodImpl};
use crate::avm2::object::{scriptobject_allocator, ClassObject, Object};
use crate::avm2::script::TranslationUnit;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::value::Value;
use crate::avm2::vtable::VTable;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::context::UpdateContext;
use crate::string::{AvmString, WString};
use bitflags::bitflags;
use fnv::FnvHashMap;
use gc_arena::barrier::unlock;
use gc_arena::lock::{OnceLock, RefLock};
use gc_arena::{Collect, Gc, Lock, Mutation};
use swf::avm2::types::Trait as AbcTrait;

use std::cell::{Cell, Ref};
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

use swf::avm2::types::{Method as AbcMethod, MethodBody as AbcMethodBody};

bitflags! {
    /// All possible attributes for a given class.
    #[derive(Clone, Copy)]
    pub struct ClassAttributes: u8 {
        /// Class is sealed, attempts to set or init dynamic properties on an
        /// object will generate a runtime error.
        const SEALED    = 1 << 0;

        /// Class is final, attempts to construct child classes from it will
        /// generate a verification error.
        const FINAL     = 1 << 1;

        /// Class is an interface.
        const INTERFACE = 1 << 2;

        /// Class accepts type parameters.
        const GENERIC = 1 << 3;
    }
}

/// A function that can be used to allocate instances of a class.
///
/// By default, the `implicit_allocator` is used, which attempts to use the base
/// class's allocator, and defaults to `ScriptObject` otherwise. Custom
/// allocators anywhere in the class inheritance chain can change the
/// representation of all subclasses that use the implicit allocator.
///
/// Parameters for the allocator are:
///
///  * `class` - The class object that is being allocated. This must be the
///    current class (using a superclass will cause the wrong class to be
///    read for traits).
///  * `activation` - The current AVM2 activation.
pub type AllocatorFn =
    for<'gc> fn(ClassObject<'gc>, &mut Activation<'_, 'gc>) -> Result<Object<'gc>, Error<'gc>>;

#[derive(Clone, Copy)]
pub struct Allocator(pub AllocatorFn);

/// A function that can be used to both allocate and construct an instance of a class.
///
/// This function should be passed an Activation, and the arguments passed to the
/// constructor, and will return an Object.
pub type CustomConstructorFn =
    for<'gc> fn(&mut Activation<'_, 'gc>, &[Value<'gc>]) -> Result<Value<'gc>, Error<'gc>>;

#[derive(Clone, Copy)]
pub struct CustomConstructor(pub CustomConstructorFn);

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Allocator")
            .field(&"<native code>".to_string())
            .finish()
    }
}

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
enum ClassLink<'gc> {
    Unlinked,
    LinkToInstance(Class<'gc>),
    LinkToClass(Class<'gc>),
}

#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct Class<'gc>(Gc<'gc, ClassData<'gc>>);

/// A loaded ABC Class which can be used to construct objects with.
#[derive(Collect)]
#[collect(no_drop)]
pub struct ClassData<'gc> {
    /// The name of the class.
    name: Lock<QName<'gc>>,

    /// The type parameter for this class (only supported for Vector)
    param: Lock<Option<Option<Class<'gc>>>>,

    /// This class's superclass, or None if it has no superclass
    super_class: Option<Class<'gc>>,

    /// Attributes of the given class.
    attributes: Cell<ClassAttributes>,

    /// The namespace that protected traits of this class are stored into.
    protected_namespace: Option<Namespace<'gc>>,

    /// The list of interfaces this class directly implements. This does not include any
    /// superinterfaces, nor interfaces implemented by the superclass.
    direct_interfaces: Box<[Class<'gc>]>,

    /// Interfaces implemented by this class, including interfaces
    /// from parent classes and superinterfaces (recursively).
    /// TODO - avoid cloning this when a subclass implements the
    /// same interface as its superclass.
    all_interfaces: OnceLock<Box<[Class<'gc>]>>,

    /// The instance allocator for this class. Defaults to the script object allocator
    /// if no allocator is provided.
    #[collect(require_static)]
    instance_allocator: Allocator,

    /// The instance initializer for this class, None if this class is not
    /// instantiable.
    ///
    /// Must be called each time a new class instance is constructed.
    instance_init: Option<Method<'gc>>,

    /// Traits (and vtable) for a given class.
    ///
    /// These are accessed as normal instance properties; they should not be
    /// present on prototypes, but instead should shadow any prototype
    /// properties that would match.
    traits: OnceLock<Box<[Trait<'gc>]>>,

    /// The class' vtable.
    vtable: OnceLock<VTable<'gc>>,

    /// The customization point for `Class(args...)` without `new`
    /// If None, a simple coercion is done.
    #[collect(require_static)]
    call_handler: Option<NativeMethodImpl>,

    /// The custom constructor for this class, if it exists.
    ///
    /// This function will both allocate and initialize the class.
    #[collect(require_static)]
    custom_constructor: Option<CustomConstructor>,

    /// The Class this Class is linked to. If this class represents instance info,
    /// this will be a ClassLink::LinkToClass. If this class represents class info,
    /// this will be a ClassLink::LinkToInstance. This must be one of the two,
    /// unless this class has not yet been fully initialized, in which case it will
    /// be set to ClassLink::Unlinked.
    linked_class: Lock<ClassLink<'gc>>,

    /// The special builtin class that this class represents. For example, this
    /// is only set to `BuiltinType::Uint` for the builtin `uint` class.
    /// This allows identifying builtin classes without having to compare them
    /// against the classes stored in SystemClassDefs.
    #[collect(require_static)]
    builtin_type: Cell<Option<BuiltinType>>,

    cell: RefLock<ClassDataMut<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
struct ClassDataMut<'gc> {
    /// Maps a type parameter to the application of this class with that parameter.
    ///
    /// Only applicable if this class is generic.
    applications: FnvHashMap<Option<Class<'gc>>, Class<'gc>>,

    /// The ClassObjects for this class.
    /// In almost all cases, this will either be empty or have a single object.
    /// However, a swf can run `newclass` multiple times on the same class
    /// to create multiple `ClassObjects`.
    class_objects: Vec<ClassObject<'gc>>,
}

impl PartialEq for Class<'_> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(self.0, other.0)
    }
}

impl Eq for Class<'_> {}

impl Hash for Class<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Gc::as_ptr(self.0).hash(state);
    }
}

impl core::fmt::Debug for Class<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Class").field("name", &self.name()).finish()
    }
}

impl<'gc> ClassData<'gc> {
    /// Create an unlinked & unloaded class.
    fn empty(name: QName<'gc>) -> Self {
        Self {
            name: Lock::new(name),
            param: Lock::new(None),
            super_class: None,
            attributes: Cell::new(ClassAttributes::empty()),
            protected_namespace: None,
            direct_interfaces: Box::default(),
            all_interfaces: OnceLock::new(),
            instance_allocator: Allocator(scriptobject_allocator),
            instance_init: None,
            traits: OnceLock::new(),
            vtable: OnceLock::new(),
            call_handler: None,
            custom_constructor: None,
            linked_class: Lock::new(ClassLink::Unlinked),
            builtin_type: Cell::new(None),
            cell: RefLock::new(ClassDataMut {
                applications: FnvHashMap::default(),
                class_objects: Vec::new(),
            }),
        }
    }
}

impl<'gc> Class<'gc> {
    pub fn as_ptr(self) -> *const () {
        Gc::as_ptr(self.0).cast()
    }

    /// Create an unlinked class from its name, superclass, and traits.
    pub fn custom_new(
        name: QName<'gc>,
        super_class: Option<Class<'gc>>,
        instance_init: Option<Method<'gc>>,
        traits: Box<[Trait<'gc>]>,
        mc: &Mutation<'gc>,
    ) -> Self {
        let mut class = ClassData::empty(name);
        class.super_class = super_class;
        class.instance_init = instance_init;
        class.traits = OnceLock::from(traits);
        Class(Gc::new(mc, class))
    }

    pub fn add_application(self, mc: &Mutation<'gc>, param: Option<Class<'gc>>, cls: Class<'gc>) {
        let write = unlock!(Gc::write(mc, self.0), ClassData, cell);
        write.borrow_mut().applications.insert(param, cls);
    }

    /// Apply type parameters to an existing class.
    ///
    /// This is used to parameterize a generic type. The returned class will no
    /// longer be generic.
    pub fn with_type_param(
        context: &mut UpdateContext<'gc>,
        this: Class<'gc>,
        param: Option<Class<'gc>>,
    ) -> Class<'gc> {
        let mc = context.gc();

        let read = Gc::as_ref(this.0).cell.borrow();
        if let Some(application) = read.applications.get(&param) {
            return *application;
        }

        // This can only happen for non-builtin Vector types,
        // so let's create one here directly.

        let object_vector_i_class = *read
            .applications
            .get(&None)
            .expect("Vector.<*> not initialized?");
        drop(read);

        let object_vector_c_class = object_vector_i_class
            .c_class()
            .expect("T$ cannot be generic");

        let param = param.expect("Trying to create Vector<*>, which shouldn't happen here");

        // FIXME - we should store a `Multiname` instead of a `QName`, and use the
        // `params` field. For now, this is good enough to get tests passing
        let name = format!("Vector.<{}>", param.name().to_qualified_name(mc));
        let name = QName::new(this.name().namespace(), AvmString::new_utf8(mc, name));

        let mut local_name_buf = WString::from(name.local_name().as_wstr());
        local_name_buf.push_char('$');
        let c_name = QName::new(name.namespace(), AvmString::new(mc, local_name_buf));

        let mut i_class = ClassData::empty(name);
        i_class.param = Lock::new(Some(Some(param)));
        i_class.super_class = Some(object_vector_i_class);
        i_class.instance_allocator = object_vector_i_class.instance_allocator();
        i_class.instance_init = object_vector_i_class.instance_init();
        i_class.call_handler = object_vector_i_class.call_handler();
        i_class.traits = OnceLock::from(Box::default());
        let i_class = Class(Gc::new(mc, i_class));

        let mut c_class = ClassData::empty(c_name);
        c_class.super_class = Some(context.avm2.class_defs().class);
        c_class.attributes = Cell::new(ClassAttributes::FINAL);
        c_class.instance_init = object_vector_c_class.instance_init();
        c_class.traits = OnceLock::from(Box::default());
        let c_class = Class(Gc::new(mc, c_class));

        i_class.link_with_c_class(mc, c_class);
        i_class.init_vtable_with_interfaces(context, Box::new([]));

        c_class.init_vtable_with_interfaces(context, Box::new([]));

        let write = unlock!(Gc::write(mc, this.0), ClassData, cell);
        write.borrow_mut().applications.insert(Some(param), i_class);
        i_class
    }

    /// Set the attributes of the class (sealed/final/interface status).
    pub fn set_attributes(self, attributes: ClassAttributes) {
        self.0.attributes.set(attributes);
    }

    pub fn add_class_object(self, mc: &Mutation<'gc>, class_object: ClassObject<'gc>) {
        let write = unlock!(Gc::write(mc, self.0), ClassData, cell);
        write.borrow_mut().class_objects.push(class_object);
    }

    pub fn class_objects(&self) -> Ref<'_, [ClassObject<'gc>]> {
        Ref::map(self.0.cell.borrow(), |c| &*c.class_objects)
    }

    pub fn class_object(self) -> Option<ClassObject<'gc>> {
        if let [obj] = *self.class_objects() {
            Some(obj)
        } else {
            None
        }
    }

    /// Construct a class from a `TranslationUnit` and its class index.
    ///
    /// The returned class will be allocated, but no traits will be loaded. The
    /// caller is responsible for storing the class in the `TranslationUnit`
    /// and calling `load_traits` to complete the trait-loading process.
    pub fn from_abc_index(
        unit: TranslationUnit<'gc>,
        class_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let mc = activation.gc();

        let i_class = Self::instance_from_abc_index(unit, class_index, activation)?;

        let c_class = Self::class_from_abc_index(
            unit,
            class_index,
            activation.avm2().class_defs().class,
            activation,
        )?;

        i_class.link_with_c_class(mc, c_class);

        Ok(i_class)
    }

    /// Loads an i_class from a `TranslationUnit`, without loading its c_class.
    pub fn instance_from_abc_index(
        unit: TranslationUnit<'gc>,
        class_index: u32,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let abc = unit.abc();
        let abc_instance = abc
            .instances
            .get(class_index as usize)
            .ok_or("LoadError: Instance index not valid")?;

        let name = QName::from_abc_multiname(activation, unit, abc_instance.name)?;

        let super_class = if abc_instance.super_name.0 == 0 {
            None
        } else {
            let multiname = unit.pool_multiname_static(activation, abc_instance.super_name)?;

            Some(
                activation
                    .domain()
                    .get_class(activation.context, &multiname)
                    .ok_or_else(|| {
                        make_error_1014(
                            activation,
                            Error1014Type::VerifyError,
                            multiname.to_qualified_name(activation.gc()),
                        )
                    })?,
            )
        };

        let protected_namespace = if let Some(ns) = &abc_instance.protected_namespace {
            Some(unit.pool_namespace(activation, *ns)?)
        } else {
            None
        };

        let interfaces = abc_instance
            .interfaces
            .iter()
            .map(|interface_name| {
                let multiname = unit.pool_multiname_static(activation, *interface_name)?;

                activation
                    .domain()
                    .get_class(activation.context, &multiname)
                    .ok_or_else(|| {
                        make_error_1014(
                            activation,
                            Error1014Type::VerifyError,
                            multiname.to_qualified_name(activation.gc()),
                        )
                    })
            })
            .collect::<Result<_, _>>()?;

        let instance_init = unit.load_method(abc_instance.init_method, false, activation)?;

        let mut attributes = ClassAttributes::empty();
        attributes.set(ClassAttributes::SEALED, abc_instance.is_sealed);
        attributes.set(ClassAttributes::FINAL, abc_instance.is_final);
        attributes.set(ClassAttributes::INTERFACE, abc_instance.is_interface);

        let mut instance_allocator = None;
        let mut call_handler = None;
        let mut custom_constructor = None;

        // When loading a class from our playerglobal, grab the corresponding native
        // allocator function from the table (which may be `None`)
        if unit.domain().is_playerglobals_domain(activation.avm2()) {
            instance_allocator = activation.avm2().native_instance_allocator_table
                [class_index as usize]
                .map(Allocator);

            if let Some(table_native_call_handler) =
                activation.avm2().native_call_handler_table[class_index as usize]
            {
                call_handler = Some(table_native_call_handler);
            }

            if let Some(table_custom_constructor) =
                activation.avm2().native_custom_constructor_table[class_index as usize]
            {
                custom_constructor = Some(table_custom_constructor);
            }
        }

        let instance_allocator = instance_allocator
            .or_else(|| super_class.map(|c| c.instance_allocator()))
            .unwrap_or(Allocator(scriptobject_allocator));

        let mut class = ClassData::empty(name);
        class.super_class = super_class;
        class.attributes = Cell::new(attributes);
        class.protected_namespace = protected_namespace;
        class.direct_interfaces = interfaces;
        class.instance_allocator = instance_allocator;
        class.instance_init = Some(instance_init);
        class.call_handler = call_handler;
        class.custom_constructor = custom_constructor.map(CustomConstructor);
        Ok(Class(Gc::new(activation.gc(), class)))
    }

    /// Loads a c_class from a `TranslationUnit`, without loading its i_class.
    pub fn class_from_abc_index(
        unit: TranslationUnit<'gc>,
        class_index: u32,
        class_class: Class<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let abc = unit.abc();

        let abc_instance = abc
            .instances
            .get(class_index as usize)
            .ok_or("LoadError: Instance index not valid")?;

        let abc_class = abc
            .classes
            .get(class_index as usize)
            .ok_or("LoadError: Class index not valid")?;

        // FIXME loading name again is a little wasteful
        let name = QName::from_abc_multiname(activation, unit, abc_instance.name)?;

        let protected_namespace = if let Some(ns) = &abc_instance.protected_namespace {
            Some(unit.pool_namespace(activation, *ns)?)
        } else {
            None
        };

        let class_init = unit.load_method(abc_class.init_method, false, activation)?;

        let name_namespace = name.namespace();
        let mut local_name_buf = WString::from(name.local_name().as_wstr());
        local_name_buf.push_char('$');

        let c_name = QName::new(
            name_namespace,
            AvmString::new(activation.gc(), local_name_buf),
        );

        let mut class = ClassData::empty(c_name);
        class.super_class = Some(class_class);
        class.attributes = Cell::new(ClassAttributes::FINAL);
        class.protected_namespace = protected_namespace;
        class.instance_init = Some(class_init);
        Ok(Class(Gc::new(activation.gc(), class)))
    }

    /// Finish the class-loading process by loading traits.
    ///
    /// This process must be done after the `Class` has been stored in the
    /// `TranslationUnit`. Failing to do so runs the risk of runaway recursion
    /// or double-borrows. It should be done before the class is actually
    /// instantiated into an `Object`.
    pub fn load_traits(
        self,
        activation: &mut Activation<'_, 'gc>,
        unit: TranslationUnit<'gc>,
        class_index: u32,
    ) -> Result<(), Error<'gc>> {
        let c_class = self
            .c_class()
            .expect("Class::load_traits should only be called on `i_class`es");

        self.load_instance_traits(activation, unit, class_index)?;
        c_class.load_class_traits(activation, unit, class_index)?;

        Ok(())
    }

    /// Loads traits for an i_class from the instance traits declared in the ABC file.
    pub fn load_instance_traits(
        self,
        activation: &mut Activation<'_, 'gc>,
        unit: TranslationUnit<'gc>,
        class_index: u32,
    ) -> Result<(), Error<'gc>> {
        let abc = unit.abc();

        let abc_instance = abc
            .instances
            .get(class_index as usize)
            .ok_or("LoadError: Instance index not valid")?;
        self.load_abc_traits(activation, unit, &abc_instance.traits)
    }

    /// Loads traits for a c_class from the class traits declared in the ABC file.
    pub fn load_class_traits(
        self,
        activation: &mut Activation<'_, 'gc>,
        unit: TranslationUnit<'gc>,
        class_index: u32,
    ) -> Result<(), Error<'gc>> {
        let abc = unit.abc();
        let abc_class = abc
            .classes
            .get(class_index as usize)
            .ok_or("LoadError: Class index not valid")?;
        self.load_abc_traits(activation, unit, &abc_class.traits)
    }

    fn load_abc_traits(
        self,
        activation: &mut Activation<'_, 'gc>,
        unit: TranslationUnit<'gc>,
        traits: &[AbcTrait],
    ) -> Result<(), Error<'gc>> {
        if self.0.traits.get().is_some() {
            return Ok(());
        }

        let traits = traits
            .iter()
            .map(|abc_traits| Trait::from_abc_trait(unit, abc_traits, activation))
            .collect::<Result<_, _>>()?;
        let write = Gc::write(activation.gc(), self.0);
        let _ = unlock!(write, ClassData, traits).set(traits);
        Ok(())
    }

    /// Completely validate a class against its resolved superclass.
    ///
    /// This should be called at class creation time once the superclass name
    /// has been resolved. It will return Ok for a valid class, and a
    /// VerifyError for any invalid class.
    pub fn validate_class(
        self,
        activation: &mut Activation<'_, 'gc>,
        allow_class_trait: bool,
    ) -> Result<(), Error<'gc>> {
        let mc = activation.gc();
        let superclass = self.0.super_class;

        if let Some(superclass) = superclass {
            // We have to make an exception for `c_class`es
            if superclass.is_final() && !self.is_c_class() {
                return Err(make_error_1103(activation, self));
            }

            if superclass.is_interface() {
                return Err(make_error_1110(activation, self, superclass));
            }

            for instance_trait in self.traits() {
                let is_protected = self.0.protected_namespace.is_some_and(|prot| {
                    prot.exact_version_match(instance_trait.name().namespace())
                });

                let mut current_superclass = Some(superclass);
                let mut did_override = false;

                while let Some(superclass) = current_superclass {
                    for supertrait in superclass.traits() {
                        let super_name = supertrait.name();
                        let my_name = instance_trait.name();

                        let names_match = super_name.local_name() == my_name.local_name()
                            && (super_name.namespace().matches_ns(my_name.namespace())
                                || (is_protected
                                    && superclass.protected_namespace().is_some_and(|prot| {
                                        prot.exact_version_match(super_name.namespace())
                                    })));
                        if names_match {
                            match (supertrait.kind(), instance_trait.kind()) {
                                //Getter/setter pairs do NOT override one another
                                (TraitKind::Getter { .. }, TraitKind::Setter { .. }) => continue,
                                (TraitKind::Setter { .. }, TraitKind::Getter { .. }) => continue,

                                (_, TraitKind::Const { .. }) | (_, TraitKind::Slot { .. }) => {
                                    did_override = true;

                                    // Const/Var traits override anything in avmplus
                                    // even if the base trait is marked as final or the
                                    // overriding trait isn't marked as override.
                                }
                                (_, TraitKind::Class { .. }) => {
                                    if !allow_class_trait {
                                        // Class traits aren't allowed in a class (except `global` classes)
                                        return Err(make_error_1059(activation));
                                    }
                                }
                                (TraitKind::Getter { .. }, TraitKind::Getter { .. })
                                | (TraitKind::Setter { .. }, TraitKind::Setter { .. })
                                | (TraitKind::Method { .. }, TraitKind::Method { .. }) => {
                                    did_override = true;

                                    if supertrait.is_final() {
                                        return Err(make_error_1053(
                                            activation,
                                            instance_trait.name().local_name(),
                                            self.name().to_qualified_name_err_message(mc),
                                        ));
                                    }

                                    if !instance_trait.is_override() {
                                        return Err(make_error_1053(
                                            activation,
                                            instance_trait.name().local_name(),
                                            self.name().to_qualified_name_err_message(mc),
                                        ));
                                    }
                                }
                                (_, TraitKind::Getter { .. })
                                | (_, TraitKind::Setter { .. })
                                | (_, TraitKind::Method { .. }) => {
                                    // Getters, setters, and methods cannot override
                                    // any other traits of a different type (except
                                    // slots, the logic for which is handled above)
                                    return Err(make_error_1053(
                                        activation,
                                        instance_trait.name().local_name(),
                                        self.name().to_qualified_name_err_message(mc),
                                    ));
                                }
                            }

                            break;
                        }
                    }

                    // The superclass is already validated so we don't need to
                    // check further.
                    if did_override {
                        break;
                    }

                    current_superclass = superclass.super_class();
                }

                if instance_trait.is_override() && !did_override {
                    return Err(make_error_1053(
                        activation,
                        instance_trait.name().local_name(),
                        self.name().to_qualified_name_err_message(mc),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Like validate_class, but instead validates the method signatures of
    /// all methods, getters, and setters in the class. This should be called
    /// at ClassObject construction time, after all classes are loaded.
    pub fn validate_signatures(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let mc = activation.gc();
        let superclass = self.0.super_class;

        if let Some(superclass) = superclass {
            for instance_trait in self.traits() {
                let is_protected = self.0.protected_namespace.is_some_and(|prot| {
                    prot.exact_version_match(instance_trait.name().namespace())
                });

                let mut current_superclass = Some(superclass);
                let mut found_match = false;

                while let Some(superclass) = current_superclass {
                    for supertrait in superclass.traits() {
                        let super_name = supertrait.name();
                        let my_name = instance_trait.name();

                        let names_match = super_name.local_name() == my_name.local_name()
                            && (super_name.namespace().matches_ns(my_name.namespace())
                                || (is_protected
                                    && superclass.protected_namespace().is_some_and(|prot| {
                                        prot.exact_version_match(super_name.namespace())
                                    })));
                        if names_match {
                            match (supertrait.kind(), instance_trait.kind()) {
                                (TraitKind::Getter { .. }, TraitKind::Setter { .. }) => continue,
                                (TraitKind::Setter { .. }, TraitKind::Getter { .. }) => continue,

                                (_, TraitKind::Const { .. })
                                | (_, TraitKind::Slot { .. })
                                | (_, TraitKind::Class { .. }) => {
                                    found_match = true;
                                }
                                (TraitKind::Getter { .. }, TraitKind::Getter { .. })
                                | (TraitKind::Setter { .. }, TraitKind::Setter { .. })
                                | (TraitKind::Method { .. }, TraitKind::Method { .. }) => {
                                    found_match = true;

                                    let instance_method = instance_trait.as_method().unwrap();
                                    instance_method.resolve_info(activation)?;

                                    let super_method = supertrait.as_method().unwrap();
                                    super_method.resolve_info(activation)?;

                                    // Methods must have same return type
                                    let instance_return_type =
                                        instance_method.resolved_return_type();
                                    let super_return_type = super_method.resolved_return_type();

                                    if instance_return_type != super_return_type {
                                        return Err(make_error_1053(
                                            activation,
                                            instance_trait.name().local_name(),
                                            self.name().to_qualified_name_err_message(mc),
                                        ));
                                    }
                                }
                                _ => unreachable!("Other trait combinations are invalid"),
                            }

                            break;
                        }
                    }

                    // The signature is already validated so we don't need to
                    // check further.
                    if found_match {
                        break;
                    }

                    current_superclass = superclass.super_class();
                }
            }
        }

        Ok(())
    }

    /// Initialize the vtable and interfaces of this Class.
    pub fn init_vtable(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        let interfaces = self.gather_interfaces(activation)?;

        self.init_vtable_with_interfaces(activation.context, interfaces);

        Ok(())
    }

    /// Given a list of interfaces, initialize the vtable and interfaces of this Class.
    /// This is useful when you want to create a class without an Activation and
    /// you already know which interfaces it has.
    pub fn init_vtable_with_interfaces(
        self,
        context: &mut UpdateContext<'gc>,
        interfaces: Box<[Class<'gc>]>,
    ) {
        if self.0.traits.get().is_none() {
            panic!(
                "Attempted to initialize vtable on a class that did not have its traits loaded yet"
            );
        } else if self.0.all_interfaces.get().is_some() {
            panic!("Attempted to initialize vtable twice");
        }

        let write = Gc::write(context.gc(), self.0);

        // The order is important here, as the VTable constructor needs the full list of interfaces.
        let _ = unlock!(write, ClassData, all_interfaces).set(interfaces);
        let _ = unlock!(write, ClassData, vtable).set(VTable::new_with_interface_properties(
            self,
            None,
            None,
            self.0.super_class.map(|c| c.vtable()),
            context,
        ));
    }

    /// Associate all the methods defined on this class with the specified
    /// MethodAssociation.
    pub fn bind_methods(
        self,
        activation: &mut Activation<'_, 'gc>,
        association: MethodAssociation<'gc>,
    ) -> Result<(), Error<'gc>> {
        let methods = self.traits().iter().filter_map(|t| t.as_method());
        for method in methods {
            method.associate(activation, association)?;
        }

        Ok(())
    }

    fn gather_interfaces(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Box<[Class<'gc>]>, Error<'gc>> {
        let mut interfaces = Vec::with_capacity(self.direct_interfaces().len());
        let mut dedup = HashSet::new();
        let mut queue = vec![self];
        while let Some(cls) = queue.pop() {
            for interface in cls.direct_interfaces() {
                if !interface.is_interface() {
                    return Err(make_error_1111(activation, self, *interface));
                }

                if dedup.insert(*interface) {
                    queue.push(*interface);
                    interfaces.push(*interface);
                }
            }

            if let Some(super_class) = cls.super_class() {
                queue.push(super_class);
            }
        }

        Ok(interfaces.into_boxed_slice())
    }

    pub fn for_activation(
        activation: &mut Activation<'_, 'gc>,
        translation_unit: TranslationUnit<'gc>,
        method: &AbcMethod,
        body: &AbcMethodBody,
    ) -> Result<Class<'gc>, Error<'gc>> {
        let name = translation_unit.pool_string(method.name.as_u30(), activation.strings())?;

        let load_trait = |trait_entry| -> Result<Trait<'gc>, Error<'gc>> {
            let loaded_trait = Trait::from_abc_trait(translation_unit, trait_entry, activation)?;

            // Methods, getters, and setters are forbidden from appearing
            // in activation traits
            if loaded_trait.as_method().is_some() {
                return Err(make_error_1107(activation));
            }
            Ok(loaded_trait)
        };

        let traits: Box<[Trait<'gc>]> = body
            .traits
            .iter()
            .map(load_trait)
            .collect::<Result<_, _>>()?;

        let name = QName::new(activation.avm2().namespaces.public_all(), name);

        let mut i_class = ClassData::empty(name);
        i_class.attributes = Cell::new(ClassAttributes::FINAL | ClassAttributes::SEALED);
        i_class.traits = OnceLock::from(traits);
        let i_class = Class(Gc::new(activation.gc(), i_class));
        i_class.init_vtable(activation)?;

        // We don't need to construct a c_class

        Ok(i_class)
    }

    pub fn for_catch(
        activation: &mut Activation<'_, 'gc>,
        variable_name: QName<'gc>,
    ) -> Result<Class<'gc>, Error<'gc>> {
        // Yes, the name of the class is the variable's name
        let mut i_class = ClassData::empty(variable_name);
        i_class.attributes = Cell::new(ClassAttributes::FINAL | ClassAttributes::SEALED);
        // TODO make the slot typed
        let domain = activation.avm2().playerglobals_domain;
        let traits: Box<[_]> = Box::new([Trait::from_const(variable_name, None, None, domain)]);
        i_class.traits = OnceLock::from(traits);
        let i_class = Class(Gc::new(activation.gc(), i_class));
        i_class.init_vtable(activation)?;

        // We don't need to construct a c_class

        Ok(i_class)
    }

    /// Determine if this class has a given type in its superclass chain.
    ///
    /// The given class `test_class` should be either a superclass or
    /// interface we are checking against this class.
    ///
    /// To test if a class *instance* is of a given type, see `Object::is_of_type`.
    pub fn has_class_in_chain(self, test_class: Class<'gc>) -> bool {
        let mut my_class = Some(self);

        while let Some(class) = my_class {
            if class == test_class {
                return true;
            }

            my_class = class.super_class()
        }

        // A `Class` stores all of the interfaces it implements, including
        // those from superinterfaces and superclasses (recursively).
        if test_class.is_interface() {
            for interface in self.all_interfaces() {
                if *interface == test_class {
                    return true;
                }
            }
        }

        false
    }

    pub fn vtable(self) -> VTable<'gc> {
        *self.0.vtable.get().expect("VTable not yet initialized!")
    }

    pub fn dollar_removed_name(self, mc: &Mutation<'gc>) -> QName<'gc> {
        let name = self.name();

        let namespace = name.namespace();

        let local_name = name.local_name();
        let local_name_wstr = local_name.as_wstr();

        // Matching avmplus, this doesn't check whether the class is a
        // c_class; it strips the suffix even for i_classes
        if let Some(stripped) = local_name_wstr.strip_suffix(b'$') {
            let new_local_name = AvmString::new(mc, stripped);

            QName::new(namespace, new_local_name)
        } else {
            name
        }
    }

    pub fn name(self) -> QName<'gc> {
        self.0.name.get()
    }

    pub fn set_name(self, mc: &Mutation<'gc>, name: QName<'gc>) {
        unlock!(Gc::write(mc, self.0), ClassData, name).set(name);
    }

    pub fn param(self) -> Option<Option<Class<'gc>>> {
        self.0.param.get()
    }

    pub fn set_param(self, mc: &Mutation<'gc>, param: Option<Option<Class<'gc>>>) {
        unlock!(Gc::write(mc, self.0), ClassData, param).set(param);
    }

    pub fn super_class(self) -> Option<Class<'gc>> {
        self.0.super_class
    }

    pub fn super_class_name(self) -> Option<Multiname<'gc>> {
        self.0.super_class.map(|c| c.name().into())
    }

    pub fn protected_namespace(self) -> Option<Namespace<'gc>> {
        self.0.protected_namespace
    }

    /// Return traits provided by this class.
    pub fn traits(&self) -> &[Trait<'gc>] {
        self.0.traits.get().expect("Traits not yet initialized!")
    }

    /// Get this class's instance allocator.
    ///
    /// If `None`, then you should use the instance allocator of the superclass
    /// or allocate as a `ScriptObject` if no such class exists.
    pub fn instance_allocator(self) -> Allocator {
        self.0.instance_allocator
    }

    /// Get this class's custom constructor.
    ///
    /// If `None`, then this class should be constructed normally.
    pub fn custom_constructor(self) -> Option<CustomConstructor> {
        self.0.custom_constructor
    }

    /// Get this class's instance initializer.
    pub fn instance_init(self) -> Option<Method<'gc>> {
        self.0.instance_init
    }

    /// Get this class's call handler.
    pub fn call_handler(self) -> Option<NativeMethodImpl> {
        self.0.call_handler
    }

    pub fn direct_interfaces(&self) -> &[Class<'gc>] {
        &self.0.direct_interfaces
    }

    pub fn all_interfaces(&self) -> &[Class<'gc>] {
        self.0
            .all_interfaces
            .get()
            .expect("Interfaces not yet initialized!")
    }

    /// Determine if this class is sealed (no dynamic properties)
    pub fn is_sealed(self) -> bool {
        self.0.attributes.get().contains(ClassAttributes::SEALED)
    }

    /// Determine if this class is final (cannot be subclassed)
    pub fn is_final(self) -> bool {
        self.0.attributes.get().contains(ClassAttributes::FINAL)
    }

    /// Determine if this class is an interface
    pub fn is_interface(self) -> bool {
        self.0.attributes.get().contains(ClassAttributes::INTERFACE)
    }

    /// Determine if this class is generic (can be specialized)
    pub fn is_generic(self) -> bool {
        self.0.attributes.get().contains(ClassAttributes::GENERIC)
    }

    /// Whether this class is one of the numerical classes (`int`, `uint`, or
    /// `Number`)
    pub fn is_builtin_numeric(self) -> bool {
        self.is_builtin_int() || self.is_builtin_uint() || self.is_builtin_number()
    }

    /// Whether this class is one of the non-null primitive classes (`int`,
    /// `uint`, `Number`, `Boolean`, or `void`). These classes can never
    /// represent a `null` value.
    pub fn is_builtin_non_null(self) -> bool {
        self.is_builtin_int()
            || self.is_builtin_uint()
            || self.is_builtin_number()
            || self.is_builtin_boolean()
            || self.is_builtin_void()
    }

    pub fn is_builtin_int(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::Int))
    }
    pub fn is_builtin_uint(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::Uint))
    }
    pub fn is_builtin_number(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::Number))
    }
    pub fn is_builtin_boolean(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::Boolean))
    }
    pub fn is_builtin_void(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::Void))
    }
    pub fn is_builtin_object(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::Object))
    }
    pub fn is_builtin_string(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::String))
    }
    pub fn is_script_traits(self) -> bool {
        matches!(self.0.builtin_type.get(), Some(BuiltinType::ScriptTraits))
    }

    pub fn mark_builtin_type(self, builtin_type: BuiltinType) {
        assert!(self.0.builtin_type.get().is_none());

        self.0.builtin_type.set(Some(builtin_type));
    }

    pub fn c_class(self) -> Option<Class<'gc>> {
        if let ClassLink::LinkToClass(c_class) = self.0.linked_class.get() {
            Some(c_class)
        } else {
            None
        }
    }

    pub fn is_c_class(self) -> bool {
        matches!(self.0.linked_class.get(), ClassLink::LinkToInstance(_))
    }

    pub fn i_class(self) -> Option<Class<'gc>> {
        if let ClassLink::LinkToInstance(i_class) = self.0.linked_class.get() {
            Some(i_class)
        } else {
            None
        }
    }

    pub fn is_i_class(self) -> bool {
        matches!(self.0.linked_class.get(), ClassLink::LinkToClass(_))
    }

    pub fn link_with_c_class(self, mc: &Mutation<'gc>, c_class: Class<'gc>) {
        let i_class = self;
        let i_link = unlock!(Gc::write(mc, i_class.0), ClassData, linked_class);
        let c_link = unlock!(Gc::write(mc, c_class.0), ClassData, linked_class);

        assert!(matches!(i_link.get(), ClassLink::Unlinked));
        assert!(matches!(c_link.get(), ClassLink::Unlinked));

        i_link.set(ClassLink::LinkToClass(c_class));
        c_link.set(ClassLink::LinkToInstance(i_class));
    }
}

// NOTE: The ordering of these variants is important; Int, Uint, and Number are
// the first three, so that a "is numerical" check can simply check if the
// enum tag is <=2, and Boolean and Void come right after, so that a "is null"
// check can check if the enum tag is <=4.
#[derive(Clone, Copy)]
pub enum BuiltinType {
    // `int`, `uint`, `Number`, `Boolean`, `void`, `Object`, `String` builtin
    // classes
    Int,
    Uint,
    Number,
    Boolean,
    Void,
    Object,
    String,

    // Any script `global` class
    ScriptTraits,
}
