//! `flash.net.URLStream` native function definitions

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2029;
use crate::avm2::globals::flash::display::loader::request_from_url_request;
use crate::avm2::globals::slots::flash_net_url_stream as url_stream_slots;
use crate::avm2::object::TObject as _;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::{Error, Object};

/// Native function definition for `URLStream.load`
pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let request = args.get_object(activation, 0, "request")?;

    spawn_fetch(activation, this, request)
}

/// Native function definition for `URLStream.close`
///
/// Per the Adobe spec, `close()` "immediately closes the stream and cancels
/// the download operation" and throws an `IOError` if the stream was not
/// open. We model "open" via the `_connected` slot: it is set synchronously
/// in `load()` and cleared on completion, error, or a previous `close()`.
///
/// Closing sets the `_closed` flag, which the async fetch loop polls after
/// every chunk; once it sees the flag it stops dispatching further events.
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let connected = matches!(
        this.get_slot(url_stream_slots::_CONNECTED),
        Value::Bool(true)
    );
    if !connected {
        return Err(make_error_2029(activation));
    }

    this.set_slot(url_stream_slots::_CLOSED, Value::Bool(true), activation)?;
    this.set_slot(url_stream_slots::_CONNECTED, Value::Bool(false), activation)?;

    // Per the Adobe spec: "No data can be read from the stream after the
    // close() method is called." Clear the internal buffer so subsequent
    // reads see `bytesAvailable == 0` and throw EOFError. This matches
    // `Socket.close()` in Ruffle, which also clears its read buffer.
    if let Some(bytearray) = this
        .get_slot(url_stream_slots::_DATA)
        .as_object()
        .and_then(|o| o.as_bytearray_object())
    {
        bytearray.storage_mut().clear();
    }

    Ok(Value::Undefined)
}

fn spawn_fetch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    stream_object: Object<'gc>,
    url_request: Object<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let request = request_from_url_request(activation, url_request)?;

    // Reset per-load state. A `close()` issued before this call must not
    // cancel the new fetch, and the byte counters start fresh. `_connected`
    // is set true synchronously so that a `close()` issued between now and
    // the arrival of the first response chunk is treated as cancelling an
    // open stream (rather than throwing error #2029).
    stream_object.set_slot(url_stream_slots::_CLOSED, Value::Bool(false), activation)?;
    stream_object.set_slot(url_stream_slots::_CONNECTED, Value::Bool(true), activation)?;

    let future = crate::loader::load_data_into_url_stream(
        activation.context,
        stream_object.as_script_object().unwrap(),
        request,
    );
    activation.context.navigator.spawn_future(future);
    Ok(Value::Undefined)
}
