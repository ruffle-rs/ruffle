//! flash.net.FileReference object

use std::cell::{Cell, RefCell};

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{NativeObject, Object, Value};
use crate::avm1_stub;
use crate::backend::ui::{FileDialogResult, FileFilter};
use crate::string::AvmString;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc};
use ruffle_macros::istr;
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
pub struct FileReferenceObject<'gc>(Gc<'gc, FileReferenceData<'gc>>);

impl<'gc> FileReferenceObject<'gc> {
    pub fn init_from_dialog_result(
        self,
        activation: &mut Activation<'_, 'gc>,
        result: &dyn FileDialogResult,
    ) {
        let mc = activation.gc();
        let write = Gc::write(mc, self.0);

        self.0.is_initialised.set(true);

        let date_proto = activation.prototypes().date_constructor;
        if let Some(creation_time) = result.creation_time() {
            if let Ok(Value::Object(obj)) = date_proto.construct(
                activation,
                &[(creation_time.timestamp_millis() as f64).into()],
            ) {
                unlock!(write, FileReferenceData, creation_date).set(Some(obj));
            }
        }

        if let Some(modification_time) = result.modification_time() {
            if let Ok(Value::Object(obj)) = date_proto.construct(
                activation,
                &[(modification_time.timestamp_millis() as f64).into()],
            ) {
                unlock!(write, FileReferenceData, modification_date).set(Some(obj));
            }
        }

        let file_type = result.file_type().map(|s| AvmString::new_utf8(mc, s));
        unlock!(write, FileReferenceData, file_type).set(file_type);

        let file_name = result.file_name().map(|s| AvmString::new_utf8(mc, s));
        unlock!(write, FileReferenceData, name).set(file_name);

        let creator = result.creator().map(|s| AvmString::new_utf8(mc, s));
        unlock!(write, FileReferenceData, creator).set(creator);

        self.0.size.replace(result.size());
        self.0.data.replace(result.contents().to_vec());
    }
}

#[derive(Clone, Default, Collect)]
#[collect(no_drop)]
pub struct FileReferenceData<'gc> {
    /// Has this object been initialised from a dialog
    is_initialised: Cell<bool>,

    creation_date: Lock<Option<Object<'gc>>>,
    creator: Lock<Option<AvmString<'gc>>>,
    modification_date: Lock<Option<Object<'gc>>>,
    name: Lock<Option<AvmString<'gc>>>,
    post_data: Lock<Option<AvmString<'gc>>>,
    size: Cell<Option<u64>>,
    file_type: Lock<Option<AvmString<'gc>>>,

    /// The contents of the referenced file
    /// We track this here so that it can be referenced in FileReference.upload
    data: RefCell<Vec<u8>>,
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

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
    broadcaster_fns: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    broadcaster_fns.initialize(context.strings, class.proto, array_proto);
    context.define_properties_on(class.constr, OBJECT_DECLS);
    class
}

pub fn creation_date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        let creation_date = file_ref.0.creation_date.get();
        return Ok(creation_date.map_or(Value::Undefined, Into::into));
    }

    Ok(Value::Undefined)
}

pub fn creator<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        let creator = file_ref.0.creator.get();
        return Ok(creator.map_or(Value::Undefined, Into::into));
    }

    Ok(Value::Undefined)
}

pub fn modification_date<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        let modification_date = file_ref.0.modification_date.get();
        return Ok(modification_date.map_or(Value::Undefined, Into::into));
    }

    Ok(Value::Undefined)
}

pub fn name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        return Ok(file_ref.0.name.get().map_or(Value::Undefined, Into::into));
    }

    Ok(Value::Undefined)
}

pub fn post_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        let post_data = file_ref.0.post_data.get();
        return Ok(post_data.unwrap_or_else(|| istr!("")).into());
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
        let write = Gc::write(activation.gc(), file_ref.0);
        unlock!(write, FileReferenceData, post_data).set(Some(post_data));
    }

    Ok(Value::Undefined)
}

pub fn size<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        let size = file_ref.0.size.get();
        return Ok(size.map_or(Value::Undefined, Into::into));
    }

    Ok(Value::Undefined)
}

pub fn file_type<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::FileReference(file_ref) = this.native() {
        let file_type = file_ref.0.file_type.get();
        return Ok(file_type.map_or(Value::Undefined, Into::into));
    }

    Ok(Value::Undefined)
}

pub fn browse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !matches!(this.native(), NativeObject::FileReference(_)) {
        return Ok(Value::Undefined);
    }

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
                        if let Some(val) = element.get_local_stored(istr!("macType"), activation) {
                            Some(val.coerce_to_string(activation)?.to_string())
                        } else {
                            None
                        };

                    let description = element.get_local_stored(istr!("description"), activation);
                    let extension = element.get_local_stored(istr!("extension"), activation);

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
                    // Method will abort if any non-Object elements are in the list
                    return Ok(false.into());
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
            let process = crate::loader::select_file_dialog_avm1(activation.context, this, dialog);

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
                match url.path().split('/').next_back() {
                    Some(path_end) => path_end.to_string(),
                    None => return Ok(false.into()),
                }
            }
        };

        let domain = url.domain().unwrap_or("<unknown domain>").to_string();

        // Create and spawn dialog
        let dialog = activation.context.ui.display_file_save_dialog(
            file_name,
            format!("Select location for download from {domain}"),
        );
        let result = match dialog {
            Some(dialog) => {
                let process = crate::loader::download_file_dialog(
                    activation.context,
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
        if !file_reference.0.is_initialised.get() {
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

            let file_name = match file_reference.0.name.get() {
                Some(name) => name.to_string(),
                None => "file".to_string(),
            };

            let process = crate::loader::upload_file(
                activation.context,
                this,
                url_string,
                file_reference.0.data.borrow().clone(),
                file_name,
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
        activation.gc(),
        NativeObject::FileReference(FileReferenceObject(Gc::new(
            activation.gc(),
            Default::default(),
        ))),
    );
    Ok(Value::Undefined)
}
