use crate::avm2::activation::Activation;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::scope::Scope;
use crate::avm2::traits::Trait;
use crate::avm2::string::AvmString;
use crate::avm2::class::Class;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};


#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ByteArrayObject<'gc>(GcCell<'gc, ByteArrayObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ByteArrayObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    storage: ByteArrayStorage
}


impl<'gc> ByteArrayObject<'gc> {
    pub fn new(
        mc: MutationContext<'gc, '_>,
        base_proto: Option<Object<'gc>>
    ) -> Object<'gc> {
        let base = ScriptObjectData::base_new(base_proto, ScriptObjectClass::NoClass);

        ByteArrayObject(GcCell::allocate(mc, ByteArrayObjectData { 
            base,
            storage: ByteArrayStorage::new()
        })).into()
    }


    /// Construct a primitive subclass.
    pub fn derive(
        base_proto: Object<'gc>,
        mc: MutationContext<'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(
            Some(base_proto),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(ByteArrayObject(GcCell::allocate(mc, ByteArrayObjectData { 
            base,
            storage: ByteArrayStorage::new()
        })).into())
    }
    


    
}
impl<'gc> TObject<'gc> for ByteArrayObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ByteArrayObject(*self);
        Ok(ByteArrayObject::new(
            activation.context.gc_context,
            Some(this)
        ))
    }

fn derive(
    &self,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    let this: Object<'gc> = Object::ByteArrayObject(*self);
    let base = ScriptObjectData::base_new(
        Some(this),
        ScriptObjectClass::InstancePrototype(class, scope),
    );

    Ok(ByteArrayObject(GcCell::allocate(activation.context.gc_context, ByteArrayObjectData { 
        base,
        storage: ByteArrayStorage::new()
    })).into())
}
    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_bytearray(&self) -> Option<Ref<ByteArrayStorage>> {
        Some(Ref::map(self.0.read(), |d| &d.storage))
    }
    
    fn as_bytearray_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<ByteArrayStorage>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.storage))
    }
}
