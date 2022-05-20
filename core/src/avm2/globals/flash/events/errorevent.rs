use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::events::EventData;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.ErrorEvent`'s instance constructor.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?; // TextEvent, Event uses these
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            let id = args
                .get(4)
                .cloned()
                .unwrap_or_else(|| 0.into())
                .coerce_to_i32(activation)?;
            let event_data = evt.event_data_mut();
            match event_data {
                EventData::Error {
                    ref mut error_id, ..
                } => {
                    *error_id = id;
                }
                EventData::IOError {
                    ref mut error_id, ..
                } => {
                    *error_id = id;
                }
                EventData::SecurityError {
                    ref mut error_id, ..
                } => {
                    *error_id = id;
                }
                _ => {}
            }
        }
    }
    Ok(Value::Undefined)
}

/// Implements `flash.events.ErrorEvent`'s class constructor.
fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `errorID`'s getter.
fn error_id<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Error { error_id, .. } = evt.event_data() {
                return Ok(Value::Integer(*error_id));
            }
            if let EventData::IOError { error_id, .. } = evt.event_data() {
                return Ok(Value::Integer(*error_id));
            }
            if let EventData::SecurityError { error_id, .. } = evt.event_data() {
                return Ok(Value::Integer(*error_id));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `ErrorEvent`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "ErrorEvent"),
        Some(QName::new(Namespace::package("flash.events"), "TextEvent").into()),
        Method::from_builtin(instance_init, "<ErrorEvent instance initializer>", mc),
        Method::from_builtin(class_init, "<ErrorEvent class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] = &[("ERROR", "error")];

    write.define_public_constant_string_class_traits(CONSTANTS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("errorID", Some(error_id), None)];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
