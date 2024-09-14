use crate::avm1::TObject as _;
use crate::avm1::Value as Avm1Value;
use crate::avm1::{Activation as Avm1Activation, ActivationIdentifier as Avm1ActivationIdentifier};
use crate::avm1::{
    ArrayObject as Avm1ArrayObject, Error as Avm1Error, Object as Avm1Object,
    ScriptObject as Avm1ScriptObject,
};
use crate::avm2::activation::Activation as Avm2Activation;
use crate::avm2::object::TObject as _;
use crate::avm2::Value as Avm2Value;
use crate::avm2::{ArrayObject as Avm2ArrayObject, Object as Avm2Object};
use crate::context::UpdateContext;
use crate::string::AvmString;
use gc_arena::Collect;
use std::collections::BTreeMap;

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
            Avm1Value::Object(object) => {
                if object.as_array_object().is_some() {
                    let length = object.length(activation)?;
                    let values: Result<Vec<_>, Avm1Error<'gc>> = (0..length)
                        .map(|i| {
                            let element = object.get_element(activation, i);
                            Value::from_avm1(activation, element)
                        })
                        .collect();
                    Value::List(values?)
                } else {
                    let keys = object.get_keys(activation, false);
                    let mut values = BTreeMap::new();
                    for key in keys {
                        let value = object.get(key, activation)?;
                        values.insert(key.to_string(), Value::from_avm1(activation, value)?);
                    }
                    Value::Object(values)
                }
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
                Avm1Value::String(AvmString::new_utf8(activation.context.gc_context, value))
            }
            Value::Object(values) => {
                let object = Avm1ScriptObject::new(
                    activation.context.gc_context,
                    Some(activation.context.avm1.prototypes().object),
                );
                for (key, value) in values {
                    let key = AvmString::new_utf8(activation.context.gc_context, key);
                    let _ = object.set(key, value.into_avm1(activation), activation);
                }
                object.into()
            }
            Value::List(values) => Avm1ArrayObject::new(
                activation.context.gc_context,
                activation.context.avm1.prototypes().array,
                values
                    .iter()
                    .map(|value| value.to_owned().into_avm1(activation)),
            )
            .into(),
        }
    }

    pub fn from_avm2(value: Avm2Value) -> Value {
        match value {
            Avm2Value::Undefined => Value::Undefined,
            Avm2Value::Null => Value::Null,
            Avm2Value::Bool(value) => value.into(),
            Avm2Value::Number(value) => value.into(),
            Avm2Value::Integer(value) => value.into(),
            Avm2Value::String(value) => Value::String(value.to_string()),
            Avm2Value::Object(object) => {
                if let Some(array) = object.as_array_storage() {
                    let length = array.length();
                    let values = (0..length)
                        .map(|i| {
                            // FIXME - is this right?
                            let element = array.get(i).unwrap_or(Avm2Value::Null);
                            Value::from_avm2(element)
                        })
                        .collect();
                    Value::List(values)
                } else {
                    tracing::warn!("from_avm2 needs to be implemented for Avm2Value::Object");
                    Value::Null
                }
            }
        }
    }

    pub fn into_avm2<'gc>(self, activation: &mut Avm2Activation<'_, 'gc>) -> Avm2Value<'gc> {
        match self {
            Value::Undefined => Avm2Value::Undefined,
            Value::Null => Avm2Value::Null,
            Value::Bool(value) => Avm2Value::Bool(value),
            Value::Number(value) => Avm2Value::Number(value),
            Value::String(value) => {
                Avm2Value::String(AvmString::new_utf8(activation.context.gc_context, value))
            }
            Value::Object(values) => {
                let obj = activation
                    .avm2()
                    .classes()
                    .object
                    .construct(activation, &[])
                    .unwrap();
                for (key, value) in values.into_iter() {
                    let key = AvmString::new_utf8(activation.context.gc_context, key);
                    let value = value.into_avm2(activation);
                    obj.set_public_property(key, value, activation).unwrap();
                }
                Avm2Value::Object(obj)
            }
            Value::List(values) => {
                let storage = values
                    .iter()
                    .map(|value| value.to_owned().into_avm2(activation))
                    .collect();

                Avm2Value::Object(Avm2ArrayObject::from_storage(activation, storage).unwrap())
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
        method: Avm2Object<'gc>,
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
                    let name = AvmString::new_utf8(activation.context.gc_context, name);
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
                    .library_for_movie(context.swf.clone())
                    .unwrap()
                    .avm2_domain();
                let mut activation = Avm2Activation::from_domain(context, domain);
                let args: Vec<Avm2Value> = args
                    .into_iter()
                    .map(|v| v.into_avm2(&mut activation))
                    .collect();
                match method.call(Avm2Value::Null, &args, &mut activation) {
                    Ok(result) => Value::from_avm2(result),
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
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>>;

    fn on_callback_available(&self, name: &str);

    fn get_id(&self) -> Option<String>;
}

pub trait ExternalInterfaceMethod {
    fn call(&self, context: &mut UpdateContext<'_>, args: &[Value]) -> Value;
}

impl<F> ExternalInterfaceMethod for F
where
    F: Fn(&mut UpdateContext<'_>, &[Value]) -> Value,
{
    fn call(&self, context: &mut UpdateContext<'_>, args: &[Value]) -> Value {
        self(context, args)
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct ExternalInterface<'gc> {
    #[collect(require_static)]
    providers: Vec<Box<dyn ExternalInterfaceProvider>>,
    callbacks: BTreeMap<String, Callback<'gc>>,
    #[collect(require_static)]
    fs_commands: Box<dyn FsCommandProvider>,
}

impl<'gc> ExternalInterface<'gc> {
    pub fn new(
        providers: Vec<Box<dyn ExternalInterfaceProvider>>,
        fs_commands: Box<dyn FsCommandProvider>,
    ) -> Self {
        Self {
            providers,
            callbacks: Default::default(),
            fs_commands,
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn ExternalInterfaceProvider>) {
        self.providers.push(provider);
    }

    pub fn add_callback(&mut self, name: String, callback: Callback<'gc>) {
        self.callbacks.insert(name.clone(), callback);
        for provider in &self.providers {
            provider.on_callback_available(&name);
        }
    }

    pub fn get_callback(&self, name: &str) -> Option<Callback<'gc>> {
        self.callbacks.get(name).cloned()
    }

    pub fn get_method_for(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        for provider in &self.providers {
            if let Some(method) = provider.get_method(name) {
                return Some(method);
            }
        }
        None
    }

    pub fn available(&self) -> bool {
        !self.providers.is_empty()
    }

    pub fn any_id(&self) -> Option<String> {
        for provider in &self.providers {
            if let Some(id) = provider.get_id() {
                return Some(id);
            }
        }
        None
    }

    pub fn invoke_fs_command(&self, command: &str, args: &str) -> bool {
        self.fs_commands.on_fs_command(command, args)
    }
}
