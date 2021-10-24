//! Boxed QNames

use crate::avm2::activation::Activation;
use crate::avm2::names::QName;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates QName objects.
pub fn qname_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(QNameObject(GcCell::allocate(
        activation.context.gc_context,
        QNameObjectData { base, qname: None },
    ))
    .into())
}

/// An Object which represents a boxed QName.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct QNameObject<'gc>(GcCell<'gc, QNameObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct QNameObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The QName name this object is associated with.
    qname: Option<QName<'gc>>,
}

impl<'gc> QNameObject<'gc> {
    /// Box a QName into an object.
    pub fn from_qname(
        activation: &mut Activation<'_, 'gc, '_>,
        qname: QName<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().qname;
        let proto = activation.avm2().prototypes().qname;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut this: Object<'gc> = QNameObject(GcCell::allocate(
            activation.context.gc_context,
            QNameObjectData {
                base,
                qname: Some(qname),
            },
        ))
        .into();
        this.install_instance_traits(activation, class)?;

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    pub fn qname(&self) -> Option<Ref<QName<'gc>>> {
        let read = self.0.read();
        read.qname.as_ref()?;

        Some(Ref::map(read, |r| r.qname.as_ref().unwrap()))
    }

    pub fn init_qname(self, mc: MutationContext<'gc, '_>, qname: QName<'gc>) {
        self.0.write(mc).qname = Some(qname);
    }
}

impl<'gc> TObject<'gc> for QNameObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_qname_object(self) -> Option<QNameObject<'gc>> {
        Some(self)
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::QNameObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(QNameObject(GcCell::allocate(
            activation.context.gc_context,
            QNameObjectData { base, qname: None },
        ))
        .into())
    }
}
