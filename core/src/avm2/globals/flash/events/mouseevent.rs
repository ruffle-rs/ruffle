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
use swf::Twips;

/// Implements `flash.events.MouseEvent`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        // Get up to three arguments
        let event_args = &args[..(std::cmp::min(args.len(), 2))];
        activation.super_init(this, event_args)?;
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            // This is technically duplicative of `Event`'s initializer, but
            // we have different default parameters.
            // TODO: When we get the ability to store parameter data on
            // builtin methods, we should remove these lines.
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

            let local_x = args
                .get(3)
                .cloned()
                .unwrap_or_else(|| f64::NAN.into())
                .coerce_to_number(activation)?;
            let local_y = args
                .get(4)
                .cloned()
                .unwrap_or_else(|| f64::NAN.into())
                .coerce_to_number(activation)?;
            let related_object = args
                .get(5)
                .cloned()
                .unwrap_or(Value::Null)
                .as_object()
                .and_then(|o| o.as_display_object())
                .and_then(|o| o.as_interactive());
            let ctrl_key = args
                .get(6)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let alt_key = args
                .get(7)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let shift_key = args
                .get(8)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let button_down = args
                .get(9)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();
            let delta = args
                .get(10)
                .cloned()
                .unwrap_or_else(|| 0.into())
                .coerce_to_i32(activation)?;

            let mut modifiers = KeyModifiers::default();
            if ctrl_key {
                modifiers.insert(KeyModifiers::CTRL);
            }

            if alt_key {
                modifiers.insert(KeyModifiers::ALT);
            }

            if shift_key {
                modifiers.insert(KeyModifiers::SHIFT);
            }

            evt.set_event_data(EventData::Mouse {
                local_x,
                local_y,
                movement_x: 0.0,
                movement_y: 0.0,
                related_object,
                modifiers,
                button_down,
                delta,
            });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.events.MouseEvent`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `altKey`'s getter.
pub fn alt_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::ALT).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `altKey`'s setter.
pub fn set_alt_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::ALT);
                } else {
                    modifiers.remove(KeyModifiers::ALT);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `commandKey`'s getter.
pub fn command_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::COMMAND).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `commandKey`'s setter.
pub fn set_command_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::COMMAND);
                } else {
                    modifiers.remove(KeyModifiers::COMMAND);
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `ctrlKey`/`controlKey`'s getter.
pub fn control_key<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { modifiers, .. } = evt.event_data() {
                return Ok(modifiers.contains(KeyModifiers::CTRL).into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `ctrlKey`/`controlKey`'s setter.
pub fn set_control_key<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { modifiers, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                if value {
                    modifiers.insert(KeyModifiers::CTRL);
                } else {
                    modifiers.remove(KeyModifiers::CTRL);
                }
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
            if let EventData::Mouse { modifiers, .. } = evt.event_data() {
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
            if let EventData::Mouse { modifiers, .. } = evt.event_data_mut() {
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

/// Implements `buttonDown`'s getter.
pub fn button_down<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { button_down, .. } = evt.event_data() {
                return Ok(Value::Bool(*button_down));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `buttonDown`'s setter.
pub fn set_button_down<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { button_down, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_boolean();

                *button_down = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `delta`'s getter.
pub fn delta<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { delta, .. } = evt.event_data() {
                return Ok(Value::Integer(*delta));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `delta`'s setter.
pub fn set_delta<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { delta, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_i32(activation)?;

                *delta = value;
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

/// Implements `localX`'s getter.
pub fn local_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { local_x, .. } = evt.event_data() {
                return Ok(Value::Number(*local_x));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `localX`'s setter.
pub fn set_local_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { local_x, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_number(activation)?;

                *local_x = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `localY`'s getter.
pub fn local_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { local_y, .. } = evt.event_data() {
                return Ok(Value::Number(*local_y));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `localY`'s setter.
pub fn set_local_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { local_y, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_number(activation)?;

                *local_y = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `movementX`'s getter.
pub fn movement_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { movement_x, .. } = evt.event_data() {
                return Ok(Value::Number(*movement_x));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `movementX`'s setter.
pub fn set_movement_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { movement_x, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_number(activation)?;

                *movement_x = value;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `movementY`'s getter.
pub fn movement_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { movement_y, .. } = evt.event_data() {
                return Ok(Value::Number(*movement_y));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `movementY`'s setter.
pub fn set_movement_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut evt) = this.as_event_mut(activation.context.gc_context) {
            if let EventData::Mouse { movement_y, .. } = evt.event_data_mut() {
                let value = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_number(activation)?;

                *movement_y = value;
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
            if let EventData::Mouse { related_object, .. } = evt.event_data() {
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
            if let EventData::Mouse { related_object, .. } = evt.event_data_mut() {
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

/// Implements `stageX`'s getter.
pub fn stage_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { local_x, .. } = evt.event_data() {
                if let Some(target) = evt.target().and_then(|t| t.as_display_object()) {
                    let as_twips = Twips::from_pixels(*local_x);
                    let xformed = target.local_to_global((as_twips, Twips::ZERO)).0;

                    return Ok(Value::Number(xformed.to_pixels()));
                } else {
                    return Ok(Value::Number(*local_x * 0.0));
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `stageY`'s getter.
pub fn stage_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            if let EventData::Mouse { local_y, .. } = evt.event_data() {
                if let Some(target) = evt.target().and_then(|t| t.as_display_object()) {
                    let as_twips = Twips::from_pixels(*local_y);
                    let xformed = target.local_to_global((Twips::ZERO, as_twips)).1;

                    return Ok(Value::Number(xformed.to_pixels()));
                } else {
                    return Ok(Value::Number(*local_y * 0.0));
                }
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
            if let EventData::Mouse {
                local_x,
                local_y,
                related_object,
                modifiers,
                button_down,
                delta,
                ..
            } = event.event_data()
            {
                let event_type = event.event_type();
                let bubbles = event.is_bubbling();
                let cancelable = event.is_cancelable();
                let phase = event.phase() as u32;

                let (stage_x, stage_y) =
                    if let Some(target) = event.target().and_then(|t| t.as_display_object()) {
                        let (x, y) = target.local_to_global((
                            Twips::from_pixels(*local_x),
                            Twips::from_pixels(*local_y),
                        ));

                        (x.to_pixels(), y.to_pixels())
                    } else {
                        (local_x * 0.0, local_y * 0.0)
                    };

                let related_object = if let Some(related_object) =
                    related_object.and_then(|ro| ro.as_displayobject().object2().as_object())
                {
                    related_object
                        .to_string(activation.context.gc_context)?
                        .coerce_to_string(activation)?
                } else {
                    "null".into()
                };

                let ctrl_key = modifiers.contains(KeyModifiers::CTRL);
                let alt_key = modifiers.contains(KeyModifiers::ALT);
                let shift_key = modifiers.contains(KeyModifiers::SHIFT);

                return Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!(
                        "[MouseEvent type=\"{}\" bubbles={} cancelable={} eventPhase={} localX={} localY={} stageX={} stageY={} relatedObject={} ctrlKey={} altKey={} shiftKey={} buttonDown={} delta={}]",
                        event_type, bubbles, cancelable, phase, local_x, local_y, stage_x, stage_y, related_object, ctrl_key, alt_key, shift_key, button_down, delta
                    ),
                )
                .into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `MouseEvent`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.events"), "MouseEvent"),
        Some(QName::new(Namespace::package("flash.events"), "Event").into()),
        Method::from_builtin(instance_init, "<MouseEvent instance initializer>", mc),
        Method::from_builtin(class_init, "<MouseEvent class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const CONSTANTS: &[(&str, &str)] = &[
        ("CLICK", "click"),
        ("CONTEXT_MENU", "contextMenu"),
        ("DOUBLE_CLICK", "doubleClick"),
        ("MIDDLE_CLICK", "middleClick"),
        ("MIDDLE_MOUSE_DOWN", "middleMouseDown"),
        ("MIDDLE_MOUSE_UP", "middleMouseUp"),
        ("MOUSE_DOWN", "mouseDown"),
        ("MOUSE_MOVE", "mouseMove"),
        ("MOUSE_OUT", "mouseOut"),
        ("MOUSE_OVER", "mouseOver"),
        ("MOUSE_UP", "mouseUp"),
        ("MOUSE_WHEEL", "mouseWheel"),
        ("RELEASE_OUTSIDE", "releaseOutside"),
        ("RIGHT_CLICK", "rightClick"),
        ("RIGHT_MOUSE_DOWN", "rightMouseDown"),
        ("RIGHT_MOUSE_UP", "rightMouseUp"),
        ("ROLL_OUT", "rollOut"),
        ("ROLL_OVER", "rollOver"),
    ];

    write.define_public_constant_string_class_traits(CONSTANTS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("altKey", Some(alt_key), Some(set_alt_key)),
        ("commandKey", Some(command_key), Some(set_command_key)),
        ("controlKey", Some(control_key), Some(set_control_key)),
        ("ctrlKey", Some(control_key), Some(set_control_key)),
        ("shiftKey", Some(shift_key), Some(set_shift_key)),
        ("buttonDown", Some(button_down), Some(set_button_down)),
        ("delta", Some(delta), Some(set_delta)),
        (
            "isRelatedObjectInaccessible",
            Some(is_related_object_inaccessible),
            None,
        ),
        ("localX", Some(local_x), Some(set_local_x)),
        ("localY", Some(local_y), Some(set_local_y)),
        ("movementX", Some(movement_x), Some(set_movement_x)),
        ("movementY", Some(movement_y), Some(set_movement_y)),
        (
            "relatedObject",
            Some(related_object),
            Some(set_related_object),
        ),
        ("stageX", Some(stage_x), None),
        ("stageY", Some(stage_y), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("toString", to_string)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
