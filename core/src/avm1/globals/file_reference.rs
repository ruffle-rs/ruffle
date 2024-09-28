use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::FunctionObject;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Executable, NativeObject, Object, ScriptObject, TObject, Value};
use crate::avm1_stub;
use crate::backend::ui::{FileDialogResult, FileFilter};
use crate::string::{AvmString, StringContext};
use gc_arena::{Collect, GcCell};
use url::Url;

// There are two undocumented functions in FileReference: convertToPPT and deleteConvertedPPT.
// Until further reason is given, they will be unimplemented.
// See:
// ASSetPropFlags(flash.net.FileReference.prototype, null, 6, 1);
// for(var k in flash.net.FileReference.prototype) {
// 	trace(k);
// }

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct FileReferenceObject<'gc>(GcCell<'gc, FileReferenceData<'gc>>);

impl<'gc> FileReferenceObject<'gc> {
    pub fn init_from_dialog_result(
        &self,
        activation: &mut Activation<'_, 'gc>,
        dialog_result: &dyn FileDialogResult,
    ) {
        let mut s = self.0.write(activation.gc());
        s.is_initialised = true;

        let date_proto = activation.context.avm1.prototypes().date_constructor;
        if let Some(creation_time) = dialog_result.creation_time() {
            if let Ok(Value::Object(obj)) = date_proto.construct(
                activation,
                &[(creation_time.timestamp_millis() as f64).into()],
            ) {
                s.creation_date = Some(obj);
            }
        }

        if let Some(modification_time) = dialog_result.modification_time() {
            if let Ok(Value::Object(obj)) = date_proto.construct(
                activation,
                &[(modification_time.timestamp_millis() as f64).into()],
            ) {
                s.modification_date = Some(obj);
            }
        }

        s.file_type = dialog_result.file_type();
        s.name = dialog_result.file_name();
        s.size = dialog_result.size();
        s.creator = dialog_result.creator();
        s.data = dialog_result.contents().to_vec();
    }
}

#[derive(Default, Clone, Collect)]
#[collect(no_drop)]
pub struct FileReferenceData<'gc> {
    /// Has this object been initialised from a dialog
    is_initialised: bool,

    creation_date: Option<Object<'gc>>,
    creator: Option<String>,
    modification_date: Option<Object<'gc>>,
    name: Option<String>,
    post_data: String,
    size: Option<u64>,
    file_type: Option<String>,

    /// The contents of the referenced file
    /// We track this here so that it can be referenced in FileReference.upload
    data: Vec<u8>,
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "creationDate" => property(creation_date; DONT_ENUM);
    "creator" => property(creator; DONT_ENUM);
    "modificationDate" => property(modification_date; DONT_ENUM);
    "name" => property(name; DONT_ENUM);
    "postData" => property(post_data, set_post_data; DONT_ENUM);
    "size" => property(size; DONT_ENUM);
    "type" => property(file_type; DONT_ENUM);
    "browse" => method(browse; DONT_ENUM);
    "cancel" => method(cancel; DONT_ENUM);
    "download" => method(download; DONT_ENUM);
    "upload" => method(upload; DONT_ENUM);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {};

pub fn creation_date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(file_ref
            .0
            .read()
            .creation_date
            .map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn creator<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(file_ref
            .0
            .read()
            .creator
            .as_ref()
            .map_or(Value::Undefined, |x| {
                AvmString::new_utf8(activation.context.gc_context, x).into()
            }));
    }

    Ok(Value::Undefined)
}

pub fn modification_date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(file_ref
            .0
            .read()
            .modification_date
            .map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(file_ref
            .0
            .read()
            .name
            .as_ref()
            .map_or(Value::Undefined, |x| {
                AvmString::new_utf8(activation.context.gc_context, x).into()
            }));
    }

    Ok(Value::Undefined)
}

pub fn post_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(AvmString::new_utf8(
            activation.context.gc_context,
            file_ref.0.read().post_data.clone(),
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn set_post_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let post_data = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    if let NativeObject::FileReference(file_ref) = this.native() {
        file_ref.0.write(activation.context.gc_context).post_data = post_data.to_string();
    }

    Ok(Value::Undefined)
}

pub fn size<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(file_ref
            .0
            .read()
            .size
            .map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn file_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(file_ref
            .0
            .read()
            .file_type
            .as_ref()
            .map_or(Value::Undefined, |x| {
                AvmString::new_utf8(activation.context.gc_context, x).into()
            }));
    }

    Ok(Value::Undefined)
}

pub fn browse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let file_filters = match args.get(0) {
        Some(Value::Object(array)) => {
            // Array of filter objects.
            let length = array.length(activation)?;

            // Empty array is not allowed
            if length == 0 {
                return Ok(false.into());
            }

            let mut results = Vec::with_capacity(length as usize);

            for i in 0..length {
                if let Value::Object(element) = array.get_element(activation, i) {
                    let mac_type =
                        if let Some(val) = element.get_local_stored("macType", activation, false) {
                            Some(val.coerce_to_string(activation)?.to_string())
                        } else {
                            None
                        };

                    let description = element.get_local_stored("description", activation, false);
                    let extension = element.get_local_stored("extension", activation, false);

                    if let (Some(description), Some(extension)) = (description, extension) {
                        let description = description.coerce_to_string(activation)?.to_string();

                        let extensions = extension.coerce_to_string(activation)?.to_string();

                        // Empty strings are not allowed for desc / extension
                        if description.is_empty() || extensions.is_empty() {
                            return Ok(false.into());
                        }

                        results.push(FileFilter {
                            description,
                            extensions,
                            mac_type,
                        });
                    } else {
                        return Ok(false.into());
                    }
                } else {
                    return Err(Error::ThrownValue("Unexpected filter value".into()));
                }
            }

            results
        }
        None => Vec::new(),
        _ => return Ok(Value::Undefined),
    };

    let dialog = activation.context.ui.display_file_open_dialog(file_filters);

    let result = match dialog {
        Some(dialog) => {
            let process = activation.context.load_manager.select_file_dialog(
                activation.context.player.clone(),
                this,
                dialog,
            );

            activation.context.navigator.spawn_future(process);
            true
        }
        None => false,
    };

    Ok(result.into())
}

pub fn cancel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "FileReference", "cancel");
    Ok(Value::Undefined)
}

pub fn download<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(url) = args.first() {
        let url_string = url.coerce_to_string(activation)?.to_string();

        // Invalid domain should bail out with false
        let url = match Url::parse(&url_string) {
            Ok(url) => url,
            Err(_) => return Ok(false.into()),
        };

        let file_name = match args.get(1) {
            Some(file_name) => file_name.coerce_to_string(activation)?.to_string(),
            None => {
                // Try to get the end of the path as a file name, if we can't bail and return false
                match url.path().split('/').last() {
                    Some(path_end) => path_end.to_string(),
                    None => return Ok(false.into()),
                }
            }
        };

        let domain = url.domain().unwrap_or("<unknown domain>").to_string();

        // Create and spawn dialog
        let dialog = activation.context.ui.display_file_save_dialog(
            file_name,
            format!("Select location for download from {}", domain),
        );
        let result = match dialog {
            Some(dialog) => {
                let process = activation.context.load_manager.download_file_dialog(
                    activation.context.player.clone(),
                    this,
                    dialog,
                    url_string,
                );

                activation.context.navigator.spawn_future(process);
                true
            }
            None => false,
        };

        return Ok(result.into());
    }

    Ok(false.into())
}

pub fn upload<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_reference) = this.native() {
        // If we haven't `.browse()`ed something yet, we can't upload it
        if !file_reference.0.read().is_initialised {
            return Ok(false.into());
        }

        if let Some(url) = args.first() {
            let url_string = url.coerce_to_string(activation)?.to_string();

            // Invalid domain should bail out with false
            let url = match Url::parse(&url_string) {
                Ok(url) => url,
                Err(_) => return Ok(false.into()),
            };

            // We should only allow uploads to http(s) urls
            match url.scheme() {
                "https" | "http" => {}
                _ => return Ok(false.into()),
            }

            let process = activation.context.load_manager.upload_file(
                activation.context.player.clone(),
                this,
                url_string,
                file_reference.0.read().data.clone(),
                file_reference
                    .0
                    .read()
                    .name
                    .clone()
                    .unwrap_or_else(|| "file".to_string()),
            );

            activation.context.navigator.spawn_future(process);

            return Ok(true.into());
        }

        return Ok(false.into());
    }

    Ok(Value::Undefined)
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set_native(
        activation.context.gc_context,
        NativeObject::FileReference(FileReferenceObject(GcCell::new(
            activation.context.gc_context,
            Default::default(),
        ))),
    );
    Ok(this.into())
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    array_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let file_reference_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, file_reference_proto, fn_proto);
    broadcaster_functions.initialize(context.gc_context, file_reference_proto.into(), array_proto);
    let constructor = FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        file_reference_proto.into(),
    );
    define_properties_on(
        OBJECT_DECLS,
        context,
        constructor.raw_script_object(),
        fn_proto,
    );
    constructor
}
