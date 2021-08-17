use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::object::file_reference::FileReferenceObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::avm_warn;
use crate::backend::ui::FileFilter;
use crate::string::AvmString;
use gc_arena::MutationContext;

// There are two undocumented functions in FileReference: convertToPPT and deleteConvertedPPT.
// Until further reason is given, they will be unimplemented.
// See:
// ASSetPropFlags(flash.net.FileReference.prototype, null, 6, 1);
// for(var k in flash.net.FileReference.prototype) {
// 	trace(k);
// }

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "creationDate" => property(creation_date);
    "creator" => property(creator);
    "modificationDate" => property(modification_date);
    "name" => property(name);
    "postData" => property(post_data, set_post_data);
    "size" => property(size);
    "type" => property(file_type);
    "browse" => method(browse; DONT_ENUM);
    "cancel" => method(cancel; DONT_ENUM);
    "download" => method(download; DONT_ENUM);
    "upload" => method(upload; DONT_ENUM);
};

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn creation_date<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref
            .creation_date()
            .map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn creator<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.creator().map_or(Value::Undefined, |x| {
            AvmString::new(activation.context.gc_context, x).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn modification_date<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref
            .modification_date()
            .map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.name().map_or(Value::Undefined, |x| {
            AvmString::new(activation.context.gc_context, x).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn post_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(AvmString::new(activation.context.gc_context, file_ref.post_data()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_post_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let post_data = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    if let Some(file_ref) = this.as_file_reference_object() {
        file_ref.set_post_data(activation.context.gc_context, post_data.to_string());
    }

    Ok(Value::Undefined)
}

pub fn size<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.size().map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn file_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.file_type().map_or(Value::Undefined, |x| {
            AvmString::new(activation.context.gc_context, x).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn browse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let file_filters = match args.get(0) {
        Some(Value::Object(array)) => {
            // Array of filter objects.
            let length = array.length(activation)?;
            let vec: Result<Vec<_>, Error<'gc>> = (0..length)
                .map(|i| {
                    if let Value::Object(element) = array.get_element(activation, i) {
                        let mac_type = if let Ok(val) = element.get("macType", activation) {
                            Some(val.coerce_to_string(activation)?.to_string())
                        } else {
                            None
                        };

                        Ok(FileFilter {
                            description: element
                                .get("description", activation)?
                                .coerce_to_string(activation)?
                                .to_string(),
                            extensions: element
                                .get("extension", activation)?
                                .coerce_to_string(activation)?
                                .to_string(),
                            mac_type,
                        })
                    } else {
                        Err(Error::ThrownValue("Unexpected filter value".into()))
                    }
                })
                .collect();
            vec?
        }
        None => Vec::new(),
        _ => return Ok(Value::Undefined),
    };

    let dialog = activation.context.ui.display_file_dialog(file_filters);
    let process = activation.context.load_manager.select_file_dialog(
        activation.context.player.clone().unwrap(),
        this,
        dialog,
    );

    activation.context.navigator.spawn_future(process);

    Ok(Value::Null)
}

pub fn cancel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "FileReference.cancel() not implemented");
    Ok(Value::Undefined)
}

pub fn download<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "FileReference.download() not implemented");
    Ok(Value::Undefined)
}

pub fn upload<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "FileReference.upload() not implemented");
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    array_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let object = FileReferenceObject::empty_object(gc_context, Some(proto));
    broadcaster_functions.initialize(gc_context, object.into(), array_proto);
    let script_object = object.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, script_object, fn_proto);
    object.into()
}
