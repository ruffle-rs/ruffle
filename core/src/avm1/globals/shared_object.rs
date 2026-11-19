use crate::avm1::amf::{
    Amf0ReferenceCache, SerializationContext, deserialize_lso_value, recursive_serialize,
};
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{Activation, Attribute, Error, NativeObject, Object, Value};
use crate::avm1_stub;
use crate::display_object::TDisplayObject;
use crate::shared_object::local_shared_object_key;
use crate::string::AvmString;
use flash_lso::amf0::writer::{Amf0Writer, CacheKey, ObjWriter};
use flash_lso::types::Lso;
use gc_arena::{Collect, Gc};
use ruffle_macros::istr;

#[derive(Default, Clone, Collect)]
#[collect(require_static)]
pub struct SharedObject {
    /// The local name of this shared object
    name: Option<String>,
    // In future this will also handle remote SharedObjects
}

impl SharedObject {
    fn name(&self) -> String {
        self.name.clone().unwrap_or_default()
    }

    fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
}

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "connect" => method(connect; DONT_ENUM | DONT_DELETE | VERSION_6);
    "send" => method(send; DONT_ENUM | DONT_DELETE | VERSION_6);
    "flush" => method(flush; DONT_ENUM | DONT_DELETE | VERSION_6);
    "close" => method(close; DONT_ENUM | DONT_DELETE | VERSION_6);
    "getSize" => method(get_size; DONT_ENUM | DONT_DELETE | VERSION_6);
    "setFps" => method(set_fps; DONT_ENUM | DONT_DELETE | VERSION_6);
    "clear" => method(clear; DONT_ENUM | DONT_DELETE | VERSION_6);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "deleteAll" => method(@As2 delete_all; DONT_ENUM);
    "getDiskUsage" => method(@As2 get_disk_usage; DONT_ENUM);
    "getLocal" => method(@As2 get_local);
    "getRemote" => method(@As2 get_remote);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto, PropertyOrder::PrototypeLast);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

fn delete_all<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "deleteAll");
    Ok(Value::Undefined)
}

fn get_disk_usage<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "getDiskUsage");
    Ok(Value::Undefined)
}

/// Deserializes a Lso into an object containing the properties stored
fn deserialize_lso<'gc>(
    activation: &mut Activation<'_, 'gc>,
    lso: &Lso,
) -> Result<Object<'gc>, Error<'gc>> {
    let obj = Object::new(
        &activation.context.strings,
        Some(activation.prototypes().object),
    );

    let mut reference_cache = Amf0ReferenceCache::default();

    // The flattened LSO body omits SharedObject.data itself, but Flash Player
    // still reserves reference-table slot 0 for that implicit root object.
    reference_cache.insert(obj.into());

    for child in &lso.body {
        obj.define_value(
            activation.gc(),
            AvmString::new_utf8(activation.gc(), &child.name),
            deserialize_lso_value(activation, child.value(), &mut reference_cache),
            Attribute::empty(),
        );
    }

    Ok(obj)
}

fn new_lso<'gc>(activation: &mut Activation<'_, 'gc>, name: &str, data: Object<'gc>) -> Lso {
    let mut w = Amf0Writer::default();

    // SharedObject.data is the first value in Flash Player's AMF0 reference
    // table, even though its properties are written directly into the LSO body.
    // Reserve its slot so child objects use the same indices as Flash Player.
    let root_reference = w.make_reference();
    w.cache_add(CacheKey::from_ptr(data.as_ptr()), root_reference);

    recursive_serialize(activation, data, &mut w, SerializationContext::SharedObject);
    w.commit_lso(
        &name
            .split('/')
            .next_back()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
    )
}

fn get_local<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let local_path = if let Some(Value::String(local_path)) = args.get(1) {
        Some(local_path.to_utf8_lossy())
    } else {
        None
    };
    // Only the boolean `true` is secure, other values aren't converted.
    let secure = matches!(args.get(2), Some(Value::Bool(true)));

    let movie = activation.base_clip().movie();
    let Some(full_name) = local_shared_object_key(
        movie.url(),
        &name.to_utf8_lossy(),
        local_path.as_deref(),
        secure,
    ) else {
        return Ok(Value::Null);
    };

    // Check if this is referencing an existing shared object
    if let Some(so) = activation.context.avm1_shared_objects.get(&full_name) {
        return Ok((*so).into());
    }

    // Data property only should exist when created with getLocal/Remote
    let class_path = [istr!("SharedObject")];
    let Value::Object(this) = activation.instantiate_class_as_script(class_path, &[])? else {
        return Ok(Value::Null);
    };

    // Set the internal name
    if let NativeObject::SharedObject(shared_object) = this.native() {
        shared_object.borrow_mut().set_name(full_name.clone());
    }

    let mut data = Value::Undefined;

    // Load the data object from storage if it existed prior
    if let Some(saved) = activation.context.storage.get(&full_name) {
        let mut reader = flash_lso::read::Reader::default();
        if let Ok(lso) = reader.parse(&saved) {
            data = deserialize_lso(activation, &lso)?.into();
        }
    }

    if data == Value::Undefined {
        // No data; create a fresh data object.
        data = Object::new(
            &activation.context.strings,
            Some(activation.prototypes().object),
        )
        .into();
    }

    this.define_value(activation.gc(), istr!("data"), data, Attribute::DONT_DELETE);

    activation
        .context
        .avm1_shared_objects
        .insert(full_name, this);

    Ok(this.into())
}

fn get_remote<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "getRemote");
    Ok(Value::Undefined)
}

fn clear<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let data = this
        .get(istr!("data"), activation)?
        .coerce_to_object_or_bare(activation)?;

    for k in &data.get_keys(activation, false) {
        data.delete(activation, *k);
    }

    if let NativeObject::SharedObject(shared_object) = this.native() {
        let name = shared_object.borrow().name();
        activation.context.storage.remove_key(&name);
    }

    Ok(Value::Undefined)
}

fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "close");
    Ok(Value::Undefined)
}

fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "connect");
    Ok(Value::Undefined)
}

pub(crate) fn flush<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let NativeObject::SharedObject(shared_object) = this.native() else {
        return Ok(Value::Undefined);
    };
    let name = shared_object.borrow().name();
    let data = this
        .get(istr!("data"), activation)?
        .coerce_to_object_or_bare(activation)?;
    let mut lso = new_lso(activation, &name, data);
    flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
    // Flash does not write empty LSOs to disk
    if lso.body.is_empty() {
        Ok(true.into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        Ok(activation.context.storage.put(&name, &bytes).into())
    }
}

fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let NativeObject::SharedObject(shared_object) = this.native() else {
        return Ok(Value::Undefined);
    };
    let name = shared_object.borrow().name();
    let data = this
        .get(istr!("data"), activation)?
        .coerce_to_object_or_bare(activation)?;
    let mut lso = new_lso(activation, &name, data);
    // Flash returns 0 for empty LSOs, but the actual number of bytes (including the header) otherwise
    if lso.body.is_empty() {
        Ok(0.into())
    } else {
        let bytes = flash_lso::write::write_to_bytes(&mut lso).unwrap_or_default();
        Ok(Value::from_usize_lossy(bytes.len()))
    }
}

fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "send");
    Ok(Value::Undefined)
}

fn set_fps<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "SharedObject", "setFps");
    Ok(Value::Undefined)
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set_native(
        activation.gc(),
        NativeObject::SharedObject(Gc::new(activation.gc(), Default::default())),
    );
    Ok(Value::Undefined)
}
