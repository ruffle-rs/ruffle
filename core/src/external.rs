use crate::avm1::NativeObject;
use crate::avm1::Value as Avm1Value;
use crate::avm1::{Activation as Avm1Activation, ActivationIdentifier as Avm1ActivationIdentifier};
use crate::avm1::{ArrayBuilder as Avm1ArrayBuilder, Error as Avm1Error, Object as Avm1Object};
use crate::avm2::activation::Activation as Avm2Activation;
use crate::avm2::error::Error as Avm2Error;
use crate::avm2::object::{
    ArrayObject as Avm2ArrayObject, FunctionObject as Avm2FunctionObject, Object as Avm2Object,
    ScriptObject as Avm2ScriptObject, TObject as _,
};
use crate::avm2::{FunctionArgs, Value as Avm2Value};
use crate::context::UpdateContext;
use crate::string::AvmString;
use gc_arena::Collect;
use std::collections::BTreeMap;
use std::rc::Rc;

/// An intermediate format of representing shared data between ActionScript and elsewhere.
///
/// Regardless of the capabilities of both sides, all data will be translated to this potentially
/// lossy format. Any recursion or additional metadata in ActionScript will not be translated.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(BTreeMap<String, Value>),
    List(Vec<Value>),
}

impl From<AvmString<'_>> for Value {
    fn from(string: AvmString<'_>) -> Self {
        Value::String(string.to_string())
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::String(string)
    }
}

impl From<&'static str> for Value {
    fn from(string: &'static str) -> Self {
        Value::String(string.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::Number(f64::from(value))
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Number(f64::from(value))
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::Number(f64::from(value))
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Number(value as f64)
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(value: BTreeMap<String, Value>) -> Self {
        Value::Object(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::List(value)
    }
}

impl Value {
    pub fn from_avm1<'gc>(
        activation: &mut Avm1Activation<'_, 'gc>,
        value: Avm1Value<'gc>,
    ) -> Result<Value, Avm1Error<'gc>> {
        Ok(match value {
            Avm1Value::Undefined => Value::Undefined,
            Avm1Value::Null => Value::Null,
            Avm1Value::Bool(value) => value.into(),
            Avm1Value::Number(value) => value.into(),
            Avm1Value::String(value) => Value::String(value.to_string()),
            Avm1Value::MovieClip(_) => Value::Null,
            Avm1Value::Object(object) if matches!(object.native(), NativeObject::Array(_)) => {
                let length = object.length(activation)?;
                let values: Result<Vec<_>, Avm1Error<'gc>> = (0..length)
                    .map(|i| {
                        let element = object.get_element(activation, i);
                        Value::from_avm1(activation, element)
                    })
                    .collect();
                Value::List(values?)
            }
            Avm1Value::Object(object) => {
                let keys = object.get_keys(activation, false);
                let mut values = BTreeMap::new();
                for key in keys {
                    let value = object.get(key, activation)?;
                    values.insert(key.to_string(), Value::from_avm1(activation, value)?);
                }
                Value::Object(values)
            }
        })
    }

    pub fn into_avm1<'gc>(self, activation: &mut Avm1Activation<'_, 'gc>) -> Avm1Value<'gc> {
        match self {
            Value::Undefined => Avm1Value::Undefined,
            Value::Null => Avm1Value::Null,
            Value::Bool(value) => Avm1Value::Bool(value),
            Value::Number(value) => Avm1Value::Number(value),
            Value::String(value) => {
                let value = if activation.swf_version() < 9 && value.trim().is_empty() {
                    "null"
                } else {
                    &value
                };
                Avm1Value::String(AvmString::new_utf8(activation.gc(), value))
            }
            Value::Object(values) => {
                let object = Avm1Object::new(
                    &activation.context.strings,
                    Some(activation.prototypes().object),
                );
                for (key, value) in values {
                    let key = AvmString::new_utf8(activation.gc(), key);
                    let _ = object.set(key, value.into_avm1(activation), activation);
                }
                object.into()
            }
            Value::List(values) => Avm1ArrayBuilder::new(activation)
                .with(
                    values
                        .iter()
                        .map(|value| value.to_owned().into_avm1(activation)),
                )
                .into(),
        }
    }

    pub fn from_avm2<'gc>(
        activation: &mut Avm2Activation<'_, 'gc>,
        value: Avm2Value<'gc>,
    ) -> Result<Value, Avm2Error<'gc>> {
        Ok(match value {
            Avm2Value::Undefined => Value::Undefined,
            Avm2Value::Null => Value::Null,
            Avm2Value::Bool(value) => value.into(),
            Avm2Value::Number(value) => value.into(),
            Avm2Value::Integer(value) => value.into(),
            Avm2Value::String(value) => Value::String(value.to_string()),
            Avm2Value::Object(obj) => {
                if let Some(array) = obj.as_array_storage() {
                    let length = array.length();
                    let values = (0..length)
                        .map(|i| {
                            // FIXME - is this right?
                            let element = array.get(i).unwrap_or(Avm2Value::Null);
                            Value::from_avm2(activation, element)
                        })
                        .collect::<Result<Vec<Value>, Avm2Error>>()?;
                    Value::List(values)
                } else if matches!(obj, Avm2Object::ScriptObject(_)) {
                    let mut values = BTreeMap::new();

                    let mut last_index = obj.get_next_enumerant(0, activation)?;
                    while last_index != 0 {
                        let name = obj
                            .get_enumerant_name(last_index, activation)?
                            .coerce_to_string(activation)?;
                        let value = obj.get_enumerant_value(last_index, activation)?;

                        values.insert(name.to_string(), Value::from_avm2(activation, value)?);

                        last_index = obj.get_next_enumerant(last_index, activation)?;
                    }

                    Value::Object(values)
                } else {
                    tracing::warn!("from_avm2 needs to be implemented for Avm2Value::Object");
                    Value::Null
                }
            }
        })
    }

    pub fn into_avm2<'gc>(self, activation: &mut Avm2Activation<'_, 'gc>) -> Avm2Value<'gc> {
        match self {
            Value::Undefined => Avm2Value::Undefined,
            Value::Null => Avm2Value::Null,
            Value::Bool(value) => Avm2Value::Bool(value),
            Value::Number(value) => Avm2Value::Number(value),
            Value::String(value) => Avm2Value::String(AvmString::new_utf8(activation.gc(), value)),
            Value::Object(values) => {
                let obj = Avm2ScriptObject::new_object(activation);

                for (key, value) in values.into_iter() {
                    let key = AvmString::new_utf8(activation.gc(), key);
                    let value = value.into_avm2(activation);
                    obj.set_dynamic_property(key, value, activation.gc());
                }
                Avm2Value::Object(obj)
            }
            Value::List(values) => {
                let storage = values
                    .iter()
                    .map(|value| value.to_owned().into_avm2(activation))
                    .collect();

                Avm2ArrayObject::from_storage(activation, storage).into()
            }
        }
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub enum Callback<'gc> {
    Avm1 {
        this: Avm1Value<'gc>,
        method: Avm1Object<'gc>,
    },
    Avm2 {
        method: Avm2FunctionObject<'gc>,
    },
}

impl<'gc> Callback<'gc> {
    pub fn call(
        &self,
        context: &mut UpdateContext<'gc>,
        name: &str,
        args: impl IntoIterator<Item = Value>,
    ) -> Value {
        match self {
            Callback::Avm1 { this, method } => {
                if let Some(base_clip) = context.stage.root_clip() {
                    let mut activation = Avm1Activation::from_nothing(
                        context,
                        Avm1ActivationIdentifier::root("[ExternalInterface]"),
                        base_clip,
                    );
                    let this = this.coerce_to_object(&mut activation);
                    let args: Vec<Avm1Value> = args
                        .into_iter()
                        .map(|v| v.into_avm1(&mut activation))
                        .collect();
                    let name = AvmString::new_utf8(activation.gc(), name);
                    if let Ok(result) = method
                        .call(name, &mut activation, this.into(), &args)
                        .and_then(|value| Value::from_avm1(&mut activation, value))
                    {
                        return result;
                    }
                }
                Value::Null
            }
            Callback::Avm2 { method } => {
                let domain = context
                    .library
                    .library_for_movie(context.root_swf.clone())
                    .unwrap()
                    .avm2_domain();
                let mut activation = Avm2Activation::from_domain(context, domain);
                let args: Vec<Avm2Value> = args
                    .into_iter()
                    .map(|v| v.into_avm2(&mut activation))
                    .collect();

                let result = method.call(
                    &mut activation,
                    Avm2Value::Null,
                    FunctionArgs::from_slice(&args),
                );
                match result.and_then(|value| Value::from_avm2(&mut activation, value)) {
                    Ok(result) => result,
                    Err(e) => {
                        tracing::error!(
                            "Unhandled error in External Interface callback {name}: {:?}",
                            e
                        );
                        Value::Null
                    }
                }
            }
        }
    }
}

pub trait FsCommandProvider {
    fn on_fs_command(&self, command: &str, args: &str) -> bool;
}

pub struct NullFsCommandProvider;

impl FsCommandProvider for NullFsCommandProvider {
    fn on_fs_command(&self, _command: &str, _args: &str) -> bool {
        false
    }
}

pub trait ExternalInterfaceProvider {
    fn call_method(&self, context: &mut UpdateContext<'_>, name: &str, args: &[Value]) -> Value;

    fn on_callback_available(&self, name: &str);

    fn get_id(&self) -> Option<String>;

    fn update(&self, _context: &mut UpdateContext<'_>) {}
}

pub struct NullExternalInterfaceProvider;

#[derive(Collect)]
#[collect(no_drop)]
pub struct ExternalInterface<'gc> {
    #[collect(require_static)]
    provider: Option<Rc<Box<dyn ExternalInterfaceProvider>>>,
    callbacks: BTreeMap<String, Callback<'gc>>,
    #[collect(require_static)]
    fs_commands: Box<dyn FsCommandProvider>,
}

impl<'gc> ExternalInterface<'gc> {
    pub fn new(
        provider: Option<Box<dyn ExternalInterfaceProvider>>,
        fs_commands: Box<dyn FsCommandProvider>,
    ) -> Self {
        Self {
            provider: provider.map(Rc::new),
            callbacks: Default::default(),
            fs_commands,
        }
    }

    pub fn set_provider(&mut self, provider: Option<Box<dyn ExternalInterfaceProvider>>) {
        self.provider = provider.map(Rc::new);
    }

    pub fn get_provider(&self) -> Option<Rc<Box<dyn ExternalInterfaceProvider>>> {
        self.provider.clone()
    }

    pub fn add_callback(&mut self, name: String, callback: Callback<'gc>) {
        self.callbacks.insert(name.clone(), callback);
        if let Some(provider) = &self.provider {
            provider.on_callback_available(&name);
        }
    }

    pub fn get_callback(&self, name: &str) -> Option<Callback<'gc>> {
        self.callbacks.get(name).cloned()
    }

    pub fn call_method(context: &mut UpdateContext<'gc>, name: &str, args: &[Value]) -> Value {
        let provider = context.external_interface.provider.clone();
        if let Some(provider) = &provider {
            provider.call_method(context, name, args)
        } else {
            Value::Undefined
        }
    }

    pub fn available(&self) -> bool {
        self.provider.is_some()
    }

    pub fn get_id(&self) -> Option<String> {
        self.provider.as_ref().and_then(|p| p.get_id())
    }

    pub fn invoke_fs_command(&self, command: &str, args: &str) -> bool {
        self.fs_commands.on_fs_command(command, args)
    }
}
