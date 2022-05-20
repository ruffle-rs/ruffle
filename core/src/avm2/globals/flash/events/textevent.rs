use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::events::EventData;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.TextEvent`'s instance constructor.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?; // Event uses these
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            let text_arg = args
                .get(3)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_string(activation)?;
            let event_data = evt.event_data_mut();
            match event_data {
                EventData::Text { ref mut text, .. } => {
                    *text = text_arg;
                }
                EventData::Error { ref mut text, .. } => {
                    *text = text_arg;
                }
                EventData::IOError { ref mut text, .. } => {
                    *text = text_arg;
                }
                EventData::SecurityError { ref mut text, .. } => {
                    *text = text_arg;
                }
                _ => {}
            }
        }
    }
    Ok(Value::Undefined)
}

/// Implements `flash.events.TextEvent`'s class constructor.
fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `text`'s setter.
fn set_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            let text_arg = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_string(activation)?;
            let event_data = evt.event_data_mut();
            match event_data {
                EventData::Text { ref mut text, .. } => {
                    *text = text_arg;
                }
                EventData::Error { ref mut text, .. } => {
                    *text = text_arg;
                }
                EventData::IOError { ref mut text, .. } => {
                    *text = text_arg;
                }
                EventData::SecurityError { ref mut text, .. } => {
                    *text = text_arg;
                }
                _ => {}
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `text`'s getter.
fn text<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Text { text, .. } = evt.event_data() {
                return Ok(Value::String(*text));
            }
            if let EventData::Error { text, .. } = evt.event_data() {
                return Ok(Value::String(*text));
            }
            if let EventData::IOError { text, .. } = evt.event_data() {
                return Ok(Value::String(*text));
            }
            if let EventData::SecurityError { text, .. } = evt.event_data() {
                return Ok(Value::String(*text));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `TextEvent`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "TextEvent"),
        Some(QName::new(Namespace::package("flash.events"), "Event").into()),
        Method::from_builtin(instance_init, "<TextEvent instance initializer>", mc),
        Method::from_builtin(class_init, "<TextEvent class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] = &[("LINK", "link"), ("TEXT_INPUT", "textInput")];

    write.define_public_constant_string_class_traits(CONSTANTS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("text", Some(text), Some(set_text))];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
