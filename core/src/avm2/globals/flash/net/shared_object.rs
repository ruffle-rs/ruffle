//! `flash.net.SharedObject` builtin/prototype

use crate::avm2::error::{make_error_2130, make_error_2134, make_error_2146};
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::{ScriptObject, SharedObjectObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, Value};
use crate::config::NetworkingAccessMode;
use crate::shared_object::local_shared_object_key;
use crate::string::AvmString;
use crate::{avm2_stub_getter, avm2_stub_method, avm2_stub_setter};
use flash_lso::types::{AMFVersion, Lso};
use ruffle_macros::istr;

fn new_lso<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &str,
    data: Object<'gc>,
) -> Result<Lso, Error<'gc>> {
    let mut elements = Vec::new();
    crate::avm2::amf::recursive_serialize(
        activation,
        data,
        &mut elements,
        None,
        AMFVersion::AMF3,
        &mut Default::default(),
    )?;
    Ok(Lso::new(
        elements,
        name.split('/')
            .next_back()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
        AMFVersion::AMF3,
    ))
}

pub fn get_local<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get_string_non_null(activation, 0, "name")?;
    let local_path = args.try_get_string(1);
    let local_path = local_path.as_ref().map(|p| p.to_utf8_lossy());
    let secure = args.get_bool(2);

    if activation.context.allow_networking == NetworkingAccessMode::None {
        return Err(make_error_2146(activation, "SharedObject.getLocal"));
    }

    let Some(full_name) = local_shared_object_key(
        activation.caller_movie_or_root(),
        &name.to_utf8_lossy(),
        local_path.as_deref(),
        secure,
    ) else {
        return Err(make_error_2134(activation));
    };

    // Check if this is referencing an existing shared object
    if let Some(so) = activation.context.avm2_shared_objects.get(&full_name) {
        return Ok((*so).into());
    }

    let mut data = None;

    // Load the data object from storage if it existed prior
    if let Some(saved) = activation.context.storage.get(&full_name)
        && let Ok(lso) = flash_lso::read::Reader::default().parse(&saved)
    {
        data = crate::avm2::amf::deserialize_lso(activation, &lso)?.into();
    }

    let data = if let Some(data) = data {
        data
    } else {
        // No data; create a fresh data object.
        ScriptObject::new_object(activation.context)
    };

    let created_shared_object =
        SharedObjectObject::from_data_and_name(activation, data, full_name.clone());

    activation
        .context
        .avm2_shared_objects
        .insert(full_name, created_shared_object);

    Ok(created_shared_object.into())
}

pub fn get_data<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let shared_object = this.as_shared_object().unwrap();

    Ok(shared_object.data().into())
}

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let shared_object = this.as_shared_object().unwrap();

    let min_disk_space = args.get_i32(0);

    flush_impl(activation, shared_object, min_disk_space).map(Value::from)
}

pub fn flush_impl<'gc>(
    activation: &mut Activation<'_, 'gc>,
    shared_object: SharedObjectObject<'gc>,
    _min_disk_space: i32,
) -> Result<AvmString<'gc>, Error<'gc>> {
    let data = shared_object.data();
    let name = shared_object.name();

    let mut lso = new_lso(activation, name, data)?;
    // Flash does not write empty LSOs to disk
    if lso.body.is_empty() {
        Ok(istr!("flushed"))
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        if activation.context.storage.put(name, &bytes) {
            Ok(istr!("flushed"))
        } else {
            Err(make_error_2130(activation))
        }
    }
    // FIXME - We should dispatch a NetStatusEvent after this function returns
}

pub fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let shared_object = this.as_shared_object().unwrap();

    let data = shared_object.data();
    let name = shared_object.name();

    let mut lso = new_lso(activation, name, data)?;
    // Flash returns 0 for empty LSOs, but the actual number of bytes (including the header) otherwise
    if lso.body.is_empty() {
        Ok(0.into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        Ok(Value::from_usize_lossy(bytes.len()))
    }
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.net.SharedObject", "close");
    Ok(Value::Undefined)
}

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let shared_object = this.as_shared_object().unwrap();

    // Clear the local data object.
    shared_object.reset_data(activation.context);

    // Delete data from storage backend.
    let name = shared_object.name();
    activation.context.storage.remove_key(name);

    Ok(Value::Undefined)
}

pub fn get_object_encoding<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.net.SharedObject", "objectEncoding");
    Ok(0.into())
}

pub fn set_object_encoding<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.net.SharedObject", "objectEncoding");
    Ok(Value::Undefined)
}
