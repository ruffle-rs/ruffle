use crate::avm2::activation::Activation;
use crate::avm2::metadata::Metadata;
use crate::avm2::method::Method;
use crate::avm2::object::{ClassObject, FunctionObject, Object};
use crate::avm2::property::{Property, PropertyClass};
use crate::avm2::property_map::PropertyMap;
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::value::Value;
use crate::avm2::{Class, Error, Multiname, Namespace, QName};
use crate::context::UpdateContext;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, Mutation};
use std::cell::Ref;
use std::collections::HashMap;
use std::ops::DerefMut;

#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct VTable<'gc>(GcCell<'gc, VTableData<'gc>>);

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct VTableData<'gc> {
    scope: Option<ScopeChain<'gc>>,

    protected_namespace: Option<Namespace<'gc>>,

    resolved_traits: PropertyMap<'gc, Property>,

    /// Use hashmaps for the metadata tables because metadata will rarely be present on traits
    slot_metadata_table: HashMap<u32, Box<[Metadata<'gc>]>>,

    disp_metadata_table: HashMap<u32, Box<[Metadata<'gc>]>>,

    /// Stores the `PropertyClass` for each slot,
    /// indexed by `slot_id`
    slot_classes: Vec<PropertyClass<'gc>>,

    /// method_table is indexed by `disp_id`
    method_table: Vec<ClassBoundMethod<'gc>>,

    default_slots: Vec<Option<Value<'gc>>>,
}

impl PartialEq for VTable<'_> {
    fn eq(&self, other: &Self) -> bool {
        GcCell::ptr_eq(self.0, other.0)
    }
}

// TODO: it might make more sense to just bind the Method to the VTable (and this its class and scope) directly
// would also be nice to somehow remove the Option-ness from `defining_class` and `scope` fields for this
// to be more intuitive and cheaper
#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct ClassBoundMethod<'gc> {
    pub class: Class<'gc>,
    pub class_obj: Option<ClassObject<'gc>>,
    pub scope: Option<ScopeChain<'gc>>,
    pub method: Method<'gc>,
}

impl<'gc> VTable<'gc> {
    pub fn empty(mc: &Mutation<'gc>) -> Self {
        VTable(GcCell::new(
            mc,
            VTableData {
                scope: None,
                protected_namespace: None,
                resolved_traits: PropertyMap::new(),
                slot_metadata_table: HashMap::new(),
                disp_metadata_table: HashMap::new(),
                slot_classes: vec![],
                method_table: vec![],
                default_slots: vec![],
            },
        ))
    }

    /// A special case for newcatch. A single variable (q)name that maps to slot 0.
    pub fn newcatch(mc: &Mutation<'gc>, vname: &QName<'gc>) -> Self {
        let mut rt = PropertyMap::new();

        rt.insert(*vname, Property::Slot { slot_id: 0 });

        let vt = VTable(GcCell::new(
            mc,
            VTableData {
                scope: None,
                protected_namespace: None,
                resolved_traits: rt,
                slot_metadata_table: HashMap::new(),
                disp_metadata_table: HashMap::new(),
                method_table: vec![],
                default_slots: vec![None],
                slot_classes: vec![PropertyClass::Any],
            },
        ));

        vt
    }

    pub fn resolved_traits(&self) -> Ref<'_, PropertyMap<'gc, Property>> {
        Ref::map(self.0.read(), |v| &v.resolved_traits)
    }

    pub fn get_metadata_for_slot(&self, slot_id: &u32) -> Option<Box<[Metadata<'gc>]>> {
        self.0.read().slot_metadata_table.get(slot_id).cloned()
    }

    pub fn get_metadata_for_disp(&self, disp_id: &u32) -> Option<Box<[Metadata<'gc>]>> {
        self.0.read().disp_metadata_table.get(disp_id).cloned()
    }

    pub fn slot_class_name(
        &self,
        slot_id: u32,
        mc: &Mutation<'gc>,
    ) -> Result<Multiname<'gc>, Error<'gc>> {
        self.0
            .read()
            .slot_classes
            .get(slot_id as usize)
            .ok_or_else(|| "Invalid slot ID".into())
            .map(|c| c.get_name(mc))
    }

    pub fn get_trait(self, name: &Multiname<'gc>) -> Option<Property> {
        if name.is_attribute() {
            return None;
        }

        self.0
            .read()
            .resolved_traits
            .get_for_multiname(name)
            .cloned()
    }

    pub fn get_trait_with_ns(self, name: &Multiname<'gc>) -> Option<(Namespace<'gc>, Property)> {
        if name.is_attribute() {
            return None;
        }

        self.0
            .read()
            .resolved_traits
            .get_with_ns_for_multiname(name)
            .map(|(ns, p)| (ns, *p))
    }

    /// Coerces `value` to the type of the slot with id `slot_id`
    pub fn coerce_trait_value(
        &self,
        slot_id: u32,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // Drop the `write()` guard, as 'slot_class.coerce' may need to access this vtable.
        let mut slot_class = { self.0.read().slot_classes[slot_id as usize] };

        let (value, changed) = slot_class.coerce(activation, value)?;

        // Calling coerce modified `PropertyClass` to cache the class lookup,
        // so store the new value back in the vtable.
        if changed {
            self.0.write(activation.context.gc_context).slot_classes[slot_id as usize] = slot_class;
        }
        Ok(value)
    }

    pub fn has_trait(self, name: &Multiname<'gc>) -> bool {
        self.0
            .read()
            .resolved_traits
            .get_for_multiname(name)
            .is_some()
    }

    pub fn get_method(self, disp_id: u32) -> Option<Method<'gc>> {
        self.0
            .read()
            .method_table
            .get(disp_id as usize)
            .cloned()
            .map(|x| x.method)
    }

    pub fn get_full_method(self, disp_id: u32) -> Option<ClassBoundMethod<'gc>> {
        self.0.read().method_table.get(disp_id as usize).cloned()
    }

    pub fn default_slots(&self) -> Ref<Vec<Option<Value<'gc>>>> {
        Ref::map(self.0.read(), |v| &v.default_slots)
    }

    pub fn slot_classes(&self) -> Ref<Vec<PropertyClass<'gc>>> {
        Ref::map(self.0.read(), |v| &v.slot_classes)
    }

    pub fn set_slot_class(&self, mc: &Mutation<'gc>, index: usize, value: PropertyClass<'gc>) {
        self.0.write(mc).slot_classes[index] = value;
    }

    /// Calculate the flattened list of instance traits that this class
    /// maintains.
    ///
    /// This should be run during the class finalization step, before instances
    /// are linked (as instances will further add traits to the list).
    #[allow(clippy::if_same_then_else)]
    pub fn init_vtable(
        self,
        defining_class_def: Class<'gc>,
        defining_class: Option<ClassObject<'gc>>,
        traits: &[Trait<'gc>],
        scope: Option<ScopeChain<'gc>>,
        superclass_vtable: Option<Self>,
        context: &mut UpdateContext<'_, 'gc>,
    ) {
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

        let mut write = self.0.write(context.gc_context);
        let write = write.deref_mut();

        write.scope = scope;

        write.protected_namespace = defining_class_def.protected_namespace();

        if let Some(superclass_vtable) = superclass_vtable {
            write.resolved_traits = superclass_vtable.0.read().resolved_traits.clone();
            write.slot_metadata_table = superclass_vtable.0.read().slot_metadata_table.clone();
            write.disp_metadata_table = superclass_vtable.0.read().disp_metadata_table.clone();
            write.slot_classes = superclass_vtable.0.read().slot_classes.clone();
            write.method_table = superclass_vtable.0.read().method_table.clone();
            write.default_slots = superclass_vtable.0.read().default_slots.clone();

            if let Some(protected_namespace) = write.protected_namespace {
                if let Some(super_protected_namespace) =
                    superclass_vtable.0.read().protected_namespace
                {
                    // Copy all protected traits from superclass
                    // but with this class's protected namespace
                    for (local_name, ns, prop) in superclass_vtable.0.read().resolved_traits.iter()
                    {
                        if ns.exact_version_match(super_protected_namespace) {
                            let new_name = QName::new(protected_namespace, local_name);
                            write.resolved_traits.insert(new_name, *prop);
                        }
                    }
                }
            }
        }

        let (
            resolved_traits,
            slot_metadata_table,
            disp_metadata_table,
            method_table,
            default_slots,
            slot_classes,
        ) = (
            &mut write.resolved_traits,
            &mut write.slot_metadata_table,
            &mut write.disp_metadata_table,
            &mut write.method_table,
            &mut write.default_slots,
            &mut write.slot_classes,
        );

        for trait_data in traits {
            match trait_data.kind() {
                TraitKind::Method { method, .. } => {
                    let entry = ClassBoundMethod {
                        class: defining_class_def,
                        class_obj: defining_class,
                        scope,
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
                        class_obj: defining_class,
                        scope,
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
                        class_obj: defining_class,
                        scope,
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
                        slot_classes.resize(new_slot_id as usize + 1, PropertyClass::Any);
                    }

                    let (new_prop, new_class) = match trait_data.kind() {
                        TraitKind::Slot {
                            type_name, unit, ..
                        } => (
                            Property::new_slot(new_slot_id),
                            PropertyClass::name(context.gc_context, type_name.clone(), *unit),
                        ),
                        TraitKind::Const {
                            type_name, unit, ..
                        } => (
                            Property::new_const_slot(new_slot_id),
                            PropertyClass::name(context.gc_context, type_name.clone(), *unit),
                        ),
                        TraitKind::Class { .. } => (
                            Property::new_const_slot(new_slot_id),
                            PropertyClass::Class(
                                context.avm2.classes().class.inner_class_definition(),
                            ),
                        ),
                        _ => unreachable!(),
                    };

                    resolved_traits.insert(trait_data.name(), new_prop);

                    if let Some(metadata) = trait_data.metadata() {
                        slot_metadata_table.insert(new_slot_id, metadata);
                    }

                    slot_classes[new_slot_id as usize] = new_class;
                }
                TraitKind::Function { .. } => panic!("TraitKind::Function shouldn't appear"),
            }
        }
    }

    /// Retrieve a bound instance method suitable for use as a value.
    ///
    /// This returns the bound method object itself, as well as it's dispatch
    /// ID. You will need the additional properties in order to install the
    /// method into your object.
    ///
    /// You should only call this method once per receiver/name pair, and cache
    /// the result. Otherwise, code that relies on bound methods having stable
    /// object identitities (e.g. `EventDispatcher.removeEventListener`) will
    /// fail.
    pub fn make_bound_method(
        self,
        activation: &mut Activation<'_, 'gc>,
        receiver: Object<'gc>,
        disp_id: u32,
    ) -> Option<FunctionObject<'gc>> {
        self.get_full_method(disp_id)
            .map(|method| Self::bind_method(activation, receiver, method))
    }

    /// Bind an instance method to a receiver, allowing it to be used as a value. See `VTable::make_bound_method`
    pub fn bind_method(
        activation: &mut Activation<'_, 'gc>,
        receiver: Object<'gc>,
        method: ClassBoundMethod<'gc>,
    ) -> FunctionObject<'gc> {
        let ClassBoundMethod {
            class_obj,
            scope,
            method,
            ..
        } = method;

        FunctionObject::from_method(
            activation,
            method,
            scope.expect("Scope should exist here"),
            Some(receiver),
            class_obj,
        )
    }

    /// Install a const trait on the global object.
    /// This should only ever be called via `Object::install_const_late`,
    /// on the `global` object.
    pub fn install_const_trait_late(
        self,
        mc: &Mutation<'gc>,
        name: QName<'gc>,
        value: Value<'gc>,
        class: ClassObject<'gc>,
    ) -> u32 {
        let mut write = self.0.write(mc);

        write.default_slots.push(Some(value));
        let new_slot_id = write.default_slots.len() as u32 - 1;
        write
            .resolved_traits
            .insert(name, Property::new_const_slot(new_slot_id));
        write
            .slot_classes
            .push(PropertyClass::Class(class.inner_class_definition()));

        new_slot_id
    }

    /// Install an existing trait under a new name, provided by interface.
    /// This should only ever be called by `link_interfaces`.
    pub fn copy_property_for_interface(
        self,
        mc: &Mutation<'gc>,
        public_name: QName<'gc>,
        interface_name: QName<'gc>,
    ) {
        let mut write = self.0.write(mc);

        let prop = write.resolved_traits.get(public_name).cloned();

        if let Some(prop) = prop {
            write.resolved_traits.insert(interface_name, prop);
        }
    }

    pub fn public_properties(self) -> Vec<(AvmString<'gc>, Property)> {
        let read = self.0.read();

        let mut props = Vec::new();

        for (name, ns, prop) in read.resolved_traits.iter() {
            if ns.is_public() {
                props.push((name, *prop));
            }
        }
        props
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
