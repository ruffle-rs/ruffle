use crate::avm2::error::{Error2006Type, make_error_2006};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Value};
use ruffle_render::backend::Context3DProfile;

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

    // Several SWFS (the examples from the Context3D documentation, and the
    // Starling framework) rely on the `context3DCreate` being fired
    // asynchronously - they initialize variables after the call to
    // `requestContext3D`, and then use those variables in the event handler.

    // Our context3D creation is synchronous, so we need to delay it. Set the
    // parameters that the context3d was requested with here, then actually
    // create the context3d at the end of this frame in
    // `frame_lifecycle::run_all_phases_avm2` and dispatch the context3DCreate
    // event.

    this_stage3d.set_requesting_context3d(activation.gc(), profile);

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

pub fn get_x<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_stage_3d() {
        return Ok(this.x().into());
    }

    Ok(Value::Undefined)
}

pub fn set_x<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_stage_3d() {
        let x = args.get_f64(0);

        // The docs say that anything less than -8191 will throw an error,
        // but it actually starts at -8192 for some reason.
        if x.is_nan() || x < -8192.0 || x > 8191.0 {
            return Err(make_error_2006(activation, Error2006Type::ArgumentError));
        }

        this.set_x(x);
    }

    Ok(Value::Undefined)
}

pub fn get_y<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_stage_3d() {
        return Ok(this.y().into());
    }

    Ok(Value::Undefined)
}

pub fn set_y<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_stage_3d() {
        let y = args.get_f64(0);

        // The docs say that anything less than -8191 will throw an error,
        // but it actually starts at -8192 for some reason.
        if y.is_nan() || y < -8192.0 || y > 8191.0 {
            return Err(make_error_2006(activation, Error2006Type::ArgumentError));
        }

        this.set_y(y);
    }

    Ok(Value::Undefined)
}
