use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::{AvmString, Object, TObject, Value};
use crate::avm_warn;
use enumset::EnumSet;
use gc_arena::MutationContext;

use crate::avm1::object::shared_object::SharedObject;

use json::JsonValue;

pub fn delete_all<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.deleteAll() not implemented");
    Ok(Value::Undefined)
}

pub fn get_disk_usage<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getDiskUsage() not implemented");
    Ok(Value::Undefined)
}

/// Serialize an Object and any children to a JSON object
/// It would be best if this was implemented via serde but due to avm and context it can't
/// Undefined fields aren't serialized
fn recursive_serialize<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    obj: Object<'gc>,
    json_obj: &mut JsonValue,
) {
    for k in &obj.get_keys(activation) {
        if let Ok(elem) = obj.get(k, activation) {
            match elem {
                Value::Undefined => {}
                Value::Null => json_obj[k] = JsonValue::Null,
                Value::Bool(b) => json_obj[k] = b.into(),
                Value::Number(f) => json_obj[k] = f.into(),
                Value::String(s) => json_obj[k] = s.to_string().into(),
                Value::Object(o) => {
                    // Don't attempt to serialize functions
                    let function = activation.context.avm1.prototypes.function;
                    let array = activation.context.avm1.prototypes.array;
                    if !o
                        .is_instance_of(activation, o, function)
                        .unwrap_or_default()
                    {
                        let mut sub_data_json = JsonValue::new_object();
                        recursive_serialize(activation, o, &mut sub_data_json);
                        if o.is_instance_of(activation, o, array).unwrap_or_default() {
                            sub_data_json["__proto__"] = "Array".into();
                            sub_data_json["length"] = o.length().into();
                        }
                        json_obj[k] = sub_data_json;
                    }
                }
            }
        }
    }
}

fn recursive_deserialize<'gc>(
    json_value: JsonValue,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Value<'gc> {
    match json_value {
        JsonValue::Null => Value::Null,
        JsonValue::Short(s) => {
            Value::String(AvmString::new(activation.context.gc_context, s.to_string()))
        }
        JsonValue::String(s) => Value::String(AvmString::new(activation.context.gc_context, s)),
        JsonValue::Number(f) => Value::Number(f.into()),
        JsonValue::Boolean(b) => Value::Bool(b),
        JsonValue::Object(o) => {
            if o.get("__proto__").and_then(JsonValue::as_str) == Some("Array") {
                deserialize_array(o, activation)
            } else {
                deserialize_object(o, activation)
            }
        }
        JsonValue::Array(_) => Value::Undefined,
    }
}

/// Deserialize an Object and any children from a JSON object
/// It would be best if this was implemented via serde but due to avm and context it can't
/// Undefined fields aren't deserialized
fn deserialize_object<'gc>(
    json_obj: json::object::Object,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Value<'gc> {
    // Deserialize Object
    let obj_proto = activation.context.avm1.prototypes.object;
    if let Ok(obj) = obj_proto.create_bare_object(activation, obj_proto) {
        for entry in json_obj.iter() {
            let value = recursive_deserialize(entry.1.clone(), activation);
            obj.define_value(
                activation.context.gc_context,
                entry.0,
                value,
                EnumSet::empty(),
            );
        }
        obj.into()
    } else {
        Value::Undefined
    }
}

/// Deserialize an Object and any children from a JSON object
/// It would be best if this was implemented via serde but due to avm and context it can't
/// Undefined fields aren't deserialized
fn deserialize_array<'gc>(
    mut json_obj: json::object::Object,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Value<'gc> {
    let array_constructor = activation.context.avm1.prototypes.array_constructor;
    let len = json_obj
        .get("length")
        .and_then(JsonValue::as_i32)
        .unwrap_or_default();
    if let Ok(obj) = array_constructor.construct(activation, &[len.into()]) {
        // Remove length and proto meta-properties.
        json_obj.remove("length");
        json_obj.remove("__proto__");

        for entry in json_obj.iter() {
            let value = recursive_deserialize(entry.1.clone(), activation);
            if let Ok(i) = entry.0.parse::<i32>() {
                obj.set_array_element(i as usize, value, activation.context.gc_context);
            } else {
                obj.define_value(
                    activation.context.gc_context,
                    entry.0,
                    value,
                    EnumSet::empty(),
                );
            }
        }

        obj.into()
    } else {
        Value::Undefined
    }
}

pub fn get_local<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(activation)?
        .to_string();

    //Check if this is referencing an existing shared object
    if let Some(so) = activation.context.shared_objects.get(&name) {
        return Ok(Value::Object(*so));
    }

    if args.len() > 1 {
        avm_warn!(
            activation,
            "SharedObject.getLocal() doesn't support localPath or secure yet"
        );
    }

    // Data property only should exist when created with getLocal/Remote
    let constructor = activation.context.avm1.prototypes.shared_object_constructor;
    let this = constructor.construct(activation, &[])?;

    // Set the internal name
    let obj_so = this.as_shared_object().unwrap();
    obj_so.set_name(activation.context.gc_context, name.to_string());

    let prototype = activation.context.avm1.prototypes.object;
    let mut data = Value::Undefined;

    // Load the data object from storage if it existed prior
    if let Some(saved) = activation.context.storage.get_string(&name) {
        if let Ok(json_data) = json::parse(&saved) {
            data = recursive_deserialize(json_data, activation);
        }
    }

    if data == Value::Undefined {
        // No data; create a fresh data object.
        data = prototype.create_bare_object(activation, prototype)?.into();
    }

    this.define_value(
        activation.context.gc_context,
        "data",
        data,
        EnumSet::empty(),
    );

    activation.context.shared_objects.insert(name, this);

    Ok(this.into())
}

pub fn get_remote<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getRemote() not implemented");
    Ok(Value::Undefined)
}

pub fn get_max_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getMaxSize() not implemented");
    Ok(Value::Undefined)
}

pub fn add_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.addListener() not implemented");
    Ok(Value::Undefined)
}

pub fn remove_listener<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.removeListener() not implemented");
    Ok(Value::Undefined)
}

pub fn create_shared_object_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    shared_object_proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let shared_obj = FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        shared_object_proto,
    );
    let mut object = shared_obj.as_script_object().unwrap();

    object.force_set_function(
        "deleteAll",
        delete_all,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    object.force_set_function(
        "getDiskUsage",
        get_disk_usage,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    object.force_set_function(
        "getLocal",
        get_local,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    object.force_set_function(
        "getRemote",
        get_remote,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    object.force_set_function(
        "getMaxSize",
        get_max_size,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    object.force_set_function(
        "addListener",
        add_listener,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    object.force_set_function(
        "removeListener",
        remove_listener,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    shared_obj
}

pub fn clear<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let data = this.get("data", activation)?.coerce_to_object(activation);

    for k in &data.get_keys(activation) {
        data.delete(activation, k);
    }

    let so = this.as_shared_object().unwrap();
    let name = so.get_name();

    activation.context.storage.remove_key(&name);

    Ok(Value::Undefined)
}

pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.close() not implemented");
    Ok(Value::Undefined)
}

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.connect() not implemented");
    Ok(Value::Undefined)
}

pub fn flush<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let data = this.get("data", activation)?.coerce_to_object(activation);

    let mut data_json = JsonValue::new_object();
    recursive_serialize(activation, data, &mut data_json);

    let this_obj = this.as_shared_object().unwrap();
    let name = this_obj.get_name();

    Ok(activation
        .context
        .storage
        .put_string(&name, data_json.dump())
        .into())
}

pub fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.getSize() not implemented");
    Ok(Value::Undefined)
}

pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.send() not implemented");
    Ok(Value::Undefined)
}

pub fn set_fps<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.setFps() not implemented");
    Ok(Value::Undefined)
}

pub fn on_status<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.onStatus() not implemented");
    Ok(Value::Undefined)
}

pub fn on_sync<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "SharedObject.onSync() not implemented");
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let shared_obj = SharedObject::empty_shared_obj(gc_context, Some(proto));
    let mut object = shared_obj.as_script_object().unwrap();

    object.force_set_function("clear", clear, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function("close", close, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "connect",
        connect,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function("flush", flush, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "getSize",
        get_size,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function("send", send, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "setFps",
        set_fps,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "onStatus",
        on_status,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "onSync",
        on_sync,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    shared_obj.into()
}

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}
