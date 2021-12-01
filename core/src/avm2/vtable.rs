use crate::avm2::activation::Activation;
use crate::avm2::property_map::PropertyMap;
use crate::avm2::property::Property;
use crate::avm2::value::Value;
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::Error;
use crate::avm2::method::Method;
use crate::avm2::names::{QName, Multiname};
use crate::avm2::object::{FunctionObject, ClassObject, Object};
use gc_arena::{Collect, GcCell, MutationContext};


#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct VTable<'gc>(GcCell<'gc, VTableData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct VTableData<'gc> {
    defining_class: Option<ClassObject<'gc>>,

    /// should always be Some post-initialization
    scope: Option<ScopeChain<'gc>>,

    resolved_traits: PropertyMap<'gc, Property>,

    method_table: Vec<Option<(Option<ClassObject<'gc>>, Method<'gc>)>>,

    default_slots: Vec<Option<Value<'gc>>>,
}
impl<'gc> VTable<'gc> {
    pub fn empty(mc: MutationContext<'gc, '_>) -> Self {
        VTable(GcCell::allocate(
            mc,
            VTableData {
                defining_class: None,
                scope: None,
                resolved_traits: PropertyMap::new(),
                method_table: vec![],
                default_slots: vec![],
            },
        ))
    }

    pub fn duplicate(self, mc: MutationContext<'gc, '_>) -> Self {
        VTable(GcCell::allocate(mc, self.0.read().clone()))
    }

    pub fn get_trait(self, name: &Multiname<'gc>) -> Option<Property> {
        self.0.read().resolved_traits.get_multiname(name).cloned()
    }

    pub fn has_trait(self, name: &Multiname<'gc>) -> bool {
        self.0.read().resolved_traits.get_multiname(name).is_some()
    }

    pub fn get_method(self, disp_id: u32) -> Option<Method<'gc>> {
        self.0.read().method_table.get(disp_id as usize).cloned().flatten().map(|x| x.1)
    }

    pub fn get_full_method(self, disp_id: u32) -> Option<(Option<ClassObject<'gc>>, Method<'gc>)> {
        self.0.read().method_table.get(disp_id as usize).cloned().flatten()
    }

    pub fn default_slots(self) -> Vec<Option<Value<'gc>>> {
        self.0.read().default_slots.clone()
    }

    /// Calculate the flattened list of instance traits that this class
    /// maintains.
    ///
    /// This should be run during the class finalization step, before instances
    /// are linked (as instances will further add traits to the list).
    pub fn init_vtable(
        self,
        defining_class: Option<ClassObject<'gc>>,
        traits: &[Trait<'gc>],
        scope: ScopeChain<'gc>,
        superclass_vtable: Option<Self>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(activation.context.gc_context);


        write.defining_class = defining_class;
        write.scope = Some(scope);

        if let Some(superclass_vtable) = superclass_vtable {
            write.resolved_traits = superclass_vtable.0.read().resolved_traits.clone();
            write.method_table = superclass_vtable.0.read().method_table.clone();
            write.default_slots = superclass_vtable.0.read().default_slots.clone();
        }

        let mut max_disp_id = write.method_table.len().saturating_sub(1) as u32;
        let mut max_slot_id = write.default_slots.len().saturating_sub(1) as u32;

        for trait_data in traits {
            if let Some(disp_id) = trait_data.disp_id() {
                max_disp_id = max_disp_id.max(disp_id);
            }
            if let Some(slot_id) = trait_data.slot_id() {
                max_slot_id = max_slot_id.max(slot_id);
            }
        }

        write.method_table.resize_with(max_disp_id as usize + 1, Default::default);
        write.default_slots.resize_with(max_slot_id as usize + 1, Default::default);

        for trait_data in traits {
            let mut trait_data = trait_data.clone();
            if matches!(trait_data.disp_id(), Some(0)) {
                max_disp_id += 1;
                trait_data.set_disp_id(max_disp_id);
                write.method_table.resize_with(max_disp_id as usize + 1, Default::default);
            }
            if let Some(disp_id) = trait_data.disp_id() {
                let entry = (defining_class, trait_data.as_method().unwrap());
                write.method_table[disp_id as usize] = Some(entry);
            }

            if matches!(trait_data.slot_id(), Some(0)) {
                max_slot_id += 1;
                trait_data.set_slot_id(max_slot_id);
                write.default_slots.resize_with(max_slot_id as usize + 1, Default::default);
            }
            if let Some(slot_id) = trait_data.slot_id() {
                let value = trait_to_default_value(scope, &trait_data, activation);
                write.default_slots[slot_id as usize] = value;
            }

            let new_prop = trait_to_property(&trait_data);

            if let Some(prop_slot) = write.resolved_traits.get_mut(trait_data.name()) {
                match trait_data.kind() {
                    TraitKind::Getter { disp_id, .. } => prop_slot.install_virtual_getter(*disp_id)?,
                    TraitKind::Setter { disp_id, .. } => prop_slot.install_virtual_setter(*disp_id)?,
                    _ => *prop_slot = new_prop,
                }
            } else {
                write.resolved_traits.insert(trait_data.name(), new_prop);
            }
        }


        Ok(())
    }


    /// Retrieve a bound instance method suitable for use as a value.
    ///
    /// This returns the bound method object itself, as well as it's dispatch
    /// ID. You will need the additional properties in order to install the
    /// method into your object.
    ///
    /// You should only call this method once per reciever/name pair, and cache
    /// the result. Otherwise, code that relies on bound methods having stable
    /// object identitities (e.g. `EventDispatcher.removeEventListener`) will
    /// fail.
    pub fn make_bound_method(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        receiver: Object<'gc>,
        disp_id: u32,
    ) -> Option<FunctionObject<'gc>> {
        if let Some((superclass, method)) = self.get_full_method(disp_id) {
            let scope = self.0.read().scope.unwrap();
            Some(
                FunctionObject::from_method(
                    activation,
                    method,
                    scope,
                    Some(receiver),
                    superclass,
                ),
            )
        } else {
            None
        }
    }

    /// Install a const trait on the global object.
    /// This should only ever be called via `Object::install_const_late`,
    /// on the `global` object.
    pub fn install_const_trait_late(
        self,
        mc: MutationContext<'gc, '_>,
        slot_id: Option<u32>,
        name: QName<'gc>,
        value: Value<'gc>,
    ) -> u32 {
        let mut write = self.0.write(mc);

        let new_slot_id = if let Some(slot_id) = slot_id {

            if slot_id == 0 {
                write.default_slots.push(Some(value));
                write.default_slots.len() as u32 - 1
            } else {
                // Should be safe from slot conflicts, as `global` has a fresh Object vtable.
                if slot_id as usize >= write.default_slots.len() {
                    write.default_slots.resize_with(slot_id as usize + 1, Default::default);
                }
                write.default_slots[slot_id as usize] = Some(value);
                slot_id
            }

        } else {
            write.default_slots.push(Some(value));
            write.default_slots.len() as u32 - 1
        };
        write.resolved_traits.insert(name, Property::new_slot(new_slot_id));

        new_slot_id
    }

    /// Install an existing trait under a new name, provided by interface.
    /// This should only ever be called by `link_interfaces`.
    pub fn copy_property_for_interface(
        self,
        mc: MutationContext<'gc, '_>,
        public_name: QName<'gc>,
        interface_name: QName<'gc>,
    ) {
        let mut write = self.0.write(mc);

        let prop = write.resolved_traits.get(public_name).cloned();

        if let Some(prop) = prop {
            write.resolved_traits.insert(interface_name, prop);
        }
    }

}

fn trait_to_property<'gc>(trait_data: &Trait<'gc>) -> Property {
    match trait_data.kind() {
        TraitKind::Slot { slot_id, .. }
        | TraitKind::Const { slot_id, .. }
        | TraitKind::Function { slot_id, .. }
        | TraitKind::Class { slot_id, .. } => Property::new_slot(*slot_id),
        TraitKind::Method { disp_id, .. } => Property::new_method(*disp_id),
        TraitKind::Getter { disp_id, .. } => {
            let mut prop = Property::new_virtual();
            prop.install_virtual_getter(*disp_id).unwrap();
            prop
        },
        TraitKind::Setter { disp_id, .. } => {
            let mut prop = Property::new_virtual();
            prop.install_virtual_setter(*disp_id).unwrap();
            prop
        },
    }
}

pub fn trait_to_default_value<'gc>(
    scope: ScopeChain<'gc>,
    trait_data: &Trait<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Option<Value<'gc>> {
    match trait_data.kind() {
        TraitKind::Slot { default_value, .. } => Some(default_value.clone()),
        TraitKind::Const { default_value, .. } => Some(default_value.clone()),
        TraitKind::Function { function, .. } => {
            Some(FunctionObject::from_function(
                activation,
                function.clone(),
                scope,
            ).unwrap().into())
        }
        TraitKind::Class { .. } => Some(Value::Undefined),
        _ => None
    }
}

