//! `flash.display.SimpleButton` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::{Avm2Button, ButtonTracking, TDisplayObject};
use crate::vminterface::Instantiator;
use swf::ButtonState;

pub use crate::avm2::globals::flash::media::soundmixer::{
    get_sound_transform, set_sound_transform,
};
use crate::avm2::parameters::ParametersExt;

/// Implements `flash.display.SimpleButton`'s 'init' method. which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let new_do = Avm2Button::empty_button(&mut activation.context);

            new_do.post_instantiation(&mut activation.context, None, Instantiator::Avm2, false);
            this.init_display_object(&mut activation.context, new_do.into());

            let up_state = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Null)
                .as_object()
                .and_then(|o| o.as_display_object());
            new_do.set_state_child(&mut activation.context, ButtonState::UP, up_state);

            let over_state = args
                .get(1)
                .cloned()
                .unwrap_or(Value::Null)
                .as_object()
                .and_then(|o| o.as_display_object());
            new_do.set_state_child(&mut activation.context, ButtonState::OVER, over_state);

            let down_state = args
                .get(2)
                .cloned()
                .unwrap_or(Value::Null)
                .as_object()
                .and_then(|o| o.as_display_object());
            new_do.set_state_child(&mut activation.context, ButtonState::DOWN, down_state);

            let hit_state = args
                .get(3)
                .cloned()
                .unwrap_or(Value::Null)
                .as_object()
                .and_then(|o| o.as_display_object());
            new_do.set_state_child(&mut activation.context, ButtonState::HIT_TEST, hit_state);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `downState`'s getter.
pub fn get_down_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn
            .get_state_child(ButtonState::DOWN)
            .map(|state| state.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `downState`'s setter.
pub fn set_down_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|val| val.as_display_object());

        btn.set_state_child(&mut activation.context, ButtonState::DOWN, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `overState`'s getter.
pub fn get_over_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn
            .get_state_child(ButtonState::OVER)
            .map(|state| state.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `overState`'s setter.
pub fn set_over_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|val| val.as_display_object());

        btn.set_state_child(&mut activation.context, ButtonState::OVER, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `hitTestState`'s getter.
pub fn get_hit_test_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn
            .get_state_child(ButtonState::HIT_TEST)
            .map(|state| state.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `hitTestState`'s setter.
pub fn set_hit_test_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|val| val.as_display_object());

        btn.set_state_child(&mut activation.context, ButtonState::HIT_TEST, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `upState`'s getter.
pub fn get_up_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn
            .get_state_child(ButtonState::UP)
            .map(|state| state.object2())
            .unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `upState`'s setter.
pub fn set_up_state<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_object()
            .and_then(|val| val.as_display_object());

        btn.set_state_child(&mut activation.context, ButtonState::UP, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `trackAsMenu`'s getter
pub fn get_track_as_menu<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        return Ok((btn.button_tracking() == ButtonTracking::Menu).into());
    }

    Ok(Value::Undefined)
}

/// Implements `trackAsMenu`'s setter
pub fn set_track_as_menu<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        match args.get_bool(0) {
            true => btn.set_button_tracking(&mut activation.context, ButtonTracking::Menu),
            false => btn.set_button_tracking(&mut activation.context, ButtonTracking::Push),
        }
    }

    Ok(Value::Undefined)
}

/// Implements `enabled`'s getter
pub fn get_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn.enabled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `enabled`'s setter
pub fn set_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        btn.set_enabled(
            &mut activation.context,
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_boolean(),
        );
    }

    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s getter
pub fn get_use_hand_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn.use_hand_cursor().into());
    }

    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s setter
pub fn set_use_hand_cursor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_avm2_button())
    {
        btn.set_use_hand_cursor(
            &mut activation.context,
            args.get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_boolean(),
        );
    }

    Ok(Value::Undefined)
}
