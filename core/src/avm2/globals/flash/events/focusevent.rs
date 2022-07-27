use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::events::{EventData, KeyModifiers};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::{AvmString, Error};
use crate::display_object::{TDisplayObject, TInteractiveObject};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.events.FocusEvent`'s instance constructor with proper defaults
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?; // Event uses these
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            evt.set_event_type(
                args.get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_string(activation)?,
            );

            evt.set_bubbles(
                args.get(1)
                    .cloned()
                    .unwrap_or(Value::Bool(true))
                    .coerce_to_boolean(),
            );

            evt.set_cancelable(
                args.get(2)
                    .cloned()
                    .unwrap_or(Value::Bool(false))
                    .coerce_to_boolean(),
            );

            let related_object = args
                .get(3)
                .cloned()
                .unwrap_or(Value::Null)
                .as_object()
                .and_then(|o| o.as_display_object())
                .and_then(|o| o.as_interactive());

            let shift_key = args
                .get(4)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();

            let key_code = args
                .get(5)
                .cloned()
                .unwrap_or_else(|| 0.into())
                .coerce_to_u32(activation)?;

            let direction = args
                .get(6)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_string(activation)?;

            let mut modifiers = KeyModifiers::default();

            if shift_key {
                modifiers.insert(KeyModifiers::SHIFT);
            }

            evt.set_event_data(EventData::Focus {
                related_object,
                key_code,
                direction,
                modifiers,
            });
        }
    }
    Ok(Value::Undefined)
}

/// Implements `flash.events.FocusEvent`'s class constructor.
fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `direction`'s setter.
fn set_direction<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Focus { direction, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_string(activation)?;
                *direction = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `direction`'s getter.
fn direction<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Focus { direction, .. } = evt.event_data() {
                return Ok(Value::String(*direction));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `shiftKey`'s getter.
pub fn shift_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Focus { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::SHIFT).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `shiftKey`'s setter.
pub fn set_shift_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Focus { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::SHIFT);
                } else {
                    modifiers.remove(KeyModifiers::SHIFT);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `relatedObject`'s getter.
pub fn related_object<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Focus { related_object, .. } = evt.event_data() {
                return Ok(related_object
                    .map(|o| o.as_displayobject().object2())
                    .unwrap_or(Value::Null));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `relatedObject`'s setter.
pub fn set_related_object<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Focus { related_object, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .and_then(|o| o.as_object())
                    .and_then(|o| o.as_display_object())
                    .and_then(|o| o.as_interactive());

                *related_object = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Stubs `isRelatedObjectInaccessible`'s getter.
pub fn is_related_object_inaccessible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(false.into())
}

/// Implements `keyCode`'s setter.
pub fn set_key_code<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Focus { key_code, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or_else(|| 0.into())
                    .coerce_to_u32(activation)?;

                *key_code = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `keyCode`'s getter.
pub fn key_code<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Focus { key_code, .. } = evt.event_data() {
                return Ok(Value::Unsigned(*key_code));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `toString`'s getter.
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(event) = this.as_event() {
            if let EventData::Focus {
                related_object,
                modifiers,
                key_code,
                ..
            } = event.event_data()
            {
                let event_type = event.event_type();
                let bubbles = event.is_bubbling();
                let cancelable = event.is_cancelable();

                let related_object = if let Some(related_object) =
                    related_object.and_then(|ro| ro.as_displayobject().object2().as_object())
                {
                    related_object
                        .to_string(activation.context.gc_context)?
                        .coerce_to_string(activation)?
                } else {
                    "null".into()
                };

                let shift_key = modifiers.contains(KeyModifiers::SHIFT);

                return Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!(
                        "[FocusEvent type=\"{}\" bubbles={} cancelable={} relatedObject={} shiftKey={} keyCode={}]",
                        event_type, bubbles, cancelable, related_object, shift_key, key_code
                    ),
                )
                .into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `FocusEvent`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "FocusEvent"),
        Some(QName::new(Namespace::package("flash.events"), "Event").into()),
        Method::from_builtin(instance_init, "<FocusEvent instance initializer>", mc),
        Method::from_builtin(class_init, "<FocusEvent class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] = &[
        ("FOCUS_IN", "focusIn"),
        ("FOCUS_OUT", "focusOut"),
        ("KEY_FOCUS_CHANGE", "keyFocusChange"),
        ("MOUSE_FOCUS_CHANGE", "mouseFocusChange"),
    ];

    write.define_public_constant_string_class_traits(CONSTANTS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("direction", Some(direction), Some(set_direction)),
        (
            "isRelatedObjectInaccessible",
            Some(is_related_object_inaccessible),
            None,
        ),
        ("keyCode", Some(key_code), Some(set_key_code)),
        (
            "relatedObject",
            Some(related_object),
            Some(set_related_object),
        ),
        ("shiftKey", Some(shift_key), Some(set_shift_key)),
    ];

    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("toString", to_string)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
