use crate::avm2::activation::Activation;
use crate::avm2::amf::deserialize_value;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Domain, Error};
use crate::context::UpdateContext;
use crate::local_connection::{LocalConnectionHandle, LocalConnections};
use crate::string::AvmString;
use core::fmt;
use flash_lso::types::Value as AmfValue;
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

#[derive(Collect)]
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

    pub fn connect(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        if self.is_connected() {
            return false;
        }

        let connection_handle = activation.context.local_connections.connect(
            &LocalConnections::get_domain(activation.context.swf.url()),
            (activation.domain(), *self),
            &name,
        );
        let result = connection_handle.is_some();
        self.0
            .write(activation.context.gc_context)
            .connection_handle = connection_handle;
        result
    }

    pub fn disconnect(&self, activation: &mut Activation<'_, 'gc>) {
        if let Some(conn_handle) = self
            .0
            .write(activation.context.gc_context)
            .connection_handle
            .take()
        {
            activation.context.local_connections.close(conn_handle);
        }
    }

    pub fn send_status(&self, context: &mut UpdateContext<'_, 'gc>, status: &'static str) {
        let mut activation = Activation::from_nothing(context.reborrow());
        if let Ok(event) = activation.avm2().classes().statusevent.construct(
            &mut activation,
            &[
                "status".into(),
                false.into(),
                false.into(),
                Value::Null,
                status.into(),
            ],
        ) {
            Avm2::dispatch_event(&mut activation.context, event, (*self).into());
        }
    }

    pub fn run_method(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        domain: Domain<'gc>,
        method_name: AvmString<'gc>,
        amf_arguments: Vec<AmfValue>,
    ) {
        let mut activation = Activation::from_domain(context.reborrow(), domain);
        let mut arguments = Vec::with_capacity(amf_arguments.len());

        for argument in amf_arguments {
            arguments
                .push(deserialize_value(&mut activation, &argument).unwrap_or(Value::Undefined));
        }

        if let Ok(client) = self
            .get_public_property("client", &mut activation)
            .and_then(|v| v.coerce_to_object(&mut activation))
        {
            if let Err(e) = client.call_public_property(method_name, &arguments, &mut activation) {
                match e {
                    Error::AvmError(error) => {
                        if let Ok(event) = activation.avm2().classes().asyncerrorevent.construct(
                            &mut activation,
                            &[
                                "asyncError".into(),
                                false.into(),
                                false.into(),
                                error,
                                error,
                            ],
                        ) {
                            Avm2::dispatch_event(&mut activation.context, event, (*self).into());
                        }
                    }
                    _ => {
                        tracing::error!("Unhandled error dispatching AVM2 LocalConnection method call to '{method_name}': {e}");
                    }
                }
            }
        }
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
