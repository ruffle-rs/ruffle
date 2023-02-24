use crate::avm2::events::Event;
use crate::avm2::object::TObject as _;
use crate::avm2::Activation;
use crate::avm2::Object;
use crate::avm2::Value;
use crate::display_object::DisplayObject;
use crate::string::AvmString;
use std::cell::{Ref, RefMut};

/// This trait is implemented for each type that can appear in the signature
/// of a method annotated with `#[native]` (e.g. `bool`, `f64`, `Object`)
pub trait ExtractFromVm<'a, 'gc>: Sized {
    /// Attempts to extract `Self` from the provided `Value`.
    /// If the extraction cannot be performed (or would require a coercion),
    /// then this should return `None`.
    ///
    /// The provided `activation` should only be used for debugging,
    /// or calling a method that requires a `MutationContext`.
    /// Any coercions (e.g. `coerce_to_string`) should have already been
    /// performed by the time this method is called.
    fn extract_from(val: &'a Value<'gc>, activation: &mut Activation<'_, 'gc>) -> Option<Self>;
}

/// Allows writing `arg: DisplayObject<'gc>` in a `#[native]` method
impl<'a, 'gc> ExtractFromVm<'a, 'gc> for DisplayObject<'gc> {
    fn extract_from(val: &'a Value<'gc>, _activation: &mut Activation<'_, 'gc>) -> Option<Self> {
        val.as_object().and_then(|o| o.as_display_object())
    }
}

/// Allows writing `arg: AvmString<'gc>` in a `#[native]` method
impl<'a, 'gc> ExtractFromVm<'a, 'gc> for AvmString<'gc> {
    fn extract_from(val: &'a Value<'gc>, _activation: &mut Activation<'_, 'gc>) -> Option<Self> {
        if let Value::String(string) = val {
            Some(*string)
        } else {
            None
        }
    }
}

/// Allows writing `arg: f64` in a `#[native]` method
impl<'a, 'gc> ExtractFromVm<'a, 'gc> for f64 {
    fn extract_from(val: &'a Value<'gc>, _activation: &mut Activation<'_, 'gc>) -> Option<Self> {
        if let Value::Number(num) = val {
            Some(*num)
        } else {
            None
        }
    }
}

/// Allows writing `arg: bool` in a `#[native]` method
impl<'a, 'gc> ExtractFromVm<'a, 'gc> for bool {
    fn extract_from(val: &'a Value<'gc>, _activation: &mut Activation<'_, 'gc>) -> Option<Self> {
        if let Value::Bool(val) = val {
            Some(*val)
        } else {
            None
        }
    }
}

/// Allows writing `arg: Ref<'_, Event<'gc>>` in a `#[native]` method.
/// This is a little more cumbersome for the user than allowing `&Event<'gc>`,
/// but it avoids complicating the implementation.
impl<'a, 'gc> ExtractFromVm<'a, 'gc> for Ref<'a, Event<'gc>> {
    fn extract_from(
        val: &'a Value<'gc>,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Option<Ref<'a, Event<'gc>>> {
        val.as_object_ref().and_then(|obj| obj.as_event())
    }
}

/// Allows writing `arg: RefMut<'_, Event<'gc>>` in a `#[native]` method.
/// This is a little more cumbersome for the user than allowing `&Event<'gc>`,
/// but it avoids complicating the implementation.
impl<'a, 'gc> ExtractFromVm<'a, 'gc> for RefMut<'a, Event<'gc>> {
    fn extract_from(
        val: &'a Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<RefMut<'a, Event<'gc>>> {
        val.as_object_ref()
            .and_then(|obj| obj.as_event_mut(activation.context.gc_context))
    }
}

/// Allows writing `arg: Object<'gc>` in a `#[native]` method
impl<'a, 'gc> ExtractFromVm<'a, 'gc> for Object<'gc> {
    fn extract_from(
        val: &'a Value<'gc>,
        _activation: &mut Activation<'_, 'gc>,
    ) -> Option<Object<'gc>> {
        val.as_object()
    }
}

/// A helper trait to allow using both `Option<SomeNativeType>` and `SomeNativeType`
/// as the receiver (`this`) argument of a `#[native]` method.
pub trait ReceiverHelper<'a, 'gc>: Sized {
    // We take an `&Option<Value>` instead of a `Option<Object>` so that we can call
    // an `ExtractFromVm` impl without lifetime issues (it's impossible to turn a
    // `&'a Object` to an `&'a Value::Object(object)`).
    fn extract_from(
        val: &'a Option<Value<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Self>;
}

/// Allows writing `this: SomeNativeType` in a `#[native]` method, where `SomeNativeType`
/// is any type with an `ExtractFromVm` (that is, it can be used as `arg: SomeNativeType`).
/// If the function is called without a receiver (e.g. `Option<Object<'gc>>` is `None`),
/// the extraction fails.
impl<'a, 'gc, T> ReceiverHelper<'a, 'gc> for T
where
    T: ExtractFromVm<'a, 'gc>,
{
    fn extract_from(
        val: &'a Option<Value<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Self> {
        if let Some(val) = val {
            let extracted: Option<T> = ExtractFromVm::extract_from(val, activation);
            extracted
        } else {
            None
        }
    }
}

/// Allows writing `this: Option<SomeNativeType>` in a `#[native]` method, where `SomeNativeType`
/// is any type with an `ExtractFromVm` (that is, it can be used as `arg: SomeNativeType`).
/// If the function is called without a receiver (e.g. `Option<Object<'gc>>` is `None`),
/// then the `#[native]` function will be called with `None`.
impl<'a, 'gc, T> ReceiverHelper<'a, 'gc> for Option<T>
where
    T: ExtractFromVm<'a, 'gc>,
{
    fn extract_from(
        val: &'a Option<Value<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Self> {
        if let Some(val) = val {
            // If the function was called with a receiver, then try to extract
            // a value.
            let extracted: Option<T> = ExtractFromVm::extract_from(val, activation);
            // If the extraction failed, then treat this extraction as having failed
            // as well. For example, if the user writes `this: Option<DisplayObject>`,
            // and the function is called with a Boolean receiver (e.g. `true`), then
            // we want an error to be produced. We do *not* want to call the user's
            // function with `None` (since an invalid receiver is different from no
            // receiver at all).
            extracted.map(Some)
        } else {
            // If there's no receiver, then the extraction succeeds (the outer `Some`),
            // and we want to call the `#[native]` method with a value of `None` for `this`.
            Some(None)
        }
    }
}
