//! `flash.events.Event` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::events::Event;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::AvmString;
use crate::avm2::Error;
use ruffle_macros::native;
use std::cell::{Ref, RefMut};

pub use crate::avm2::object::event_allocator;

#[native]
pub fn init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    mut this: RefMut<'_, Event<'gc>>,
    event_type: AvmString<'gc>,
    bubbles: bool,
    cancelable: bool,
) -> Result<Value<'gc>, Error<'gc>> {
    this.set_event_type(event_type);
    this.set_bubbles(bubbles);
    this.set_cancelable(cancelable);
    Ok(Value::Undefined)
}

/// Implements `bubbles` property's getter
pub fn get_bubbles<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_bubbling().into());
    }

    Ok(Value::Undefined)
}

/// Implements `cancelable` property's getter
pub fn get_cancelable<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(evt) = this.unwrap().as_event() {
        return Ok(evt.is_cancelable().into());
    }

    Ok(Value::Undefined)
}

/// Implements `type` property's getter
#[native]
pub fn get_type<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Ref<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.event_type().into())
}

/// Implements `target` property's getter
#[native]
pub fn get_target<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Ref<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.target().map(|o| o.into()).unwrap_or(Value::Null))
}

/// Implements `currentTarget` property's getter
#[native]
pub fn get_current_target<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Ref<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .current_target()
        .map(|o| o.into())
        .unwrap_or(Value::Null))
}

/// Implements `eventPhase` property's getter
#[native]
pub fn get_event_phase<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Ref<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((this.phase() as u32).into())
}

/// Implements `isDefaultPrevented`
#[native]
pub fn is_default_prevented<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Ref<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.is_cancelled().into())
}

/// Implements `preventDefault`
#[native]
pub fn prevent_default<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    mut this: RefMut<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    this.cancel();
    Ok(Value::Undefined)
}

/// Implements `stopPropagation`
#[native]
pub fn stop_propagation<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    mut this: RefMut<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    this.stop_propagation();
    Ok(Value::Undefined)
}

/// Implements `stopImmediatePropagation`
#[native]
pub fn stop_immediate_propagation<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    mut this: RefMut<'_, Event<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    this.stop_immediate_propagation();
    Ok(Value::Undefined)
}
