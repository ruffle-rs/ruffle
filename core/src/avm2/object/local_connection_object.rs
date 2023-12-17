use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::local_connection::{LocalConnection, LocalConnectionHandle};
use crate::string::AvmString;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates LocalConnection objects.
pub fn local_connection_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(LocalConnectionObject(GcCell::new(
        activation.context.gc_context,
        LocalConnectionObjectData {
            base,
            connection_handle: None,
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct LocalConnectionObject<'gc>(pub GcCell<'gc, LocalConnectionObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct LocalConnectionObjectWeak<'gc>(pub GcWeakCell<'gc, LocalConnectionObjectData<'gc>>);

impl fmt::Debug for LocalConnectionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalConnectionObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct LocalConnectionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    #[collect(require_static)]
    connection_handle: Option<LocalConnectionHandle>,
}

impl<'gc> LocalConnectionObject<'gc> {
    pub fn is_connected(&self) -> bool {
        self.0.read().connection_handle.is_some()
    }

    pub fn connection_handle(&self) -> Option<LocalConnectionHandle> {
        self.0.read().connection_handle
    }

    pub fn connect(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) {
        assert!(!self.is_connected());

        let connection_handle = activation
            .context
            .local_connections
            .insert(LocalConnection::new(*self, name));
        self.0
            .write(activation.context.gc_context)
            .connection_handle = Some(connection_handle);
    }

    pub fn disconnect(&self, activation: &mut Activation<'_, 'gc>) {
        if let Some(conn_handle) = self.0.read().connection_handle {
            activation.context.local_connections.remove(conn_handle);
        }

        self.0
            .write(activation.context.gc_context)
            .connection_handle = None;
    }
}

impl<'gc> TObject<'gc> for LocalConnectionObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object((*self).into()))
    }

    fn as_local_connection_object(&self) -> Option<LocalConnectionObject<'gc>> {
        Some(*self)
    }
}
