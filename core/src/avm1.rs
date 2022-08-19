#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod function;
#[macro_use]
pub mod property_decl;

pub mod activation;
mod callable_value;
pub mod debug;
pub mod error;
mod fscommand;
pub mod globals;
pub mod object;
pub mod property;
pub mod property_map;
pub mod runtime;
mod scope;
mod value;

#[cfg(test)]
mod tests;

pub use crate::avm1::activation::{Activation, ActivationIdentifier};
pub use crate::avm1::error::Error;
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::string::AvmString;
pub use globals::SystemPrototypes;
pub use object::array_object::ArrayObject;
pub use object::script_object::ScriptObject;
pub use object::sound_object::SoundObject;
pub use object::stage_object::StageObject;
pub use object::{Object, ObjectPtr, TObject};
use ruffle_render::bounding_box::BoundingBox;
use swf::Twips;
pub use value::Value;

#[macro_export]
macro_rules! avm_warn {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            log::warn!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            log::warn!($($arg)*)
        }
    )
}

#[macro_export]
macro_rules! avm_error {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            log::error!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            log::error!($($arg)*)
        }
    )
}

pub fn root_error_handler<'gc>(activation: &mut Activation<'_, 'gc, '_>, error: Error<'gc>) {
    match &error {
        Error::ThrownValue(value) => {
            let message = value
                .coerce_to_string(activation)
                .unwrap_or_else(|_| "undefined".into());
            activation.context.avm_trace(&message.to_utf8_lossy());
            // Continue execution without halting.
            return;
        }
        Error::InvalidSwf(swf_error) => {
            log::error!("{}: {}", error, swf_error);
        }
        _ => {
            log::error!("{}", error);
        }
    }
    activation.context.avm1.halt();
}

/// Starts dragging this display object, making it follow the cursor.
/// Runs via the `startDrag` method or `StartDrag` AVM1 action.
pub fn start_drag<'gc>(
    display_object: DisplayObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    args: &[Value<'gc>],
) {
    let lock_center = args
        .get(0)
        .map(|o| o.as_bool(activation.context.swf.version()))
        .unwrap_or(false);

    let offset = if lock_center {
        // The object's origin point is locked to the mouse.
        Default::default()
    } else {
        // The object moves relative to current mouse position.
        // Calculate the offset from the mouse to the object in world space.
        let (object_x, object_y) = display_object.local_to_global(Default::default());
        let (mouse_x, mouse_y) = *activation.context.mouse_position;
        (object_x - mouse_x, object_y - mouse_y)
    };

    let constraint = if args.len() > 1 {
        // Invalid values turn into 0.
        let mut x_min = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_min = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut x_max = args
            .get(3)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();
        let mut y_max = args
            .get(4)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)
            .map(|n| if n.is_finite() { n } else { 0.0 })
            .map(Twips::from_pixels)
            .unwrap_or_default();

        // Normalize the bounds.
        if x_max.get() < x_min.get() {
            std::mem::swap(&mut x_min, &mut x_max);
        }
        if y_max.get() < y_min.get() {
            std::mem::swap(&mut y_min, &mut y_max);
        }
        BoundingBox {
            valid: true,
            x_min,
            y_min,
            x_max,
            y_max,
        }
    } else {
        // No constraints.
        Default::default()
    };

    let drag_object = crate::player::DragObject {
        display_object,
        offset,
        constraint,
    };
    *activation.context.drag_object = Some(drag_object);
}
