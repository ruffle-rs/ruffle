//! `flash.display.SimpleButton` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{ClassObject, Object, StageObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::{Avm2Button, ButtonTracking, TDisplayObject};
use swf::ButtonState;

pub use crate::avm2::globals::flash::media::sound_mixer::{
    get_sound_transform, set_sound_transform,
};
use crate::avm2::parameters::ParametersExt;

pub fn simple_button_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    use crate::vminterface::Instantiator;

    let simplebutton_cls = activation
        .avm2()
        .classes()
        .simplebutton
        .inner_class_definition();

    let mut class_def = Some(class.inner_class_definition());
    let orig_class = class;
    while let Some(class) = class_def {
        if class == simplebutton_cls {
            let button = Avm2Button::empty_button(activation.context);
            // [NA] Buttons specifically need to PO'd
            button.post_instantiation(activation.context, None, Instantiator::Avm2, false);
            let display_object = button.into();
            let obj = StageObject::for_display_object(activation, display_object, orig_class)?;
            display_object.set_object2(activation.context, obj.into());
            return Ok(obj.into());
        }

        if let Some((movie, symbol)) = activation
            .context
            .library
            .avm2_class_registry()
            .class_symbol(class)
        {
            let child = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .instantiate_by_id(symbol, activation.context.gc_context)?;

            return initialize_for_allocator(activation, child, orig_class);
        }
        class_def = class.super_class();
    }
    unreachable!("A SimpleButton subclass should have SimpleButton in superclass chain");
}

/// Implements `flash.display.SimpleButton`'s 'init' method. which is called from the constructor
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(new_do) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        let up_state = args
            .try_get_object(activation, 0)
            .and_then(|o| o.as_display_object());
        if up_state.is_some() {
            new_do.set_state_child(activation.context, ButtonState::UP, up_state);
        }

        let over_state = args
            .try_get_object(activation, 1)
            .and_then(|o| o.as_display_object());
        if over_state.is_some() {
            new_do.set_state_child(activation.context, ButtonState::OVER, over_state);
        }

        let down_state = args
            .try_get_object(activation, 2)
            .and_then(|o| o.as_display_object());
        if down_state.is_some() {
            new_do.set_state_child(activation.context, ButtonState::DOWN, down_state);
        }

        let hit_state = args
            .try_get_object(activation, 3)
            .and_then(|o| o.as_display_object());
        if hit_state.is_some() {
            new_do.set_state_child(activation.context, ButtonState::HIT_TEST, hit_state);
        }

        // This performs the child state construction.
        new_do.construct_frame(activation.context);
    } else {
        unreachable!();
    }

    Ok(Value::Undefined)
}

/// Implements `downState`'s getter.
pub fn get_down_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .try_get_object(activation, 0)
            .and_then(|val| val.as_display_object());

        btn.set_state_child(activation.context, ButtonState::DOWN, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `overState`'s getter.
pub fn get_over_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .try_get_object(activation, 0)
            .and_then(|val| val.as_display_object());

        btn.set_state_child(activation.context, ButtonState::OVER, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `hitTestState`'s getter.
pub fn get_hit_test_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .try_get_object(activation, 0)
            .and_then(|val| val.as_display_object());

        btn.set_state_child(activation.context, ButtonState::HIT_TEST, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `upState`'s getter.
pub fn get_up_state<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        let new_state = args
            .try_get_object(activation, 0)
            .and_then(|val| val.as_display_object());

        btn.set_state_child(activation.context, ButtonState::UP, new_state);
    }

    Ok(Value::Undefined)
}

/// Implements `trackAsMenu`'s getter
pub fn get_track_as_menu<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        return Ok((btn.button_tracking() == ButtonTracking::Menu).into());
    }

    Ok(Value::Undefined)
}

/// Implements `trackAsMenu`'s setter
pub fn set_track_as_menu<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        match args.get_bool(0) {
            true => btn.set_button_tracking(ButtonTracking::Menu),
            false => btn.set_button_tracking(ButtonTracking::Push),
        }
    }

    Ok(Value::Undefined)
}

/// Implements `enabled`'s getter
pub fn get_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn.enabled().into());
    }

    Ok(Value::Undefined)
}

/// Implements `enabled`'s setter
pub fn set_enabled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        btn.set_enabled(activation.context, args.get_bool(0));
    }

    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s getter
pub fn get_use_hand_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        return Ok(btn.use_hand_cursor().into());
    }

    Ok(Value::Undefined)
}

/// Implements `useHandCursor`'s setter
pub fn set_use_hand_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(btn) = this
        .as_display_object()
        .and_then(|this| this.as_avm2_button())
    {
        btn.set_use_hand_cursor(args.get_bool(0));
    }

    Ok(Value::Undefined)
}
