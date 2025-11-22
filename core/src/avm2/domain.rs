//! Application Domains

use std::cell::{Ref, RefMut};

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::error::{make_error_1065, make_error_1504, Error};
use crate::avm2::object::{ByteArrayObject, TObject};
use crate::avm2::property_map::PropertyMap;
use crate::avm2::script::Script;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Multiname, QName};
use crate::context::UpdateContext;
use crate::string::AvmString;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, OnceLock, RefLock};
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_wstr::WStr;

/// Represents a set of scripts and movies that share traits across different
/// script-global scopes.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct Domain<'gc>(Gc<'gc, DomainData<'gc>>);

/// Weak version of the Domain struct, used to mark children of a Domain when
/// the egui feature is enabled
#[allow(dead_code)]
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct DomainWeak<'gc>(GcWeak<'gc, DomainData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
struct DomainData<'gc> {
    cell: RefLock<DomainDataMut<'gc>>,

    /// The parent domain.
    parent: Option<Domain<'gc>>,

    /// The bytearray used for storing domain memory
    ///
    /// Note: While this property is optional, it is not recommended to set it
    /// to `None`. It is only optional to avoid an order-of-events problem in
    /// player globals setup (we need a global domain to put globals into, but
    /// that domain needs the bytearray global)
    domain_memory: Lock<Option<ByteArrayObject<'gc>>>,

    default_domain_memory: OnceLock<ByteArrayObject<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
struct DomainDataMut<'gc> {
    /// A list of all exported definitions and the script that exported them.
    defs: PropertyMap<'gc, Script<'gc>>,

    /// A map of all Clasess defined in this domain. Used by ClassObject
    /// to perform early interface resolution.
    classes: PropertyMap<'gc, Class<'gc>>,

    /// All children of this domain. This is intended exclusively for
    /// use with `debug_ui`
    #[cfg(feature = "egui")]
    children: Vec<DomainWeak<'gc>>,
}

const MIN_DOMAIN_MEMORY_LENGTH: usize = 1024;

impl<'gc> Domain<'gc> {
    fn cell(self) -> Ref<'gc, DomainDataMut<'gc>> {
        Gc::as_ref(self.0).cell.borrow()
    }

    fn cell_mut(self, mc: &Mutation<'gc>) -> RefMut<'gc, DomainDataMut<'gc>> {
        unlock!(Gc::write(mc, self.0), DomainData, cell).borrow_mut()
    }

    /// Create a new domain with no parent.
    ///
    /// This is intended exclusively for creating the player globals domain,
    /// and stage domain, which are created before ByteArray is available.
    ///
    /// Note: the global domain will be created without valid domain memory.
    /// You must initialize domain memory later on after the ByteArray class is
    /// instantiated but before user code runs.
    pub fn uninitialized_domain(mc: &Mutation<'gc>, parent: Option<Domain<'gc>>) -> Domain<'gc> {
        let this = Self(Gc::new(
            mc,
            DomainData {
                cell: RefLock::new(DomainDataMut {
                    defs: PropertyMap::new(),
                    classes: PropertyMap::new(),
                    #[cfg(feature = "egui")]
                    children: Vec::new(),
                }),
                parent,
                domain_memory: Default::default(),
                default_domain_memory: Default::default(),
            },
        ));

        #[cfg(feature = "egui")]
        if let Some(parent) = parent {
            parent
                .cell_mut(mc)
                .children
                .push(DomainWeak(Gc::downgrade(this.0)));
        }
        this
    }

    pub fn classes(&self) -> Ref<'_, PropertyMap<'gc, Class<'gc>>> {
        Ref::map(self.0.cell.borrow(), |r| &r.classes)
    }

    pub fn is_playerglobals_domain(&self, avm2: &Avm2<'gc>) -> bool {
        std::ptr::eq(Gc::as_ptr(avm2.playerglobals_domain.0), Gc::as_ptr(self.0))
    }

    #[cfg(feature = "egui")]
    pub fn children(self, mc: &Mutation<'gc>) -> Vec<Domain<'gc>> {
        // Take this opportunity to clean up dead children.
        let mut output = Vec::new();
        self.cell_mut(mc).children.retain(|child| {
            if let Some(child_cell) = GcWeak::upgrade(child.0, mc) {
                output.push(Domain(child_cell));
                true
            } else {
                false
            }
        });
        output
    }

    /// Create a new domain with a given parent.
    ///
    /// This function must not be called before the player globals have been
    /// fully allocated.
    pub fn movie_domain(context: &mut UpdateContext<'gc>, parent: Domain<'gc>) -> Domain<'gc> {
        let domain_memory = Self::create_default_domain_memory(context);
        let this = Self(Gc::new(
            context.gc(),
            DomainData {
                cell: RefLock::new(DomainDataMut {
                    defs: PropertyMap::new(),
                    classes: PropertyMap::new(),
                    #[cfg(feature = "egui")]
                    children: Vec::new(),
                }),
                parent: Some(parent),
                domain_memory: Lock::new(Some(domain_memory)),
                default_domain_memory: OnceLock::from(domain_memory),
            },
        ));

        #[cfg(feature = "egui")]
        {
            parent
                .cell_mut(context.gc())
                .children
                .push(DomainWeak(Gc::downgrade(this.0)));
        }

        this
    }

    /// Get the parent of this domain
    pub fn parent_domain(self) -> Option<Domain<'gc>> {
        self.0.parent
    }

    /// Determine if something has been defined within the current domain (including parents)
    pub fn has_definition(self, name: QName<'gc>) -> bool {
        if self.cell().defs.contains_key(name) {
            return true;
        }

        if let Some(parent) = self.0.parent {
            return parent.has_definition(name);
        }

        false
    }

    /// Determine if a class has been defined within the current domain (including parents)
    pub fn has_class(self, name: QName<'gc>) -> bool {
        if self.cell().classes.contains_key(name) {
            return true;
        }

        if let Some(parent) = self.0.parent {
            return parent.has_class(name);
        }

        false
    }

    /// Resolve a Multiname and return the script that provided it.
    ///
    /// If a name does not exist or cannot be resolved, no script or name will
    /// be returned.
    pub fn get_defining_script(
        self,
        multiname: &Multiname<'gc>,
    ) -> Option<(QName<'gc>, Script<'gc>)> {
        if let Some(name) = multiname.local_name() {
            if let Some((ns, script)) = self.cell().defs.get_with_ns_for_multiname(multiname) {
                let qname = QName::new(ns, name);
                return Some((qname, *script));
            }
        }

        if let Some(parent) = self.0.parent {
            return parent.get_defining_script(multiname);
        }

        None
    }

    fn get_class_inner(self, multiname: &Multiname<'gc>) -> Option<Class<'gc>> {
        if let Some(class) = self.cell().classes.get_for_multiname(multiname) {
            return Some(*class);
        }

        if let Some(parent) = self.0.parent {
            return parent.get_class_inner(multiname);
        }

        None
    }

    pub fn get_class(
        self,
        context: &mut UpdateContext<'gc>,
        multiname: &Multiname<'gc>,
    ) -> Option<Class<'gc>> {
        let class = self.get_class_inner(multiname);

        if let Some(class) = class {
            if let Some(param) = multiname.param() {
                if let Some(param) = param {
                    if let Some(resolved_param) = self.get_class(context, &param) {
                        return Some(Class::with_type_param(context, class, Some(resolved_param)));
                    }
                    return None;
                } else {
                    return Some(Class::with_type_param(context, class, None));
                }
            }
        }
        class
    }

    /// Resolve a Multiname and return the script that provided it.
    ///
    /// If a name does not exist or cannot be resolved, an error will be thrown.
    pub fn find_defining_script(
        self,
        activation: &mut Activation<'_, 'gc>,
        multiname: &Multiname<'gc>,
    ) -> Result<(QName<'gc>, Script<'gc>), Error<'gc>> {
        match self.get_defining_script(multiname) {
            Some(val) => Ok(val),
            None => Err(make_error_1065(activation, multiname)),
        }
    }

    /// Retrieve a value from this domain.
    pub fn get_defined_value(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: QName<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let (name, script) = self.find_defining_script(activation, &name.into())?;
        let globals = script.globals(activation.context)?;

        Value::from(globals).get_property(&name.into(), activation)
    }

    /// Retrieve a value from this domain, with special handling for 'Vector.<SomeType>'.
    /// This is used by `getQualifiedClassName, ApplicationDomain.getDefinition, and ApplicationDomain.hasDefinition`.
    pub fn get_defined_value_handling_vector(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // Special-case lookups of `Vector.<SomeType>` - these get internally converted
        // to a lookup of `Vector,` a lookup of `SomeType`, and `vector_class.apply(some_type_class)`
        if let Some(type_name) = vector_parameter_from_name(activation.gc(), name) {
            let vector_class = activation.avm2().classes().generic_vector;
            let parameter_value = self.get_defined_value_handling_vector(activation, type_name)?;

            return vector_class
                .apply(activation, &[parameter_value])
                .map(|obj| obj.into());
        }

        // If we're not hitting the special-case, just call `get_defined_value`

        let name = QName::from_qualified_name(name, activation.context);
        self.get_defined_value(activation, name)
    }

    pub fn has_defined_value_handling_vector(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
    ) -> bool {
        if let Some(type_name) = vector_parameter_from_name(activation.gc(), name) {
            // avmplus just checks if the type parameter exists, so we do the same
            self.has_defined_value_handling_vector(activation, type_name)
        } else {
            let name = QName::from_qualified_name(name, activation.context);

            self.get_defining_script(&name.into()).is_some()
        }
    }

    pub fn get_defined_names(self) -> Vec<QName<'gc>> {
        self.cell()
            .defs
            .iter()
            .map(|(name, namespace, _)| QName::new(namespace, name))
            .collect()
    }

    /// Export a definition from a script into the current application domain.
    ///
    /// This does nothing if the definition already exists in this domain or a parent.
    pub fn export_definition(self, name: QName<'gc>, script: Script<'gc>, mc: &Mutation<'gc>) {
        if self.has_definition(name) {
            return;
        }

        self.cell_mut(mc).defs.insert(name, script);
    }

    /// Export a class into the current application domain.
    ///
    /// This does nothing if the definition already exists in this domain or a parent.
    pub fn export_class(self, export_name: QName<'gc>, class: Class<'gc>, mc: &Mutation<'gc>) {
        if self.has_class(export_name) {
            return;
        }
        self.cell_mut(mc).classes.insert(export_name, class);
    }

    pub fn defs(&self) -> Ref<'_, PropertyMap<'gc, Script<'gc>>> {
        Ref::map(self.cell(), |this| &this.defs)
    }

    pub fn is_default_domain_memory(&self) -> bool {
        let domain_memory_ptr = self.domain_memory().as_ptr();
        let default_domain_memory_ptr = self
            .0
            .default_domain_memory
            .get()
            .expect("Missing default domain memory")
            .as_ptr();
        std::ptr::eq(domain_memory_ptr, default_domain_memory_ptr)
    }

    pub fn domain_memory(self) -> ByteArrayObject<'gc> {
        self.0
            .domain_memory
            .get()
            .expect("Domain must have valid memory at all times")
    }

    pub fn set_domain_memory(
        &self,
        activation: &mut Activation<'_, 'gc>,
        domain_memory: Option<ByteArrayObject<'gc>>,
    ) -> Result<(), Error<'gc>> {
        let memory = if let Some(domain_memory) = domain_memory {
            if domain_memory.storage().len() < MIN_DOMAIN_MEMORY_LENGTH {
                return Err(make_error_1504(activation));
            }
            domain_memory
        } else {
            let memory = self.0.default_domain_memory.get();
            *memory.expect("Default domain memory not initialized")
        };
        let write = Gc::write(activation.gc(), self.0);
        unlock!(write, DomainData, domain_memory).set(Some(memory));
        Ok(())
    }

    fn create_default_domain_memory(context: &mut UpdateContext<'gc>) -> ByteArrayObject<'gc> {
        let initial_data = vec![0; MIN_DOMAIN_MEMORY_LENGTH];
        let storage = ByteArrayStorage::from_vec(context, initial_data);
        ByteArrayObject::from_storage(context, storage)
    }

    /// Allocate the default domain memory for this domain, if it does not
    /// already exist.
    ///
    /// This function is only necessary to be called for domains created via
    /// `global_domain`. It will panic on already fully-initialized domains.
    pub fn init_default_domain_memory(self, context: &mut UpdateContext<'gc>) {
        let memory = Self::create_default_domain_memory(context);

        let write = Gc::write(context.gc(), self.0);
        match unlock!(write, DomainData, default_domain_memory).set(memory) {
            Ok(_) => unlock!(write, DomainData, domain_memory).set(Some(memory)),
            Err(_) => panic!("Already initialized domain memory!"),
        };
    }

    pub fn as_ptr(self) -> *const DomainPtr {
        Gc::as_ptr(self.0) as _
    }
}

/// Given a class name such as `Vector.<int>`, returns the Vector type
/// parameter (`int`), or `None` if the class name does not represent a
/// parametrized Vector class (e.g. `flash.display::MovieClip`).
fn vector_parameter_from_name<'gc>(
    mc: &Mutation<'gc>,
    name: AvmString<'gc>,
) -> Option<AvmString<'gc>> {
    if (name.starts_with(WStr::from_units(b"__AS3__.vec::Vector.<"))
        || name.starts_with(WStr::from_units(b"Vector.<")))
        && name.ends_with(WStr::from_units(b">"))
    {
        let start = name.find(WStr::from_units(b".<")).unwrap();

        Some(AvmString::new(mc, &name[(start + 2)..(name.len() - 1)]))
    } else {
        None
    }
}

pub enum DomainPtr {}

impl PartialEq for Domain<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(Gc::as_ptr(self.0), Gc::as_ptr(other.0))
    }
}

impl Eq for Domain<'_> {}
