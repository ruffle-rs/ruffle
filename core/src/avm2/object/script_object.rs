//! Default AVM2 object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::dynamic_map::{DynamicKey, DynamicMap};
use crate::avm2::error;
use crate::avm2::object::{ClassObject, FunctionObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::vtable::VTable;
use crate::avm2::{Error, Multiname, QName};
use crate::string::AvmString;
use gc_arena::barrier::{unlock, Write};
use gc_arena::{
    lock::{Lock, RefLock},
    Collect, Gc, GcWeak, Mutation,
};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

/// A class instance allocator that allocates `ScriptObject`s.
pub fn scriptobject_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(ScriptObject(Gc::new(activation.gc(), base)).into())
}

/// Default implementation of `avm2::Object`.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(pub Gc<'gc, ScriptObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ScriptObjectWeak<'gc>(pub GcWeak<'gc, ScriptObjectData<'gc>>);

/// Base data common to all `TObject` implementations.
///
/// Host implementations of `TObject` should embed `ScriptObjectData` and
/// forward any trait method implementations it does not overwrite to this
/// struct.
#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(align(8))]
pub struct ScriptObjectData<'gc> {
    /// Values stored on this object.
    values: RefLock<DynamicMap<DynamicKey<'gc>, Value<'gc>>>,

    /// Slots stored on this object.
    slots: Vec<Lock<Value<'gc>>>,

    /// Methods stored on this object.
    bound_methods: RefLock<Vec<Option<FunctionObject<'gc>>>>,

    /// Implicit prototype of this script object.
    proto: Lock<Option<Object<'gc>>>,

    /// The `Class` that this is an instance of.
    instance_class: Class<'gc>,

    /// The table used for non-dynamic property lookups.
    vtable: Lock<VTable<'gc>>,
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        self.0
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }
}

pub fn maybe_int_property(name: AvmString<'_>) -> DynamicKey<'_> {
    // TODO: this should use a custom implementation, not parse()
    // FP is much stricter here, only allowing pure natural numbers without sign or leading zeros
    if let Ok(val) = name.parse::<u32>() {
        DynamicKey::Uint(val)
    } else {
        DynamicKey::String(name)
    }
}

impl<'gc> ScriptObject<'gc> {
    /// Creates an instance of the Object class, exactly as if `new Object()`
    /// were called, but without going through any construction or call
    /// machinery (since it's unnecessary for the Object class).
    pub fn new_object(activation: &mut Activation<'_, 'gc>) -> Object<'gc> {
        let object_class = activation.avm2().classes().object;

        ScriptObject(Gc::new(
            activation.gc(),
            ScriptObjectData::new(object_class),
        ))
        .into()
    }

    /// Construct an instance with a possibly-none class and proto chain.
    /// NOTE: this is a low-level function.
    /// This should *not* be used unless you really need
    /// to do something low-level, weird or lazily initialize the object.
    /// You shouldn't let scripts observe this weirdness.
    ///
    /// The proper way to create a normal empty ScriptObject (AS "Object") is to call
    /// `ScriptObject::new_object(activation)`.
    ///
    /// Calling `custom_object(mc, object_class, object_class.prototype()` is
    /// technically also equivalent, but not recommended outside VM initialization code
    pub fn custom_object(
        mc: &Mutation<'gc>,
        class: Class<'gc>,
        proto: Option<Object<'gc>>,
        vtable: VTable<'gc>,
    ) -> Object<'gc> {
        ScriptObject(Gc::new(
            mc,
            ScriptObjectData::custom_new(class, proto, vtable),
        ))
        .into()
    }
}

impl<'gc> ScriptObjectData<'gc> {
    /// Create new object data of a given class.
    /// This is a low-level function used to implement things like object allocators.
    pub fn new(instance_of: ClassObject<'gc>) -> Self {
        Self::custom_new(
            instance_of.inner_class_definition(),
            Some(instance_of.prototype()),
            instance_of.instance_vtable(),
        )
    }

    /// Create new custom object data of a given possibly-none class and prototype.
    /// This is a low-level function used to implement things like object allocators.
    /// This should *not* be used, unless you really need
    /// to do something weird or lazily initialize the object.
    /// You shouldn't let scripts observe this weirdness.
    pub fn custom_new(
        instance_class: Class<'gc>,
        proto: Option<Object<'gc>>,
        vtable: VTable<'gc>,
    ) -> Self {
        let default_slots = vtable.default_slots();
        let mut slots = vec![Lock::new(Value::Undefined); default_slots.len()];

        for (i, value) in default_slots.iter().enumerate() {
            if let Some(value) = value {
                slots[i] = Lock::new(*value);
            }
        }

        ScriptObjectData {
            values: RefLock::new(Default::default()),
            slots,
            bound_methods: RefLock::new(Vec::new()),
            proto: Lock::new(proto),
            instance_class,
            vtable: Lock::new(vtable),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ScriptObjectWrapper<'gc>(pub Gc<'gc, ScriptObjectData<'gc>>);

impl<'gc> ScriptObjectWrapper<'gc> {
    /// Retrieve the values stored directly on this ScriptObjectData.
    pub fn values(&self) -> Ref<'_, DynamicMap<DynamicKey<'gc>, Value<'gc>>> {
        self.0.values.borrow()
    }

    pub fn values_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> RefMut<'_, DynamicMap<DynamicKey<'gc>, Value<'gc>>> {
        unlock!(Gc::write(mc, self.0), ScriptObjectData, values).borrow_mut()
    }

    fn bound_methods(&self) -> Ref<'_, Vec<Option<FunctionObject<'gc>>>> {
        self.0.bound_methods.borrow()
    }

    fn bound_methods_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> RefMut<'_, Vec<Option<FunctionObject<'gc>>>> {
        unlock!(Gc::write(mc, self.0), ScriptObjectData, bound_methods).borrow_mut()
    }

    pub fn get_property_local(
        &self,
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let dynamic_lookup = get_dynamic_property(
            activation,
            multiname,
            Some(&*self.values()),
            self.proto(),
            self.instance_class(),
        )?;

        if let Some(value) = dynamic_lookup {
            Ok(value)
        } else {
            // Special case: Unresolvable properties on dynamic classes are treated
            // as dynamic properties that have not yet been set, and yield
            // `undefined`
            if self.is_sealed() {
                Err(error::make_reference_error(
                    activation,
                    error::ReferenceErrorCode::InvalidRead,
                    multiname,
                    self.instance_class(),
                ))
            } else {
                Ok(Value::Undefined)
            }
        }
    }

    pub fn set_property_local(
        &self,
        multiname: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        if self.is_sealed() || !multiname.valid_dynamic_name() {
            return Err(error::make_reference_error(
                activation,
                error::ReferenceErrorCode::InvalidWrite,
                multiname,
                self.instance_class(),
            ));
        }

        let Some(local_name) = multiname.local_name() else {
            return Err(error::make_reference_error(
                activation,
                error::ReferenceErrorCode::InvalidWrite,
                multiname,
                self.instance_class(),
            ));
        };

        // Unbelievably cursed special case in avmplus:
        // https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/ScriptObject.cpp#L311-L315
        let key = maybe_int_property(local_name);

        self.values_mut(activation.gc()).insert(key, value);
        Ok(())
    }

    pub fn init_property_local(
        &self,
        multiname: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        self.set_property_local(multiname, value, activation)
    }

    pub fn delete_property_local(&self, mc: &Mutation<'gc>, multiname: &Multiname<'gc>) -> bool {
        // TODO: FP behaves differently here in interpreter mode vs JIT mode
        if !multiname.valid_dynamic_name() {
            return false;
        }

        if let Some(name) = multiname.local_name() {
            let key = maybe_int_property(name);
            self.values_mut(mc).remove(&key);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn get_slot(&self, id: u32) -> Value<'gc> {
        self.0
            .slots
            .get(id as usize)
            .cloned()
            .map(|s| s.get())
            .expect("Slot index out of bounds")
    }

    /// Set a slot by its index.
    pub fn set_slot(&self, id: u32, value: Value<'gc>, mc: &Mutation<'gc>) {
        let slot = self
            .0
            .slots
            .get(id as usize)
            .expect("Slot index out of bounds");

        Gc::write(mc, self.0);
        // SAFETY: We just triggered a write barrier on the Gc.
        let slot_write = unsafe { Write::assume(slot) };
        slot_write.unlock().set(value);
    }

    /// Retrieve a bound method from the method table.
    pub fn get_bound_method(&self, id: u32) -> Option<FunctionObject<'gc>> {
        self.bound_methods().get(id as usize).and_then(|v| *v)
    }

    pub fn has_own_dynamic_property(&self, name: &Multiname<'gc>) -> bool {
        if name.valid_dynamic_name() {
            if let Some(name) = name.local_name() {
                let key = maybe_int_property(name);
                return self.values().as_hashmap().get(&key).is_some();
            }
        }
        false
    }

    pub fn has_own_property(&self, name: &Multiname<'gc>) -> bool {
        self.vtable().has_trait(name) || self.has_own_dynamic_property(name)
    }

    pub fn proto(&self) -> Option<Object<'gc>> {
        self.0.proto.get()
    }

    pub fn set_proto(&self, mc: &Mutation<'gc>, proto: Object<'gc>) {
        unlock!(Gc::write(mc, self.0), ScriptObjectData, proto).set(Some(proto));
    }

    pub fn get_next_enumerant(&self, last_index: u32) -> u32 {
        self.values()
            .next(last_index as usize)
            .map(|val| val as u32)
            .unwrap_or(0)
    }

    pub fn get_enumerant_name(&self, index: u32) -> Option<Value<'gc>> {
        self.values().key_at(index as usize).map(|key| match key {
            DynamicKey::String(name) => Value::String(*name),
            DynamicKey::Object(obj) => Value::Object(*obj),
            DynamicKey::Uint(val) => Value::Number(*val as f64),
        })
    }

    pub fn property_is_enumerable(&self, name: AvmString<'gc>) -> bool {
        let key = maybe_int_property(name);
        self.values()
            .as_hashmap()
            .get(&key)
            .is_some_and(|prop| prop.enumerable)
    }

    pub fn set_local_property_is_enumerable(
        &self,
        mc: &Mutation<'gc>,
        name: AvmString<'gc>,
        is_enumerable: bool,
    ) {
        let key = maybe_int_property(name);
        self.values_mut(mc).entry(key).and_modify(|v| {
            v.enumerable = is_enumerable;
        });
    }

    /// Install a method into the object.
    pub fn install_bound_method(
        &self,
        mc: &Mutation<'gc>,
        disp_id: u32,
        function: FunctionObject<'gc>,
    ) {
        let mut bound_methods = self.bound_methods_mut(mc);

        if bound_methods.len() <= disp_id as usize {
            bound_methods.resize_with(disp_id as usize + 1, Default::default);
        }

        *bound_methods.get_mut(disp_id as usize).unwrap() = Some(function);
    }

    /// Get the `Class` for this object.
    pub fn instance_class(&self) -> Class<'gc> {
        self.0.instance_class
    }

    /// Get the vtable for this object, if it has one.
    pub fn vtable(&self) -> VTable<'gc> {
        self.0.vtable.get()
    }

    pub fn is_sealed(&self) -> bool {
        self.instance_class().is_sealed()
    }

    pub fn set_vtable(&self, mc: &Mutation<'gc>, vtable: VTable<'gc>) {
        // Make sure both vtables have the same number of slots
        assert_eq!(
            self.vtable().default_slots().len(),
            vtable.default_slots().len()
        );

        unlock!(Gc::write(mc, self.0), ScriptObjectData, vtable).set(vtable);
    }

    pub fn class_name(&self) -> QName<'gc> {
        self.instance_class().name()
    }
}

impl Debug for ScriptObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("ScriptObject")
            .field("name", &ScriptObjectWrapper(self.0).class_name())
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

/// General-purpose function for looking up dynamic properties on an object. This
/// is used in `ScriptObject::get_property_local`, `Value::get_property`,
/// `Value::call_property`, and `VectorObject::get_property_local`. This method
/// returns `None` when the property is found on neither the local values nor
/// the prototype.
pub fn get_dynamic_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    multiname: &Multiname<'gc>,
    local_values: Option<&DynamicMap<DynamicKey<'gc>, Value<'gc>>>,
    prototype: Option<Object<'gc>>,
    instance_class: Class<'gc>,
) -> Result<Option<Value<'gc>>, Error<'gc>> {
    if !multiname.valid_dynamic_name() {
        let error_code = if multiname.has_multiple_ns() {
            error::ReferenceErrorCode::InvalidNsRead
        } else {
            error::ReferenceErrorCode::InvalidRead
        };

        return Err(error::make_reference_error(
            activation,
            error_code,
            multiname,
            instance_class,
        ));
    }

    let Some(local_name) = multiname.local_name() else {
        // when can this happen?
        return Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::InvalidRead,
            multiname,
            instance_class,
        ));
    };

    // Unbelievably cursed special case in avmplus:
    // https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/ScriptObject.cpp#L195-L199
    let key = maybe_int_property(local_name);

    if let Some(values) = local_values {
        let value = values.as_hashmap().get(&key);
        if let Some(value) = value {
            return Ok(Some(value.value));
        }
    }

    // follow the prototype chain
    let mut proto = prototype;
    while let Some(obj) = proto {
        let obj = obj.base();
        let values = obj.values();
        let value = values.as_hashmap().get(&key);
        if let Some(value) = value {
            return Ok(Some(value.value));
        }
        proto = obj.proto();
    }

    Ok(None)
}
