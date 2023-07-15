use crate::avm2::object::Context3DObject;
use crate::avm2::object::TObject;

use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, Value};

pub use crate::avm2::object::stage_3d_allocator;

pub fn request_context3d_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_stage3d = this.as_stage_3d().unwrap();
    if this_stage3d.context3d().is_none() {
        let context = activation.context.renderer.create_context3d()?;
        let context3d_obj = Context3DObject::from_context(activation, context)?;
        this_stage3d.set_context3d(context3d_obj, activation.context.gc_context);

        let event = activation
            .avm2()
            .classes()
            .event
            .construct(activation, &["context3DCreate".into()])?;

        // FIXME - fire this at least one frame later,
        // since some seems to expect this (e.g. the adobe triangle example)
        this.call_public_property("dispatchEvent", &[event.into()], activation)?;
    }

    Ok(Value::Undefined)
}

pub fn get_context_3d<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_stage_3d() {
        return Ok(this.context3d().map_or(Value::Null, |obj| obj.into()));
    }
    Ok(Value::Undefined)
}

pub fn get_visible<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_stage_3d() {
        return Ok(this.visible().into());
    }
    Ok(Value::Undefined)
}

pub fn set_visible<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_stage_3d() {
        this.set_visible(args.get_bool(0));
    }
    Ok(Value::Undefined)
}
