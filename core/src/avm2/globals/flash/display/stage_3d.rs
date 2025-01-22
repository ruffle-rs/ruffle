use crate::avm2::globals::methods::flash_events_event_dispatcher as event_dispatcher_methods;
use crate::avm2::object::{Context3DObject, EventObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};
use ruffle_render::backend::Context3DProfile;

pub use crate::avm2::object::stage_3d_allocator;

const PROFILES_HIGH_TO_LOW: &[(&[u8], Context3DProfile)] = [
    (
        b"standardExtended".as_slice(),
        Context3DProfile::StandardExtended,
    ),
    (b"standard".as_slice(), Context3DProfile::Standard),
    (
        b"standardConstrained".as_slice(),
        Context3DProfile::StandardConstrained,
    ),
    (
        b"baselineExtended".as_slice(),
        Context3DProfile::BaselineExtended,
    ),
    (b"baseline".as_slice(), Context3DProfile::Baseline),
    (
        b"baselineConstrained".as_slice(),
        Context3DProfile::BaselineConstrained,
    ),
]
.as_slice();

pub fn request_context3d_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_object = this.as_object().unwrap();

    let this_stage3d = this_object.as_stage_3d().unwrap();
    let profiles = args.get_object(activation, 1, "profiles")?;
    let profiles = profiles.as_vector_storage().unwrap();

    let profile = PROFILES_HIGH_TO_LOW
        .iter()
        .find_map(|(profile, profile_enum)| {
            profiles
                .iter()
                .any(|p| {
                    if let Value::String(p) = p {
                        &*p == *profile
                    } else {
                        unreachable!()
                    }
                })
                .then_some(*profile_enum)
        })
        .unwrap();

    if this_stage3d.context3d().is_none() {
        let context = activation.context.renderer.create_context3d(profile)?;
        let context3d_obj = Context3DObject::from_context(activation, context, this_stage3d)?;
        this_stage3d.set_context3d(Some(context3d_obj), activation.gc());

        let event = EventObject::bare_default_event(activation.context, "context3DCreate");

        this.call_method(
            event_dispatcher_methods::DISPATCH_EVENT,
            &[event.into()],
            activation,
        )?;
    }

    Ok(Value::Undefined)
}

pub fn get_context_3d<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_stage_3d() {
        return Ok(this.context3d().map_or(Value::Null, |obj| obj.into()));
    }
    Ok(Value::Undefined)
}

pub fn get_visible<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_stage_3d() {
        return Ok(this.visible().into());
    }
    Ok(Value::Undefined)
}

pub fn set_visible<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_stage_3d() {
        this.set_visible(args.get_bool(0));
    }
    Ok(Value::Undefined)
}
