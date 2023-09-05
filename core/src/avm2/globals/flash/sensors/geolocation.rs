use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::events::PermissionStatus;
use crate::string::AvmString;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Add current class instance to this struct to dispatch geo events later.
    let instances = activation.context.geolocation_instances;
    instances.set(this, &mut activation.context);

    // Early versions of Android didn't have permission request APIs
    // so one could just start listening to GeolocationEvents without
    // requesting a permission. We push it right when a Geolocation object
    // is created to address this issue.
    activation
        .context
        .geolocation
        .request_geolocation_permission();

    Ok(Value::Undefined)
}

pub fn get_permission_status<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let status = activation
        .context
        .geolocation
        .geolocation_permission_status();
    tracing::debug!("geolocation.rs: permission_status: {:?}", status.as_str());
    Ok(Value::String(AvmString::new_utf8(
        activation.context.gc_context,
        status.as_str(),
    )))
}

pub fn get_is_supported<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!(
        "geolocation.rs: get_is_supported: {:?}",
        activation.context.geolocation.is_geolocation_supported()
    );
    Ok(activation
        .context
        .geolocation
        .is_geolocation_supported()
        .into())
}

pub fn get_muted<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    tracing::debug!(
        "geolocation.rs: get_muted: permission_status = {:?}",
        activation
            .context
            .geolocation
            .geolocation_permission_status()
    );

    // On mobile devices `muted` field reflects a GPS sensor being on or off.
    // We can't check this on our current targets, so we return a status of
    // a permission.
    Ok(Value::Bool(!matches!(
        activation
            .context
            .geolocation
            .geolocation_permission_status(),
        PermissionStatus::Granted
    )))
}

pub fn set_requested_update_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut interval = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;

    // Interval should be greater or equal 0
    if interval < 0.0 {
        return Err(Error::from(
            "ArgumentError: The value specified for argument interval is invalid.",
        ));
    }

    // Interval CAN be NaN for some reason but we don't care about it
    if interval.is_nan() {
        return Ok(Value::Undefined);
    }

    // Sanely limit the interval
    if interval < 1000.0 {
        interval = 1000.0;
    }

    activation
        .context
        .geolocation
        .set_geolocation_update_interval(interval);
    Ok(Value::Undefined)
}
