use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext, Collect};
use crate::avm1::property::Attribute;
use crate::display_object::DisplayObject;
use crate::avm1::sound_object::SoundObject;

use json::JsonValue;
use std::fmt;

/// A SharedObject
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct SharedObject<'gc>(GcCell<'gc, SharedObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SharedObjectData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    /// The local name of this shared object
    name: Option<String>,
}

impl fmt::Debug for SharedObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("SharedObject")
            .field("name", &this.name)
            .finish()
    }
}

impl<'gc> SharedObject<'gc> {
    fn empty_shared_obj(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Self {
        SharedObject(GcCell::allocate(
            gc_context,
            SharedObjectData {
                base: ScriptObject::object(gc_context, proto),
                name: None,
            },
        ))
    }
    //TODO: any need for these

    //TODO: use enum Remote(url), Local(name)

    fn set_name(&self, gc_context: MutationContext<'gc, '_>, name: String) {
        self.0.write(gc_context).name = Some(name);
    }

    fn get_name(&self) -> String {
        self.0.read().name.as_ref().cloned().unwrap_or("".to_owned())
    }

    fn base(self) -> ScriptObject<'gc> {
        self.0.read().base
    }
}

impl<'gc> TObject<'gc> for SharedObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error> {
        self.base().get_local(name, avm, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.base().set(name, value, avm, context)
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error> {
        self.base().call(avm, context, this, base_proto, args)
    }

    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base().call_setter(name, value, avm, context, this)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
         Ok(SharedObject::empty_shared_obj(context.gc_context, Some(avm.prototypes.shared_object)).into())
    }

    fn delete(
        &self,
        avm: &mut Avm1<'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().delete(avm, gc_context, name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.base().proto()
    }

    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Option<Object<'gc>>) {
        self.base().set_proto(gc_context, prototype);
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .add_property(gc_context, name, get, set, attributes)
    }

    fn add_property_with_case(
        &self,
        avm: &mut Avm1<'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base()
            .add_property_with_case(avm, gc_context, name, get, set, attributes)
    }

    fn has_property(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().has_property(avm, context, name)
    }

    fn has_own_property(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().has_own_property(avm, context, name)
    }

    fn has_own_virtual(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base().has_own_virtual(avm, context, name)
    }

    fn is_property_overwritable(&self, avm: &mut Avm1<'gc>, name: &str) -> bool {
        self.base().is_property_overwritable(avm, name)
    }

    fn is_property_enumerable(&self, avm: &mut Avm1<'gc>, name: &str) -> bool {
        self.base().is_property_enumerable(avm, name)
    }

    fn get_keys(&self, avm: &mut Avm1<'gc>) -> Vec<String> {
        self.base().get_keys(avm)
    }

    fn as_string(&self) -> String {
        self.base().as_string()
    }

    fn type_of(&self) -> &'static str {
        self.base().type_of()
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.base().interfaces()
    }

    fn set_interfaces(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        iface_list: Vec<Object<'gc>>,
    ) {
        self.base().set_interfaces(gc_context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base())
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        None
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    fn as_sound_object(&self) -> Option<SoundObject<'gc>> {
        None
    }

    fn as_shared_object(&self) -> Option<SharedObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn length(&self) -> usize {
        self.base().length()
    }

    fn array(&self) -> Vec<Value<'gc>> {
        self.base().array()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, length: usize) {
        self.base().set_length(gc_context, length)
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        self.base().array_element(index)
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.base().set_array_element(index, value, gc_context)
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        self.base().delete_array_element(index, gc_context)
    }
}

pub fn delete_all<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.deleteAll() not implemented");
    Ok(Value::Undefined.into())
}

pub fn get_disk_usage<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getDiskUsage() not implemented");
    Ok(Value::Undefined.into())
}

/// Serialize an Object and any children to a JSON object
/// It would be best if this was implemented via serde but due to avm and context it can't
/// Undefined fields aren't serialized
fn recursive_serialize<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    obj: Object<'gc>,
    json_obj: &mut JsonValue,
) {
    for k in &obj.get_keys(avm) {
        let elem = obj.get(k, avm, action_context).unwrap();

        match elem {
            Value::Undefined => {},
            Value::Null => json_obj[k] = JsonValue::Null,
            Value::Bool(b) => json_obj[k] = b.into(),
            Value::Number(f) => json_obj[k] = f.into(),
            Value::String(s) => json_obj[k] = s.into(),
            Value::Object(o) => {
                // Don't attempt to serialize functions, etc
                if !o
                    .is_instance_of(avm, action_context, o, avm.prototypes.function)
                    .unwrap()
                {
                    let mut sub_data_json = JsonValue::new_object();
                    recursive_serialize(avm, action_context, o, &mut sub_data_json);
                    json_obj[k] = sub_data_json;
                }
            }
        }
    }
}

/// Deserialize an Object and any children from a JSON object
/// It would be best if this was implemented via serde but due to avm and context it can't
/// Undefined fields aren't deserialized
fn recursive_deserialize<'gc>(
    json_obj: JsonValue,
    avm: &mut Avm1<'gc>,
    object: Object<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) {
    for entry in json_obj.entries() {
        match entry.1 {
            JsonValue::Null => {
                object.define_value(context.gc_context, entry.0, Value::Null, EnumSet::empty());
            }
            JsonValue::Short(s) => {
                let val: String = s.as_str().to_string();
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::String(val),
                    EnumSet::empty(),
                );
            }
            JsonValue::String(s) => {
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::String(s.clone()),
                    EnumSet::empty(),
                );
            }
            JsonValue::Number(f) => {
                let val: f64 = f.clone().into();
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::Number(val),
                    EnumSet::empty(),
                );
            }
            JsonValue::Boolean(b) => {
                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::Bool(*b),
                    EnumSet::empty(),
                );
            }
            JsonValue::Object(o) => {
                let so = avm.prototypes.object;
                let obj = so.new(avm, context, so, &[]).unwrap();
                let _ = crate::avm1::globals::object::constructor(avm, context, obj, &[]).unwrap();
                recursive_deserialize(JsonValue::Object(o.clone()), avm, obj, context);

                object.define_value(
                    context.gc_context,
                    entry.0,
                    Value::Object(obj),
                    EnumSet::empty(),
                );
            }
            JsonValue::Array(_) => {}
        }
    }
}

pub fn get_local<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let name = args.get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(avm, action_context)?;

    if args.len() > 1 {
        log::warn!("SharedObject.getLocal() doesn't support localPath or secure yet");
    }

    // Data property only should exist when created with getLocal/Remote
    let so = avm.prototypes.shared_object;
    let this = so.new(avm, action_context, so, &[])?;
    let _ = constructor(avm, action_context, this, &[])?;

    // Set the internal name
    let obj_so = this.as_shared_object().unwrap();
    obj_so.set_name(action_context.gc_context, name.clone());

    // Create the data object
    let data_proto = avm.prototypes.object;
    let data = data_proto.new(avm, action_context, so, &[])?;
    let _ = crate::avm1::globals::object::constructor(avm, action_context, data, &[])?;

    // Load the data object from storage if it existed prior
    if let Some(saved) = action_context.storage.get_string(&name) {
        if let Ok(json_data) = json::parse(&saved) {
            recursive_deserialize(json_data, avm, data, action_context);
        }
    }

    this.define_value(
        action_context.gc_context,
        "data",
        data.into(),
        EnumSet::empty(),
    );

    Ok(this.into())
}

pub fn get_remote<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getRemote() not implemented");
    Ok(Value::Undefined.into())
}

pub fn get_max_size<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getMaxSize() not implemented");
    Ok(Value::Undefined.into())
}

pub fn add_listener<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.addListener() not implemented");
    Ok(Value::Undefined.into())
}

pub fn remove_listener<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.removeListener() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_shared_object_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    array_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let shared_obj = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        array_proto,
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
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let data = this
        .get("data", avm, action_context)
        .unwrap()
        .as_object()
        .unwrap();

    for k in &data.get_keys(avm) {
        data.delete(avm, action_context.gc_context, k);
    }

    let so = this.as_shared_object().unwrap();
    let name = so.get_name();

    action_context.storage.remove_key(&name);

    Ok(Value::Undefined.into())
}

pub fn close<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.close() not implemented");
    Ok(Value::Undefined.into())
}

pub fn connect<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.connect() not implemented");
    Ok(Value::Undefined.into())
}

pub fn flush<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let data = this
        .get("data", avm, action_context)
        .unwrap()
        .as_object()
        .unwrap();

    let mut data_json = JsonValue::new_object();
    recursive_serialize(avm, action_context, data, &mut data_json);

    let this_obj = this.as_shared_object().unwrap();
    let name = this_obj.get_name();

    Ok(action_context
        .storage
        .put_string(&name, data_json.dump()).into())
}

pub fn get_size<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getSize() not implemented");
    Ok(Value::Undefined.into())
}

pub fn send<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.send() not implemented");
    Ok(Value::Undefined.into())
}

pub fn set_fps<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.setFps() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_status<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.onStatus() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_sync<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.onSync() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let shared_obj =  SharedObject::empty_shared_obj(gc_context, Some(proto));
    let mut object =  shared_obj.as_script_object().unwrap();

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
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}
