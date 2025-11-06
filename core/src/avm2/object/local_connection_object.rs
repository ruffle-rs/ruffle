use crate::avm2::activation::Activation;
use crate::avm2::amf::deserialize_value;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, EventObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Domain, Error};
use crate::context::UpdateContext;
use crate::local_connection::{LocalConnectionHandle, LocalConnections};
use crate::string::AvmString;
use core::fmt;
use flash_lso::types::Value as AmfValue;
use gc_arena::barrier::unlock;
use gc_arena::{lock::Lock, Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use std::cell::RefCell;

/// A class instance allocator that allocates LocalConnection objects.
pub fn local_connection_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let object = LocalConnectionObject(Gc::new(
        activation.gc(),
        LocalConnectionObjectData {
            base,
            connection_handle: RefCell::new(None),
            client: Lock::new(None),
        },
    ));

    object.set_client(activation.gc(), object.into());

    Ok(object.into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct LocalConnectionObject<'gc>(pub Gc<'gc, LocalConnectionObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct LocalConnectionObjectWeak<'gc>(pub GcWeak<'gc, LocalConnectionObjectData<'gc>>);

impl fmt::Debug for LocalConnectionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalConnectionObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct LocalConnectionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    connection_handle: RefCell<Option<LocalConnectionHandle>>,

    client: Lock<Option<Object<'gc>>>,
}

impl<'gc> LocalConnectionObject<'gc> {
    pub fn is_connected(self) -> bool {
        self.0.connection_handle.borrow().is_some()
    }

    pub fn client(self) -> Object<'gc> {
        self.0.client.get().expect("Client must be initialized")
    }

    pub fn set_client(&self, mc: &Mutation<'gc>, client: Object<'gc>) {
        unlock!(Gc::write(mc, self.0), LocalConnectionObjectData, client).set(Some(client));
    }

    pub fn connect(self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        if self.is_connected() {
            return false;
        }

        let connection_handle = activation.context.local_connections.connect(
            &LocalConnections::get_domain(activation.context.root_swf.url()),
            (activation.domain(), self),
            &name,
        );
        let result = connection_handle.is_some();

        *self.0.connection_handle.borrow_mut() = connection_handle;

        result
    }

    pub fn disconnect(self, activation: &mut Activation<'_, 'gc>) {
        if let Some(conn_handle) = self.0.connection_handle.borrow_mut().take() {
            activation.context.local_connections.close(conn_handle);
        }
    }

    pub fn send_status(self, context: &mut UpdateContext<'gc>, status: AvmString<'gc>) {
        let mut activation = Activation::from_nothing(context);

        let status_event_cls = activation.avm2().classes().statusevent;
        let event_name = istr!("status");
        let event = EventObject::from_class_and_args(
            &mut activation,
            status_event_cls,
            &[
                event_name.into(),
                false.into(),
                false.into(),
                Value::Null,
                status.into(),
            ],
        );

        Avm2::dispatch_event(activation.context, event, self.into());
    }

    pub fn run_method(
        self,
        context: &mut UpdateContext<'gc>,
        domain: Domain<'gc>,
        method_name: AvmString<'gc>,
        amf_arguments: Vec<AmfValue>,
    ) {
        let mut activation = Activation::from_domain(context, domain);
        let mut arguments = Vec::with_capacity(amf_arguments.len());

        for argument in amf_arguments {
            arguments
                .push(deserialize_value(&mut activation, &argument).unwrap_or(Value::Undefined));
        }
        let arguments = FunctionArgs::from_slice(&arguments);

        let client = Value::from(self.client());
        if let Err(e) = client.call_public_property(method_name, arguments, &mut activation) {
            match e.as_avm_error() {
                Some(error) => {
                    let event_name = istr!("asyncError");
                    let async_error_event_cls = activation.avm2().classes().asyncerrorevent;

                    let text = AvmString::new_utf8(activation.gc(), format!("Error #2095: flash.net.LocalConnection was unable to invoke callback {method_name}."));

                    let event = EventObject::from_class_and_args(
                        &mut activation,
                        async_error_event_cls,
                        &[
                            event_name.into(),
                            false.into(),
                            false.into(),
                            text.into(),
                            error,
                        ],
                    );

                    Avm2::dispatch_event(activation.context, event, self.into());
                }
                _ => {
                    tracing::error!("Unhandled error dispatching AVM2 LocalConnection method call to '{method_name}': {e}");
                }
            }
        }
    }
}

impl<'gc> TObject<'gc> for LocalConnectionObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
