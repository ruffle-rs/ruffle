//! AVM2 classes

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_1014;
use crate::avm2::method::{Method, NativeMethodImpl};
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
use crate::string::WString;
use bitflags::bitflags;
use fnv::FnvHashMap;
use gc_arena::{Collect, Gc, GcCell, Mutation};

use std::cell::Ref;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

use swf::avm2::types::{
    Class as AbcClass, Instance as AbcInstance, Method as AbcMethod, MethodBody as AbcMethodBody,
};

use super::method::ParamConfig;
use super::string::AvmString;

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

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Allocator")
            .field(&"<native code>".to_string())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
enum ClassLink<'gc> {
    Unlinked,
    LinkToInstance(Class<'gc>),
    LinkToClass(Class<'gc>),
}

#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct Class<'gc>(pub GcCell<'gc, ClassData<'gc>>);

/// A loaded ABC Class which can be used to construct objects with.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ClassData<'gc> {
    /// The name of the class.
    name: QName<'gc>,

    /// The type parameter for this class (only supported for Vector)
    param: Option<Option<Class<'gc>>>,

    /// This class's superclass, or None if it has no superclass
    super_class: Option<Class<'gc>>,

    /// Attributes of the given class.
    #[collect(require_static)]
    attributes: ClassAttributes,

    /// The namespace that protected traits of this class are stored into.
    protected_namespace: Option<Namespace<'gc>>,

    /// The list of interfaces this class directly implements. This does not include any
    /// superinterfaces, nor interfaces implemented by the superclass.
    direct_interfaces: Vec<Class<'gc>>,

    /// Interfaces implemented by this class, including interfaces
    /// from parent classes and superinterfaces (recursively).
    /// TODO - avoid cloning this when a subclass implements the
    /// same interface as its superclass.
    all_interfaces: Vec<Class<'gc>>,

    /// The instance allocator for this class. Defaults to the script object allocator
    /// if no allocator is provided.
    #[collect(require_static)]
    instance_allocator: Allocator,

    /// The instance initializer for this class.
    ///
    /// Must be called each time a new class instance is constructed.
    instance_init: Method<'gc>,

    /// The super initializer for this class, called when super() is called for
    /// a subclass.
    ///
    /// This may be provided to allow natively-constructed classes to
    /// initialize themselves in a different manner from user-constructed ones.
    /// For example, the user-accessible constructor may error out (as it's not
    /// a valid class to construct for users), but native code may still call
    /// its constructor stack.
    ///
    /// By default, a class's `super_init` will be initialized to the
    /// same method as the regular one. You must specify a separate super
    /// initializer to change initialization behavior based on what code is
    /// constructing the class.
    super_init: Method<'gc>,

    /// Traits for a given class.
    ///
    /// These are accessed as normal instance properties; they should not be
    /// present on prototypes, but instead should shadow any prototype
    /// properties that would match.
    traits: Vec<Trait<'gc>>,

    vtable: VTable<'gc>,

    /// The customization point for `Class(args...)` without `new`
    /// If None, a simple coercion is done.
    call_handler: Option<Method<'gc>>,

    /// Whether or not this `Class` has loaded its traits or not.
    traits_loaded: bool,

    /// Maps a type parameter to the application of this class with that parameter.
    ///
    /// Only applicable if this class is generic.
    applications: FnvHashMap<Option<Class<'gc>>, Class<'gc>>,

    /// Whether or not this is a system-defined class.
    ///
    /// System defined classes are allowed to have illegal trait configurations
    /// without throwing a VerifyError.
    is_system: bool,

    /// The Class this Class is linked to. If this class represents instance info,
    /// this will be a ClassLink::LinkToClass. If this class represents class info,
    /// this will be a ClassLink::LinkToInstance. This must be one of the two,
    /// unless this class has not yet been fully initialized, in which case it will
    /// be set to ClassLink::Unlinked.
    linked_class: ClassLink<'gc>,

    /// The ClassObjects for this class.
    /// In almost all cases, this will either be empty or have a single object.
    /// However, a swf can run `newclass` multiple times on the same class
    /// to create multiple `ClassObjects`.
    class_objects: Vec<ClassObject<'gc>>,
}

impl PartialEq for Class<'_> {
    fn eq(&self, other: &Self) -> bool {
        GcCell::ptr_eq(self.0, other.0)
    }
}

impl Eq for Class<'_> {}

impl Hash for Class<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl<'gc> core::fmt::Debug for Class<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Class").field("name", &self.name()).finish()
    }
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
        super_class: Option<Class<'gc>>,
        instance_init: Method<'gc>,
        class_init: Method<'gc>,
        class_i_class: Class<'gc>,
        mc: &Mutation<'gc>,
    ) -> Self {
        let super_init = instance_init;

        let instance_allocator = super_class
            .map(|c| c.instance_allocator())
            .unwrap_or(Allocator(scriptobject_allocator));

        let i_class = Class(GcCell::new(
            mc,
            ClassData {
                name,
                param: None,
                super_class,
                attributes: ClassAttributes::empty(),
                protected_namespace: None,
                direct_interfaces: Vec::new(),
                all_interfaces: Vec::new(),
                instance_allocator,
                instance_init,
                super_init,
                traits: Vec::new(),
                vtable: VTable::empty(mc),
                call_handler: None,
                traits_loaded: false,
                is_system: true,
                linked_class: ClassLink::Unlinked,
                applications: FnvHashMap::default(),
                class_objects: Vec::new(),
            },
        ));

        let name_namespace = name.namespace();
        let mut local_name_buf = WString::from(name.local_name().as_wstr());
        local_name_buf.push_char('$');

        let c_name = QName::new(name_namespace, AvmString::new(mc, local_name_buf));

        let c_class = Class(GcCell::new(
            mc,
            ClassData {
                name: c_name,
                param: None,
                super_class: Some(class_i_class),
                attributes: ClassAttributes::FINAL,
                protected_namespace: None,
                direct_interfaces: Vec::new(),
                all_interfaces: Vec::new(),
                instance_allocator: Allocator(scriptobject_allocator),
                instance_init: class_init,
                super_init: class_init,
                traits: Vec::new(),
                vtable: VTable::empty(mc),
                call_handler: None,
                traits_loaded: false,
                is_system: true,
                linked_class: ClassLink::LinkToInstance(i_class),
                applications: FnvHashMap::default(),
                class_objects: Vec::new(),
            },
        ));

        i_class.set_c_class(mc, c_class);

        i_class
    }

    pub fn custom_new(
        name: QName<'gc>,
        super_class: Option<Class<'gc>>,
        instance_init: Method<'gc>,
        mc: &Mutation<'gc>,
    ) -> Self {
        let super_init = instance_init;

        Class(GcCell::new(
            mc,
            ClassData {
                name,
                param: None,
                super_class,
                attributes: ClassAttributes::empty(),
                protected_namespace: None,
                direct_interfaces: Vec::new(),
                all_interfaces: Vec::new(),
                instance_allocator: Allocator(scriptobject_allocator),
                instance_init,
                super_init,
                traits: Vec::new(),
                vtable: VTable::empty(mc),
                call_handler: None,
                traits_loaded: false,
                is_system: true,
                linked_class: ClassLink::Unlinked,
                applications: FnvHashMap::default(),
                class_objects: Vec::new(),
            },
        ))
    }

    pub fn add_application(self, mc: &Mutation<'gc>, param: Option<Class<'gc>>, cls: Class<'gc>) {
        self.0.write(mc).applications.insert(param, cls);
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
        let mc = context.gc_context;
        let this_read = this.0.read();

        if let Some(application) = this_read.applications.get(&param) {
            return *application;
        }

        // This can only happen for non-builtin Vector types,
        // so let's create one here directly.

        let object_vector_i_class = this_read
            .applications
            .get(&None)
            .expect("Vector.<*> not initialized?");

        let object_vector_c_class = object_vector_i_class
            .c_class()
            .expect("T$ cannot be generic");

        let param = param.expect("Trying to create Vector<*>, which shouldn't happen here");
        let name = format!("Vector.<{}>", param.name().to_qualified_name(mc));

        let new_class = Self::new(
            // FIXME - we should store a `Multiname` instead of a `QName`, and use the
            // `params` field. For now, this is good enough to get tests passing
            QName::new(this.name().namespace(), AvmString::new_utf8(mc, name)),
            Some(
                context
                    .avm2
                    .classes()
                    .object_vector
                    .inner_class_definition(),
            ),
            object_vector_i_class.instance_init(),
            object_vector_c_class.instance_init(),
            context.avm2.class_defs().class,
            mc,
        );

        new_class.set_param(mc, Some(Some(param)));
        new_class.0.write(mc).call_handler = object_vector_i_class.call_handler();

        new_class.mark_traits_loaded(context.gc_context);
        new_class
            .init_vtable(context)
            .expect("Vector class doesn't have any interfaces, so `init_vtable` cannot error");

        let c_class = new_class.c_class().expect("Class::new returns an i_class");

        c_class.mark_traits_loaded(context.gc_context);
        c_class
            .init_vtable(context)
            .expect("Vector$ class doesn't have any interfaces, so `init_vtable` cannot error");

        drop(this_read);

        this.0.write(mc).applications.insert(Some(param), new_class);
        new_class
    }

    /// Set the attributes of the class (sealed/final/interface status).
    pub fn set_attributes(self, mc: &Mutation<'gc>, attributes: ClassAttributes) {
        self.0.write(mc).attributes = attributes;
    }

    pub fn add_class_object(self, mc: &Mutation<'gc>, class_object: ClassObject<'gc>) {
        self.0.write(mc).class_objects.push(class_object);
    }

    pub fn class_objects(&self) -> Ref<Vec<ClassObject<'gc>>> {
        Ref::map(self.0.read(), |c| &c.class_objects)
    }

    pub fn class_object(self) -> Option<ClassObject<'gc>> {
        let read = self.0.read();

        if read.class_objects.len() == 1 {
            Some(read.class_objects[0])
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
        let abc = unit.abc();
        let abc_class: Result<&AbcClass, Error<'gc>> = abc
            .classes
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Class index not valid".into());
        let abc_class = abc_class?;

        let abc_instance: Result<&AbcInstance, Error<'gc>> = abc
            .instances
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Instance index not valid".into());
        let abc_instance = abc_instance?;

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
                        make_error_1014(activation, multiname.to_qualified_name(activation.gc()))
                    })?,
            )
        };

        let protected_namespace = if let Some(ns) = &abc_instance.protected_namespace {
            Some(unit.pool_namespace(activation, *ns)?)
        } else {
            None
        };

        let mut interfaces = Vec::with_capacity(abc_instance.interfaces.len());
        for interface_name in &abc_instance.interfaces {
            let multiname = unit.pool_multiname_static(activation, *interface_name)?;

            interfaces.push(
                activation
                    .domain()
                    .get_class(activation.context, &multiname)
                    .ok_or_else(|| {
                        make_error_1014(activation, multiname.to_qualified_name(activation.gc()))
                    })?,
            );
        }

        let instance_init = unit.load_method(abc_instance.init_method, false, activation)?;
        let mut super_init = instance_init;
        let class_init = unit.load_method(abc_class.init_method, false, activation)?;
        let mut native_call_handler = None;

        let mut attributes = ClassAttributes::empty();
        attributes.set(ClassAttributes::SEALED, abc_instance.is_sealed);
        attributes.set(ClassAttributes::FINAL, abc_instance.is_final);
        attributes.set(ClassAttributes::INTERFACE, abc_instance.is_interface);

        let mut instance_allocator = None;

        // When loading a class from our playerglobal, grab the corresponding native
        // allocator function from the table (which may be `None`)
        if unit.domain().is_playerglobals_domain(activation.avm2()) {
            instance_allocator = activation.avm2().native_instance_allocator_table
                [class_index as usize]
                .map(|(_name, ptr)| Allocator(ptr));

            if let Some((name, table_native_init)) =
                activation.avm2().native_super_initializer_table[class_index as usize]
            {
                let method = Method::from_builtin_and_params(
                    table_native_init,
                    name,
                    instance_init.signature().to_vec(),
                    instance_init.return_type(),
                    instance_init.is_variadic(),
                    activation.context.gc_context,
                );
                super_init = method;
            }

            if let Some((name, table_native_call_handler)) =
                activation.avm2().native_call_handler_table[class_index as usize]
            {
                let method = Method::from_builtin_and_params(
                    table_native_call_handler,
                    name,
                    // A 'callable' class doesn't have a signature - let the
                    // method do any needed coercions
                    vec![],
                    None,
                    true,
                    activation.context.gc_context,
                );
                native_call_handler = Some(method);
            }
        }

        let instance_allocator = instance_allocator
            .or_else(|| super_class.map(|c| c.instance_allocator()))
            .unwrap_or(Allocator(scriptobject_allocator));

        let i_class = Class(GcCell::new(
            activation.context.gc_context,
            ClassData {
                name,
                param: None,
                super_class,
                attributes,
                protected_namespace,
                direct_interfaces: interfaces,
                all_interfaces: Vec::new(),
                instance_allocator,
                instance_init,
                super_init,
                traits: Vec::new(),
                vtable: VTable::empty(activation.context.gc_context),
                call_handler: native_call_handler,
                traits_loaded: false,
                is_system: false,
                linked_class: ClassLink::Unlinked,
                applications: Default::default(),
                class_objects: Vec::new(),
            },
        ));

        let name_namespace = name.namespace();
        let mut local_name_buf = WString::from(name.local_name().as_wstr());
        local_name_buf.push_char('$');

        let c_name = QName::new(
            name_namespace,
            AvmString::new(activation.context.gc_context, local_name_buf),
        );

        let c_class = Class(GcCell::new(
            activation.context.gc_context,
            ClassData {
                name: c_name,
                param: None,
                super_class: Some(activation.avm2().class_defs().class),
                attributes: ClassAttributes::FINAL,
                protected_namespace,
                direct_interfaces: Vec::new(),
                all_interfaces: Vec::new(),
                instance_allocator: Allocator(scriptobject_allocator),
                instance_init: class_init,
                super_init: class_init,
                traits: Vec::new(),
                vtable: VTable::empty(activation.context.gc_context),
                call_handler: None,
                traits_loaded: false,
                is_system: false,
                linked_class: ClassLink::LinkToInstance(i_class),
                applications: FnvHashMap::default(),
                class_objects: Vec::new(),
            },
        ));

        i_class.set_c_class(activation.context.gc_context, c_class);

        Ok(i_class)
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
        // We should only call `load_traits` on `i_class` Classes
        // (if i_class() is None it means that the Class is an i_class)
        assert!(self.i_class().is_none());

        if self.0.read().traits_loaded {
            return Ok(());
        }

        let c_class = self
            .c_class()
            .expect("Just checked that this class was an i_class");
        let mut c_class_write = c_class.0.write(activation.context.gc_context);

        let mut i_class_write = self.0.write(activation.context.gc_context);

        i_class_write.traits_loaded = true;
        c_class_write.traits_loaded = true;

        let abc = unit.abc();
        let abc_class: Result<&AbcClass, Error<'gc>> = abc
            .classes
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Class index not valid".into());
        let abc_class = abc_class?;

        let abc_instance: Result<&AbcInstance, Error<'gc>> = abc
            .instances
            .get(class_index as usize)
            .ok_or_else(|| "LoadError: Instance index not valid".into());
        let abc_instance = abc_instance?;

        for abc_trait in abc_instance.traits.iter() {
            i_class_write
                .traits
                .push(Trait::from_abc_trait(unit, abc_trait, activation)?);
        }

        for abc_trait in abc_class.traits.iter() {
            c_class_write
                .traits
                .push(Trait::from_abc_trait(unit, abc_trait, activation)?);
        }

        Ok(())
    }

    /// Completely validate a class against it's resolved superclass.
    ///
    /// This should be called at class creation time once the superclass name
    /// has been resolved. It will return Ok for a valid class, and a
    /// VerifyError for any invalid class.
    pub fn validate_class(self, superclass: Option<ClassObject<'gc>>) -> Result<(), Error<'gc>> {
        let read = self.0.read();

        // System classes do not throw verify errors.
        if read.is_system {
            return Ok(());
        }

        if let Some(superclass) = superclass {
            for instance_trait in read.traits.iter() {
                let is_protected = read.protected_namespace.map_or(false, |prot| {
                    prot.exact_version_match(instance_trait.name().namespace())
                });

                let mut current_superclass = Some(superclass);
                let mut did_override = false;

                while let Some(superclass) = current_superclass {
                    let superclass_def = superclass.inner_class_definition();

                    for supertrait in &*superclass_def.traits() {
                        let super_name = supertrait.name();
                        let my_name = instance_trait.name();

                        let names_match = super_name.local_name() == my_name.local_name()
                            && (super_name.namespace().matches_ns(my_name.namespace())
                                || (is_protected
                                    && superclass_def
                                        .protected_namespace()
                                        .map_or(false, |prot| {
                                            prot.exact_version_match(super_name.namespace())
                                        })));
                        if names_match {
                            match (supertrait.kind(), instance_trait.kind()) {
                                //Getter/setter pairs do NOT override one another
                                (TraitKind::Getter { .. }, TraitKind::Setter { .. }) => continue,
                                (TraitKind::Setter { .. }, TraitKind::Getter { .. }) => continue,
                                (_, _) => did_override = true,
                            }

                            if supertrait.is_final() {
                                return Err(format!("VerifyError: Trait {} in class {} overrides final trait {} in class {}", instance_trait.name().local_name(), superclass_def.name().local_name(), supertrait.name().local_name(), superclass_def.name().local_name()).into());
                            }

                            if !instance_trait.is_override() {
                                return Err(format!("VerifyError: Trait {} in class {} has same name as trait {} in class {}, but does not override it", instance_trait.name().local_name(), self.name().local_name(), supertrait.name().local_name(), superclass_def.name().local_name()).into());
                            }

                            break;
                        }
                    }

                    // The superclass is already validated so we don't need to
                    // check further.
                    if did_override {
                        break;
                    }

                    current_superclass = superclass.superclass_object();
                }

                if instance_trait.is_override() && !did_override {
                    return Err(format!("VerifyError: Trait {} in class {:?} marked as override, does not override any other trait", instance_trait.name().local_name(), read.name).into());
                }
            }
        }

        Ok(())
    }

    pub fn init_vtable(self, context: &mut UpdateContext<'gc>) -> Result<(), Error<'gc>> {
        let read = self.0.read();

        if !read.traits_loaded {
            panic!(
                "Attempted to initialize vtable on a class that did not have its traits loaded yet"
            );
        }

        read.vtable.init_vtable(
            self,
            None,
            None,
            read.super_class.map(|c| c.vtable()),
            context.gc_context,
        );
        drop(read);

        self.link_interfaces(context)?;

        Ok(())
    }

    pub fn link_interfaces(self, context: &mut UpdateContext<'gc>) -> Result<(), Error<'gc>> {
        let mut interfaces = Vec::with_capacity(self.direct_interfaces().len());

        let mut dedup = HashSet::new();
        let mut queue = vec![self];
        while let Some(cls) = queue.pop() {
            for interface in &*cls.direct_interfaces() {
                if !interface.is_interface() {
                    return Err(format!(
                        "Class {:?} is not an interface and cannot be implemented by classes",
                        interface.name().local_name()
                    )
                    .into());
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

        // FIXME - we should only be copying properties for newly-implemented
        // interfaces (i.e. those that were not already implemented by the superclass)
        // Otherwise, our behavior diverges from Flash Player in certain cases.
        // See the ignored test 'tests/tests/swfs/avm2/weird_superinterface_properties/'
        let ns = context.avm2.namespaces.public_vm_internal();
        for interface in &interfaces {
            for interface_trait in &*interface.traits() {
                if !interface_trait.name().namespace().is_public() {
                    let public_name = QName::new(ns, interface_trait.name().local_name());
                    self.0.read().vtable.copy_property_for_interface(
                        context.gc_context,
                        public_name,
                        interface_trait.name(),
                    );
                }
            }
        }

        self.0.write(context.gc_context).all_interfaces = interfaces;

        Ok(())
    }

    pub fn for_activation(
        activation: &mut Activation<'_, 'gc>,
        translation_unit: TranslationUnit<'gc>,
        method: &AbcMethod,
        body: &AbcMethodBody,
    ) -> Result<Class<'gc>, Error<'gc>> {
        let name = translation_unit.pool_string(method.name.as_u30(), activation.strings())?;
        let mut traits = Vec::with_capacity(body.traits.len());

        for trait_entry in body.traits.iter() {
            traits.push(Trait::from_abc_trait(
                translation_unit,
                trait_entry,
                activation,
            )?);
        }

        let name = QName::new(activation.avm2().namespaces.public_all(), name);

        let i_class = Class(GcCell::new(
            activation.context.gc_context,
            ClassData {
                name,
                param: None,
                super_class: None,
                attributes: ClassAttributes::FINAL,
                protected_namespace: None,
                direct_interfaces: Vec::new(),
                all_interfaces: Vec::new(),
                instance_allocator: Allocator(scriptobject_allocator),
                instance_init: Method::from_builtin(
                    |_, _, _| Ok(Value::Undefined),
                    "<Activation object constructor>",
                    activation.context.gc_context,
                ),
                super_init: Method::from_builtin(
                    |_, _, _| Ok(Value::Undefined),
                    "<Activation object constructor>",
                    activation.context.gc_context,
                ),
                traits,
                vtable: VTable::empty(activation.context.gc_context),
                call_handler: None,
                traits_loaded: true,
                is_system: false,
                linked_class: ClassLink::Unlinked,
                applications: Default::default(),
                class_objects: Vec::new(),
            },
        ));

        let name_namespace = name.namespace();
        let mut local_name_buf = WString::from(name.local_name().as_wstr());
        local_name_buf.push_char('$');

        let c_name = QName::new(
            name_namespace,
            AvmString::new(activation.context.gc_context, local_name_buf),
        );

        let c_class = Class(GcCell::new(
            activation.context.gc_context,
            ClassData {
                name: c_name,
                param: None,
                super_class: Some(activation.avm2().class_defs().class),
                attributes: ClassAttributes::FINAL,
                protected_namespace: None,
                direct_interfaces: Vec::new(),
                all_interfaces: Vec::new(),
                instance_allocator: Allocator(scriptobject_allocator),
                instance_init: Method::from_builtin(
                    |_, _, _| Ok(Value::Undefined),
                    "<Activation object class constructor>",
                    activation.context.gc_context,
                ),
                super_init: Method::from_builtin(
                    |_, _, _| Ok(Value::Undefined),
                    "<Activation object class constructor>",
                    activation.context.gc_context,
                ),
                traits: Vec::new(),
                vtable: VTable::empty(activation.context.gc_context),
                call_handler: None,
                traits_loaded: true,
                is_system: false,
                linked_class: ClassLink::LinkToInstance(i_class),
                applications: FnvHashMap::default(),
                class_objects: Vec::new(),
            },
        ));

        i_class.set_c_class(activation.context.gc_context, c_class);

        i_class.init_vtable(activation.context)?;
        c_class.init_vtable(activation.context)?;

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
            for interface in &*self.all_interfaces() {
                if *interface == test_class {
                    return true;
                }
            }
        }

        false
    }

    pub fn vtable(self) -> VTable<'gc> {
        self.0.read().vtable
    }

    pub fn dollar_removed_name(self, mc: &Mutation<'gc>) -> QName<'gc> {
        let name = self.0.read().name;

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
        self.0.read().name
    }

    pub fn try_name(self) -> Result<QName<'gc>, std::cell::BorrowError> {
        self.0.try_read().map(|r| r.name)
    }

    /// Attempts to obtain the name of this class.
    /// If we are unable to read from the necessary `GcCell`,
    /// the returned value will be some kind of error message.
    ///
    /// This should only be used in a debug context, where
    /// we need infallible access to *something* to print
    /// out.
    pub fn debug_name(self) -> Box<dyn fmt::Debug + 'gc> {
        let class_name = self.try_name();

        match class_name {
            Ok(class_name) => Box::new(class_name),
            Err(err) => Box::new(err),
        }
    }

    pub fn param(self) -> Option<Option<Class<'gc>>> {
        self.0.read().param
    }

    pub fn set_param(self, mc: &Mutation<'gc>, param: Option<Option<Class<'gc>>>) {
        self.0.write(mc).param = param;
    }

    pub fn super_class(self) -> Option<Class<'gc>> {
        self.0.read().super_class
    }

    pub fn super_class_name(self) -> Option<Multiname<'gc>> {
        self.0.read().super_class.map(|c| c.name().into())
    }

    pub fn protected_namespace(self) -> Option<Namespace<'gc>> {
        self.0.read().protected_namespace
    }

    pub fn mark_traits_loaded(self, mc: &Mutation<'gc>) {
        self.0.write(mc).traits_loaded = true;
    }

    #[inline(never)]
    pub fn define_constant_class_instance_trait(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: QName<'gc>,
        class_object: ClassObject<'gc>,
    ) {
        self.define_instance_trait(
            activation.context.gc_context,
            Trait::from_class(name, class_object.inner_class_definition()),
        );
    }

    #[inline(never)]
    pub fn define_constant_function_instance_trait(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: QName<'gc>,
        value: Value<'gc>,
    ) {
        self.define_instance_trait(
            activation.gc(),
            Trait::from_const(
                name,
                Some(activation.avm2().multinames.function),
                Some(value),
            ),
        );
    }

    #[inline(never)]
    pub fn define_constant_number_class_traits(
        self,
        namespace: Namespace<'gc>,
        items: &[(&'static str, f64)],
        activation: &mut Activation<'_, 'gc>,
    ) {
        for &(name, value) in items {
            self.define_class_trait(
                activation.gc(),
                Trait::from_const(
                    QName::new(namespace, name),
                    Some(activation.avm2().multinames.number),
                    Some(value.into()),
                ),
            );
        }
    }

    #[inline(never)]
    pub fn define_constant_uint_class_traits(
        self,
        namespace: Namespace<'gc>,
        items: &[(&'static str, u32)],
        activation: &mut Activation<'_, 'gc>,
    ) {
        for &(name, value) in items {
            self.define_class_trait(
                activation.gc(),
                Trait::from_const(
                    QName::new(namespace, name),
                    Some(activation.avm2().multinames.uint),
                    Some(value.into()),
                ),
            );
        }
    }

    #[inline(never)]
    pub fn define_constant_int_class_traits(
        self,
        namespace: Namespace<'gc>,
        items: &[(&'static str, i32)],
        activation: &mut Activation<'_, 'gc>,
    ) {
        for &(name, value) in items {
            self.define_class_trait(
                activation.context.gc_context,
                Trait::from_const(
                    QName::new(namespace, name),
                    Some(activation.avm2().multinames.int),
                    Some(value.into()),
                ),
            );
        }
    }

    #[inline(never)]
    pub fn define_builtin_instance_methods(
        self,
        mc: &Mutation<'gc>,
        namespace: Namespace<'gc>,
        items: &[(&'static str, NativeMethodImpl)],
    ) {
        for &(name, value) in items {
            self.define_instance_trait(
                mc,
                Trait::from_method(
                    QName::new(namespace, name),
                    Method::from_builtin(value, name, mc),
                ),
            );
        }
    }

    #[allow(clippy::type_complexity)]
    #[inline(never)]
    pub fn define_builtin_instance_methods_with_sig(
        self,
        mc: &Mutation<'gc>,
        namespace: Namespace<'gc>,
        items: Vec<(
            &'static str,
            NativeMethodImpl,
            Vec<ParamConfig<'gc>>,
            Option<Gc<'gc, Multiname<'gc>>>,
        )>,
    ) {
        for (name, value, params, return_type) in items {
            self.define_instance_trait(
                mc,
                Trait::from_method(
                    QName::new(namespace, name),
                    Method::from_builtin_and_params(value, name, params, return_type, false, mc),
                ),
            );
        }
    }

    #[inline(never)]
    pub fn define_builtin_class_methods(
        self,
        mc: &Mutation<'gc>,
        namespace: Namespace<'gc>,
        items: &[(&'static str, NativeMethodImpl)],
    ) {
        for &(name, value) in items {
            self.define_class_trait(
                mc,
                Trait::from_method(
                    QName::new(namespace, name),
                    Method::from_builtin(value, name, mc),
                ),
            );
        }
    }

    #[inline(never)]
    pub fn define_builtin_instance_properties(
        self,
        mc: &Mutation<'gc>,
        namespace: Namespace<'gc>,
        items: &[(
            &'static str,
            Option<NativeMethodImpl>,
            Option<NativeMethodImpl>,
        )],
    ) {
        for &(name, getter, setter) in items {
            if let Some(getter) = getter {
                self.define_instance_trait(
                    mc,
                    Trait::from_getter(
                        QName::new(namespace, name),
                        Method::from_builtin(getter, name, mc),
                    ),
                );
            }
            if let Some(setter) = setter {
                self.define_instance_trait(
                    mc,
                    Trait::from_setter(
                        QName::new(namespace, name),
                        Method::from_builtin(setter, name, mc),
                    ),
                );
            }
        }
    }

    #[inline(never)]
    pub fn define_constant_int_instance_traits(
        self,
        namespace: Namespace<'gc>,
        items: &[(&'static str, i32)],
        activation: &mut Activation<'_, 'gc>,
    ) {
        for &(name, value) in items {
            self.define_instance_trait(
                activation.context.gc_context,
                Trait::from_const(
                    QName::new(namespace, name),
                    Some(activation.avm2().multinames.int),
                    Some(value.into()),
                ),
            );
        }
    }

    /// Define a trait on the class.
    ///
    /// Class traits will be accessible as properties on the class object.
    pub fn define_class_trait(self, mc: &Mutation<'gc>, my_trait: Trait<'gc>) {
        self.c_class()
            .expect("Accessed class_traits on a c_class")
            .define_instance_trait(mc, my_trait);
    }

    /// Define a trait on instances of the class.
    ///
    /// Instance traits will be accessible as properties on instances of the
    /// class. They will not be accessible on the class prototype, and any
    /// properties defined on the prototype will be shadowed by these traits.
    pub fn define_instance_trait(self, mc: &Mutation<'gc>, my_trait: Trait<'gc>) {
        self.0.write(mc).traits.push(my_trait);
    }

    /// Return traits provided by this class.
    pub fn traits(&self) -> Ref<Vec<Trait<'gc>>> {
        Ref::map(self.0.read(), |c| &c.traits)
    }

    pub fn set_traits(&self, mc: &Mutation<'gc>, traits: Vec<Trait<'gc>>) {
        self.0.write(mc).traits = traits;
    }

    /// Get this class's instance allocator.
    ///
    /// If `None`, then you should use the instance allocator of the superclass
    /// or allocate as a `ScriptObject` if no such class exists.
    pub fn instance_allocator(self) -> Allocator {
        self.0.read().instance_allocator
    }

    /// Set this class's instance allocator.
    pub fn set_instance_allocator(self, mc: &Mutation<'gc>, alloc: AllocatorFn) {
        self.0.write(mc).instance_allocator = Allocator(alloc);
    }

    /// Get this class's instance initializer.
    pub fn instance_init(self) -> Method<'gc> {
        self.0.read().instance_init
    }

    /// Get this class's super() initializer.
    pub fn super_init(self) -> Method<'gc> {
        self.0.read().super_init
    }

    /// Set a super() initializer for this class.
    pub fn set_super_init(self, mc: &Mutation<'gc>, new_super_init: Method<'gc>) {
        self.0.write(mc).super_init = new_super_init;
    }

    /// Set a call handler for this class.
    pub fn set_call_handler(self, mc: &Mutation<'gc>, new_call_handler: Method<'gc>) {
        self.0.write(mc).call_handler = Some(new_call_handler);
    }

    /// Get this class's call handler.
    pub fn call_handler(self) -> Option<Method<'gc>> {
        self.0.read().call_handler
    }

    pub fn direct_interfaces(&self) -> Ref<Vec<Class<'gc>>> {
        Ref::map(self.0.read(), |c| &c.direct_interfaces)
    }

    pub fn all_interfaces(&self) -> Ref<Vec<Class<'gc>>> {
        Ref::map(self.0.read(), |c| &c.all_interfaces)
    }

    /// Determine if this class is sealed (no dynamic properties)
    pub fn is_sealed(self) -> bool {
        self.0.read().attributes.contains(ClassAttributes::SEALED)
    }

    /// Determine if this class is final (cannot be subclassed)
    pub fn is_final(self) -> bool {
        self.0.read().attributes.contains(ClassAttributes::FINAL)
    }

    /// Determine if this class is an interface
    pub fn is_interface(self) -> bool {
        self.0
            .read()
            .attributes
            .contains(ClassAttributes::INTERFACE)
    }

    /// Determine if this class is generic (can be specialized)
    pub fn is_generic(self) -> bool {
        self.0.read().attributes.contains(ClassAttributes::GENERIC)
    }

    pub fn c_class(self) -> Option<Class<'gc>> {
        if let ClassLink::LinkToClass(c_class) = self.0.read().linked_class {
            Some(c_class)
        } else {
            None
        }
    }

    pub fn set_c_class(self, mc: &Mutation<'gc>, c_class: Class<'gc>) {
        assert!(matches!(self.0.read().linked_class, ClassLink::Unlinked));

        self.0.write(mc).linked_class = ClassLink::LinkToClass(c_class);
    }

    pub fn is_c_class(self) -> bool {
        matches!(self.0.read().linked_class, ClassLink::LinkToInstance(_))
    }

    pub fn i_class(self) -> Option<Class<'gc>> {
        if let ClassLink::LinkToInstance(i_class) = self.0.read().linked_class {
            Some(i_class)
        } else {
            None
        }
    }

    pub fn set_i_class(self, mc: &Mutation<'gc>, i_class: Class<'gc>) {
        assert!(matches!(self.0.read().linked_class, ClassLink::Unlinked));

        self.0.write(mc).linked_class = ClassLink::LinkToInstance(i_class);
    }

    pub fn is_i_class(self) -> bool {
        matches!(self.0.read().linked_class, ClassLink::LinkToClass(_))
    }
}
