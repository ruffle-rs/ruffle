use crate::avm2::activation::Activation;
use crate::avm2::metadata::Metadata;
use crate::avm2::method::Method;
use crate::avm2::object::{ClassObject, FunctionObject};
use crate::avm2::property::{Property, PropertyClass};
use crate::avm2::property_map::PropertyMap;
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::value::Value;
use crate::avm2::{Class, Error, Multiname, Namespace, QName};
use crate::context::UpdateContext;
use crate::string::{AvmString, StringContext};
use gc_arena::barrier::{field, unlock};
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use std::collections::HashMap;

#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct VTable<'gc>(Gc<'gc, VTableData<'gc>>);

#[derive(Collect, Default)]
#[collect(no_drop)]
struct VTableData<'gc> {
    scope: Option<ScopeChain<'gc>>,

    protected_namespace: Option<Namespace<'gc>>,

    resolved_traits: PropertyMap<'gc, Property>,

    /// Use hashmaps for the metadata tables because metadata will rarely be present on traits
    slot_metadata_table: HashMap<u32, Box<[Metadata<'gc>]>>,

    disp_metadata_table: HashMap<u32, Box<[Metadata<'gc>]>>,

    /// Stores the `PropertyClass` for each slot,
    /// indexed by `slot_id`
    slot_classes: Box<[Lock<PropertyClass<'gc>>]>,

    /// method_table is indexed by `disp_id`
    method_table: Box<[ClassBoundMethod<'gc>]>,

    default_slots: Box<[Option<Value<'gc>>]>,
}

impl PartialEq for VTable<'_> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(self.0, other.0)
    }
}

// TODO: it might make more sense to just bind the Method to the VTable (and this its class and scope) directly
// would also be nice to somehow remove the Option-ness from `defining_class` and `scope` fields for this
// to be more intuitive and cheaper
#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct ClassBoundMethod<'gc> {
    pub class: Class<'gc>,
    pub super_class_obj: Option<ClassObject<'gc>>,
    scope: Lock<Option<ScopeChain<'gc>>>,
    pub method: Method<'gc>,
}

impl<'gc> ClassBoundMethod<'gc> {
    pub fn scope(&self) -> ScopeChain<'gc> {
        self.scope.get().expect("Scope should exists here")
    }
}

impl<'gc> VTable<'gc> {
    pub fn empty(mc: &Mutation<'gc>) -> Self {
        VTable(Gc::new(mc, VTableData::default()))
    }

    /// Builds a new vtable by calculating the flattened list of instance traits
    /// that this class maintains.
    pub fn new(
        defining_class_def: Class<'gc>,
        super_class_obj: Option<ClassObject<'gc>>,
        scope: Option<ScopeChain<'gc>>,
        superclass_vtable: Option<Self>,
        mc: &Mutation<'gc>,
    ) -> Self {
        let this = Self::init_vtable(
            defining_class_def,
            super_class_obj,
            scope,
            superclass_vtable,
        );
        VTable(Gc::new(mc, this))
    }

    /// Like `VTable::new`, but also copies properties from the defining class' interfaces.
    pub fn new_with_interface_properties(
        defining_class_def: Class<'gc>,
        super_class_obj: Option<ClassObject<'gc>>,
        scope: Option<ScopeChain<'gc>>,
        superclass_vtable: Option<Self>,
        context: &UpdateContext<'gc>,
    ) -> Self {
        let mut this = Self::init_vtable(
            defining_class_def,
            super_class_obj,
            scope,
            superclass_vtable,
        );
        Self::copy_interface_properties(&mut this, defining_class_def, context);
        VTable(Gc::new(context.gc(), this))
    }

    pub fn resolved_traits(self) -> &'gc PropertyMap<'gc, Property> {
        &Gc::as_ref(self.0).resolved_traits
    }

    pub fn get_metadata_for_slot(self, slot_id: u32) -> Option<&'gc [Metadata<'gc>]> {
        Gc::as_ref(self.0)
            .slot_metadata_table
            .get(&slot_id)
            .map(|v| &**v)
    }

    pub fn get_metadata_for_disp(self, disp_id: u32) -> Option<&'gc [Metadata<'gc>]> {
        Gc::as_ref(self.0)
            .disp_metadata_table
            .get(&disp_id)
            .map(|v| &**v)
    }

    pub fn slot_class_name(self, context: &mut StringContext<'gc>, slot_id: u32) -> AvmString<'gc> {
        self.0
            .slot_classes
            .get(slot_id as usize)
            .expect("Invalid slot ID")
            .get()
            .get_name(context)
    }

    pub fn get_trait(self, name: &Multiname<'gc>) -> Option<Property> {
        if name.is_attribute() {
            return None;
        }

        self.resolved_traits().get_for_multiname(name).cloned()
    }

    pub fn get_trait_with_ns(self, name: &Multiname<'gc>) -> Option<(Namespace<'gc>, Property)> {
        if name.is_attribute() {
            return None;
        }

        self.resolved_traits()
            .get_with_ns_for_multiname(name)
            .map(|(ns, p)| (ns, *p))
    }

    /// Coerces `value` to the type of the slot with id `slot_id`
    pub fn coerce_trait_value(
        self,
        slot_id: u32,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let slot_id = slot_id as usize;
        let mut slot_class = self.0.slot_classes[slot_id].get();

        let (value, changed) = slot_class.coerce(activation, value)?;

        // Calling coerce modified `PropertyClass` to cache the class lookup,
        // so store the new value back in the vtable.
        if changed {
            let write = Gc::write(activation.gc(), self.0);
            let slots = field!(write, VTableData, slot_classes).as_deref();
            slots[slot_id].unlock().set(slot_class);
        }
        Ok(value)
    }

    pub fn has_trait(self, name: &Multiname<'gc>) -> bool {
        self.resolved_traits().get_for_multiname(name).is_some()
    }

    pub fn get_method(self, disp_id: u32) -> Option<Method<'gc>> {
        self.0
            .method_table
            .get(disp_id as usize)
            .cloned()
            .map(|x| x.method)
    }

    pub fn get_full_method(self, disp_id: u32) -> Option<&'gc ClassBoundMethod<'gc>> {
        Gc::as_ref(self.0).method_table.get(disp_id as usize)
    }

    pub fn default_slots(&self) -> &[Option<Value<'gc>>] {
        &self.0.default_slots
    }

    pub fn slot_class(self, slot_id: u32) -> Option<PropertyClass<'gc>> {
        self.0.slot_classes.get(slot_id as usize).map(Lock::get)
    }

    pub fn set_slot_class(self, mc: &Mutation<'gc>, slot_id: u32, value: PropertyClass<'gc>) {
        let slots = field!(Gc::write(mc, self.0), VTableData, slot_classes).as_deref();
        slots[slot_id as usize].unlock().set(value);
    }

    pub fn replace_scopes_with(self, mc: &Mutation<'gc>, new_scope: ScopeChain<'gc>) {
        let methods = field!(Gc::write(mc, self.0), VTableData, method_table).as_deref();
        for i in 0..methods.len() {
            unlock!(&methods[i], ClassBoundMethod, scope).set(Some(new_scope));
        }
    }

    fn init_vtable(
        defining_class_def: Class<'gc>,
        super_class_obj: Option<ClassObject<'gc>>,
        scope: Option<ScopeChain<'gc>>,
        superclass_vtable: Option<Self>,
    ) -> VTableData<'gc> {
        // Let's talk about slot_ids and disp_ids.
        // Specification is one thing, but reality is another.

        // disp_id in FP:
        // It appears that FP completely ignores it and assigns values on its own.
        // Any attempt to use `callmethod` opcode to observe the disp_id fails
        // with VerifyError.
        //
        // disp_id in Ruffle:
        // Let's just do the same. We could go the easy way and always-increment,
        // but reusing same disp_id for overriding virtual methods is a nice idea,
        // both for space savings and lets us still use call_method() internally
        // for virtual dispatch when it's safe to do so.
        // And let's error on every `callmethod` opcode and hope it never ever happens.

        // slot_id in FP:
        // It's a bit more complex here.
        //
        // If class and superclass come from the same ABC (constant pool) or superclass has no slots,
        // then slot_ids are respected; conflicts result in VerifyError.
        // You are only allowed to call `getslot` on the object if calling method,
        // callee's class and all subclasses come from the same ABC (constant pool).
        // (or class has no slots, but then `getslot` fails verification anyway as it's out-of-range)
        //
        // If class and superclass come from different ABC (constant pool) and superclass has slots,
        // then subclass's slot_ids are ignored and assigned automatically.
        // ignored, as in: even if trait's slot_id conflicts, it's not verified at all.
        //
        // In practice, this all means that compiler is allowed to use `getslot`
        // or affect/observe slots in any other way only on classes
        // it had 100% control over slot layout of, on the entire class hierarchy.
        //
        // (*in particular, trying to use `getslot` in script initializer
        //   on class defined in same script also throws VerifyError;
        //   not sure why it's treated as "different constant pool")

        // slot_id in Ruffle:
        // Currently we don't really have ability to "compare abc between
        // methods/activations/traits/etc", so let's do something simpler.
        // We try to respect slot_id whenever possible, but if a conflict arises,
        // let's just auto-assign a higher one.
        // The logic is that if we ever see a conflict, either it's a class that
        // wouldn't have passed verification in the first place, or trying to observe
        // such slot with `getslot` wouldn't have passed verification in the first place.
        // So such SWFs shouldn't be encountered in the wild.
        //
        // Worst-case is that someone can hand-craft such an SWF specifically for Ruffle
        // and be able to access private class members with `getslot/setslot,
        // so long-term it's still something we should verify.
        // (and it's far from the only verification check we lack anyway)

        let mut resolved_traits = PropertyMap::new();
        let mut slot_metadata_table = HashMap::new();
        let mut disp_metadata_table = HashMap::new();
        let mut method_table = Vec::new();
        let mut default_slots = Vec::new();
        let mut slot_classes = Vec::new();

        if let Some(superclass_vtable) = superclass_vtable {
            resolved_traits = superclass_vtable.resolved_traits().clone();
            slot_metadata_table = superclass_vtable.0.slot_metadata_table.clone();
            disp_metadata_table = superclass_vtable.0.disp_metadata_table.clone();
            slot_classes.extend_from_slice(&superclass_vtable.0.slot_classes);
            method_table.extend_from_slice(&superclass_vtable.0.method_table);
            default_slots.extend_from_slice(&superclass_vtable.0.default_slots);

            if let Some(protected_namespace) = defining_class_def.protected_namespace() {
                if let Some(super_protected_namespace) = superclass_vtable.0.protected_namespace {
                    // Copy all protected traits from superclass
                    // but with this class's protected namespace
                    for (local_name, ns, prop) in superclass_vtable.resolved_traits().iter() {
                        if ns.exact_version_match(super_protected_namespace) {
                            let new_name = QName::new(protected_namespace, local_name);
                            resolved_traits.insert(new_name, *prop);
                        }
                    }
                }
            }
        }

        for trait_data in defining_class_def.traits() {
            match trait_data.kind() {
                TraitKind::Method { method, .. } => {
                    let entry = ClassBoundMethod {
                        class: defining_class_def,
                        super_class_obj,
                        scope: Lock::new(scope),
                        method: *method,
                    };
                    match resolved_traits.get(trait_data.name()) {
                        Some(Property::Method { disp_id, .. }) => {
                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(*disp_id, metadata);
                            }

                            method_table[*disp_id as usize] = entry;
                        }
                        // note: ideally overwriting other property types
                        // should be a VerifyError
                        _ => {
                            let disp_id = method_table.len() as u32;
                            method_table.push(entry);
                            resolved_traits
                                .insert(trait_data.name(), Property::new_method(disp_id));

                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(disp_id, metadata);
                            }
                        }
                    }
                }
                TraitKind::Getter { method, .. } => {
                    let entry = ClassBoundMethod {
                        class: defining_class_def,
                        super_class_obj,
                        scope: Lock::new(scope),
                        method: *method,
                    };
                    match resolved_traits.get_mut(trait_data.name()) {
                        Some(Property::Virtual {
                            get: Some(disp_id), ..
                        }) => {
                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(*disp_id, metadata);
                            }

                            method_table[*disp_id as usize] = entry;
                        }
                        Some(Property::Virtual { get, .. }) => {
                            let disp_id = method_table.len() as u32;
                            *get = Some(disp_id);
                            method_table.push(entry);

                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(disp_id, metadata);
                            }
                        }
                        _ => {
                            let disp_id = method_table.len() as u32;
                            method_table.push(entry);
                            resolved_traits
                                .insert(trait_data.name(), Property::new_getter(disp_id));

                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(disp_id, metadata);
                            }
                        }
                    }
                }
                TraitKind::Setter { method, .. } => {
                    let entry = ClassBoundMethod {
                        class: defining_class_def,
                        super_class_obj,
                        scope: Lock::new(scope),
                        method: *method,
                    };
                    match resolved_traits.get_mut(trait_data.name()) {
                        Some(Property::Virtual {
                            set: Some(disp_id), ..
                        }) => {
                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(*disp_id, metadata);
                            }

                            method_table[*disp_id as usize] = entry;
                        }
                        Some(Property::Virtual { set, .. }) => {
                            let disp_id = method_table.len() as u32;
                            method_table.push(entry);
                            *set = Some(disp_id);

                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(disp_id, metadata);
                            }
                        }
                        _ => {
                            let disp_id = method_table.len() as u32;
                            method_table.push(entry);
                            resolved_traits
                                .insert(trait_data.name(), Property::new_setter(disp_id));

                            if let Some(metadata) = trait_data.metadata() {
                                disp_metadata_table.insert(disp_id, metadata);
                            }
                        }
                    }
                }
                TraitKind::Slot { slot_id, .. }
                | TraitKind::Const { slot_id, .. }
                | TraitKind::Class { slot_id, .. } => {
                    let slot_id = *slot_id;

                    let value = trait_to_default_value(trait_data);
                    let value = Some(value);

                    let new_slot_id = if slot_id == 0 {
                        default_slots.push(value);
                        default_slots.len() as u32 - 1
                    } else {
                        // it's non-zero, so let's turn it from 1-based to 0-based.
                        let slot_id = slot_id - 1;
                        if let Some(Some(_)) = default_slots.get(slot_id as usize) {
                            // slot_id conflict
                            default_slots.push(value);
                            default_slots.len() as u32 - 1
                        } else {
                            if slot_id as usize >= default_slots.len() {
                                default_slots.resize_with(slot_id as usize + 1, Default::default);
                            }
                            default_slots[slot_id as usize] = value;
                            slot_id
                        }
                    };

                    if new_slot_id as usize >= slot_classes.len() {
                        // We will overwrite `PropertyClass::Any` when we process the slots
                        // with the ids that we just skipped over.
                        slot_classes
                            .resize(new_slot_id as usize + 1, Lock::new(PropertyClass::Any));
                    }

                    let (new_prop, new_class) = match trait_data.kind() {
                        TraitKind::Slot {
                            type_name, domain, ..
                        } => (
                            Property::new_slot(new_slot_id),
                            PropertyClass::name(*type_name, *domain),
                        ),
                        TraitKind::Const {
                            type_name, domain, ..
                        } => (
                            Property::new_const_slot(new_slot_id),
                            PropertyClass::name(*type_name, *domain),
                        ),
                        TraitKind::Class { class, .. } => (
                            Property::new_const_slot(new_slot_id),
                            PropertyClass::Class(
                                class.c_class().expect("Trait should hold an i_class"),
                            ),
                        ),
                        _ => unreachable!(),
                    };

                    resolved_traits.insert(trait_data.name(), new_prop);

                    if let Some(metadata) = trait_data.metadata() {
                        slot_metadata_table.insert(new_slot_id, metadata);
                    }

                    slot_classes[new_slot_id as usize] = Lock::new(new_class);
                }
            }
        }

        VTableData {
            scope,
            protected_namespace: defining_class_def.protected_namespace(),
            resolved_traits,
            slot_metadata_table,
            disp_metadata_table,
            method_table: method_table.into_boxed_slice(),
            default_slots: default_slots.into_boxed_slice(),
            slot_classes: slot_classes.into_boxed_slice(),
        }
    }

    fn copy_interface_properties(
        this: &mut VTableData<'gc>,
        class: Class<'gc>,
        context: &UpdateContext<'gc>,
    ) {
        // FIXME - we should only be copying properties for newly-implemented
        // interfaces (i.e. those that were not already implemented by the superclass)
        // Otherwise, our behavior diverges from Flash Player in certain cases.
        // See the ignored test 'tests/tests/swfs/avm2/weird_superinterface_properties/'
        let internal_ns = context.avm2.namespaces.public_vm_internal();
        for interface in class.all_interfaces() {
            for interface_trait in interface.traits() {
                let interface_name = interface_trait.name();
                if !interface_name.namespace().is_public() {
                    let public_name = QName::new(internal_ns, interface_name.local_name());
                    if let Some(prop) = this.resolved_traits.get(public_name).copied() {
                        this.resolved_traits.insert(interface_name, prop);
                    }
                }
            }
        }
    }

    /// Retrieve a bound instance method suitable for use as a value.
    ///
    /// This returns the bound method object itself, as well as its dispatch
    /// ID. You will need the additional properties in order to install the
    /// method into your object.
    ///
    /// You should only call this method once per receiver/name pair, and cache
    /// the result. Otherwise, code that relies on bound methods having stable
    /// object identitities (e.g. `EventDispatcher.removeEventListener`) will
    /// fail.
    ///
    /// It is the caller's responsibility to ensure that the `receiver` passed
    /// to this method is not Value::Null or Value::Undefined.
    pub fn make_bound_method(
        self,
        activation: &mut Activation<'_, 'gc>,
        receiver: Value<'gc>,
        disp_id: u32,
    ) -> Option<FunctionObject<'gc>> {
        self.get_full_method(disp_id)
            .map(|method| Self::bind_method(activation, receiver, method))
    }

    /// Bind an instance method to a receiver, allowing it to be used as a value. See `VTable::make_bound_method`
    ///
    /// It is the caller's responsibility to ensure that the `receiver` passed
    /// to this method is not Value::Null or Value::Undefined.
    pub fn bind_method(
        activation: &mut Activation<'_, 'gc>,
        receiver: Value<'gc>,
        method: &ClassBoundMethod<'gc>,
    ) -> FunctionObject<'gc> {
        FunctionObject::from_method(
            activation,
            method.method,
            method.scope(),
            Some(receiver),
            method.super_class_obj,
            Some(method.class),
        )
    }

    pub fn public_properties(self) -> impl Iterator<Item = (AvmString<'gc>, Property)> {
        self.resolved_traits()
            .iter()
            .filter(|(_, ns, _)| ns.is_public())
            .map(|(name, _, prop)| (name, *prop))
    }
}

fn trait_to_default_value<'gc>(trait_data: &Trait<'gc>) -> Value<'gc> {
    match trait_data.kind() {
        TraitKind::Slot { default_value, .. } => *default_value,
        TraitKind::Const { default_value, .. } => *default_value,
        TraitKind::Class { .. } => Value::Null,
        _ => unreachable!(),
    }
}
