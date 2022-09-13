//! `flash.utils.Timer` native methods

use crate::avm2::activation::Activation;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::{Error, Object};
use crate::timer::TimerCallback;

/// Implements `Timer.stop`
pub fn stop<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut this = this.expect("`this` should be set in native method!");
    let id = this
        .get_property(
            &Multiname::new(Namespace::Private("".into()), "_timerId"),
            activation,
        )
        .unwrap()
        .coerce_to_i32(activation)?;

    if id != -1 {
        activation.context.timers.remove(id);
        this.set_property(
            &Multiname::new(Namespace::Private("".into()), "_timerId"),
            (-1).into(),
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `Timer.start`
pub fn start<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut this = this.expect("`this` should be set in native method!");
    let id = this
        .get_property(
            &Multiname::new(Namespace::Private("".into()), "_timerId"),
            activation,
        )
        .unwrap()
        .coerce_to_i32(activation)?;

    let delay = this
        .get_property(
            &Multiname::new(Namespace::Private("".into()), "_delay"),
            activation,
        )
        .unwrap()
        .coerce_to_number(activation)?;

    if id == -1 {
        let on_update = this
            .get_property(
                &Multiname::new(Namespace::Private("".into()), "onUpdate"),
                activation,
            )?
            .coerce_to_object(activation)?;
        let id = activation.context.timers.add_timer(
            TimerCallback::Avm2Callback {
                closure: on_update,
                params: vec![],
            },
            delay as _,
            false,
        );
        this.set_property(
            &Multiname::new(Namespace::Private("".into()), "_timerId"),
            id.into(),
            activation,
        )?;
    }
    Ok(Value::Undefined)
}
