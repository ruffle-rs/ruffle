use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::error::{
    make_error_2007, make_error_2037, make_error_2097, make_error_2146, make_error_2174,
};
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::slots::flash_net_file_filter as file_filter_slots;
use crate::avm2::globals::slots::flash_net_url_request as url_request_slots;
use crate::avm2::object::{ByteArrayObject, DateObject, FileReference};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Avm2, Error, EventObject, TObject as _, Value};
use crate::backend::navigator::NavigationMethod;
use crate::backend::ui::FileFilter;
use crate::config::NetworkingAccessMode;
use crate::string::AvmString;
use indexmap::IndexMap;

pub use crate::avm2::object::file_reference_allocator;

pub fn get_creation_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let creation_date = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogSelection(ref selection) => {
            if let Some(time) = selection.creation_time() {
                DateObject::from_date_time(activation.context, time).into()
            } else {
                Value::Null
            }
        }
    };

    Ok(creation_date)
}

pub fn get_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let bytearray = match *this.file_reference() {
        FileReference::FileDialogSelection(ref selection) if this.loaded() => {
            let bytes = selection.contents();
            let storage = ByteArrayStorage::from_vec(activation.context, bytes.to_vec());
            ByteArrayObject::from_storage(activation.context, storage)
        }
        // Contrary to other getters `data` will return null instead of throwing.
        _ => return Ok(Value::Null),
    };

    Ok(bytearray.into())
}

pub fn get_modification_date<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let modification_date = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogSelection(ref selection) => {
            if let Some(time) = selection.modification_time() {
                DateObject::from_date_time(activation.context, time).into()
            } else {
                Value::Null
            }
        }
    };

    Ok(modification_date)
}

pub fn get_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let name = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogSelection(ref selection) => {
            let name = selection.file_name();
            AvmString::new_utf8(activation.gc(), name).into()
        }
    };

    Ok(name)
}

pub fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let size = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogSelection(ref selection) => selection.size().unwrap_or(0),
    };

    Ok(Value::Number(size as f64))
}

pub fn get_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let type_ = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogSelection(ref selection) => {
            let type_ = selection.file_type().unwrap_or_default();
            AvmString::new_utf8(activation.gc(), type_).into()
        }
    };

    Ok(type_)
}

pub fn browse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let mut filters = Vec::new();
    if let Some(obj) = args.try_get_object(0)
        && let Some(array_storage) = obj.as_array_storage()
    {
        for filter in array_storage.iter() {
            if let Some(Value::Object(obj)) = filter {
                let filefilter = activation
                    .avm2()
                    .classes()
                    .filefilter
                    .inner_class_definition();
                if !obj.is_of_type(filefilter) {
                    return Err(make_error_2097(activation));
                }

                let description = obj.get_slot(file_filter_slots::_DESCRIPTION);
                let extension = obj.get_slot(file_filter_slots::_EXTENSION);
                let mac_type = obj.get_slot(file_filter_slots::_MAC_TYPE);

                // The description and extension must be non-empty strings.
                match (description, extension) {
                    (Value::String(description), Value::String(extension))
                        if !description.is_empty() && !extension.is_empty() =>
                    {
                        let mac_type = match mac_type {
                            Value::String(mac_type) if !mac_type.is_empty() => {
                                Some(mac_type.to_string())
                            }
                            _ => None,
                        };

                        filters.push(FileFilter {
                            description: description.to_string(),
                            extensions: extension.to_string(),
                            mac_type,
                        });
                    }
                    _ => return Err(make_error_2097(activation)),
                }
            } else {
                return Err(make_error_2097(activation));
            }
        }
    }

    let dialog = activation.context.ui.display_file_open_dialog(filters);
    let result = match dialog {
        Some(dialog) => {
            let process = crate::loader::select_file_dialog_avm2(activation.context, this, dialog);

            activation.context.navigator.spawn_future(process);
            true
        }
        None => false,
    };

    Ok(result.into())
}

pub fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    // Somewhat unexpectedly, we don't need to load anything here, because
    // that already happened during browse() or save().

    let size = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogSelection(ref selection) => selection.size().unwrap_or(0),
    };

    let size = size as usize;

    let open_evt = EventObject::bare_default_event(activation.context, "open");
    Avm2::dispatch_event(activation.context, open_evt, this.into());

    let progress_evt = EventObject::progress_event(activation, "progress", 0, size);
    Avm2::dispatch_event(activation.context, progress_evt, this.into());

    let open_evt2 = EventObject::bare_default_event(activation.context, "open");
    Avm2::dispatch_event(activation.context, open_evt2, this.into());

    let progress_evt2 = EventObject::progress_event(activation, "progress", size, size);
    Avm2::dispatch_event(activation.context, progress_evt2, this.into());

    this.set_loaded(true);

    let complete_evt = EventObject::bare_default_event(activation.context, "complete");
    Avm2::dispatch_event(activation.context, complete_evt, this.into());

    Ok(Value::Undefined)
}

pub fn save_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();
    let byte_array = args
        .try_get_object(0)
        .expect("AS code ensures data is non-null");

    let mut byte_array = byte_array.as_bytearray_mut().unwrap();

    let mut data = Vec::new();
    byte_array.swap_storage_with(&mut data);
    // The ByteArray is now empty, and `data` stores the data previously
    // contained in it

    let file_name = args
        .try_get_string(1)
        .expect("AS code ensures defaultFileName is non-null");

    // Create and spawn dialog
    let dialog = activation.context.ui.display_file_save_dialog(
        file_name.to_string(),
        format!("Select location to save the file {file_name}"),
    );

    match dialog {
        Some(dialog) => {
            let process = crate::loader::save_file_dialog(activation.context, this, dialog, data);

            activation.context.navigator.spawn_future(process);
        }
        None => return Err(make_error_2174(activation)),
    }

    Ok(Value::Undefined)
}

pub fn upload_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_file_reference().unwrap();

    let request = args.get_object(activation, 0, "request")?;
    let data_field = args.get_string(activation, 1).to_string();

    // Uploading needs a file from browse(), but not load().
    let (file_name, data) = match *this.file_reference() {
        FileReference::None => return Err(make_error_2037(activation)),
        FileReference::FileDialogSelection(ref selection) => {
            (selection.file_name(), selection.contents().to_vec())
        }
    };

    if activation.context.allow_networking == NetworkingAccessMode::None {
        return Err(make_error_2146(activation, "FileReference.upload"));
    }

    let url = match request.get_slot(url_request_slots::_URL) {
        Value::Null => return Err(make_error_2007(activation, "url")),
        url => url.coerce_to_string(activation)?.to_string(),
    };
    // Uploads are always POSTs. With a POST request, URLVariables go in the
    // multipart body as separate fields, and any other data is appended to
    // the URL, as navigateToURL does. With a GET request, as Flash Player
    // does, the data is appended to the URL as a string, so URLVariables
    // are escaped as their toString() escapes them.
    let method = request
        .get_slot(url_request_slots::_METHOD)
        .coerce_to_string(activation)?;
    let method =
        NavigationMethod::from_method_str(&method).expect("URLRequest should have a valid method");
    let data_value = request.get_slot(url_request_slots::_DATA);
    let (url, fields) = match (method, data_value) {
        (NavigationMethod::Get, Value::Null | Value::Undefined) => (url, IndexMap::new()),
        (NavigationMethod::Get, data) => {
            let mut url = url;
            if !url.contains('?') {
                url.push('?');
            }
            url.push_str(&data.coerce_to_string(activation)?.to_string());
            (url, IndexMap::new())
        }
        (NavigationMethod::Post, data) => super::parse_data(activation, &url, &data)?,
    };

    let future = crate::loader::upload_file_avm2(
        activation.context,
        this,
        url,
        fields,
        data_field,
        file_name,
        data,
    );
    activation.context.navigator.spawn_future(future);

    Ok(Value::Undefined)
}
