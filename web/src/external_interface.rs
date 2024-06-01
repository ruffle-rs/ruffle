use crate::{JavascriptPlayer, CURRENT_CONTEXT};
use js_sys::{Array, Function, Object};
use ruffle_core::context::UpdateContext;
use ruffle_core::external::{
    ExternalInterfaceMethod, ExternalInterfaceProvider, FsCommandProvider, Value as ExternalValue,
    Value,
};
use std::collections::BTreeMap;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Clone)]
pub struct JavascriptInterface {
    js_player: JavascriptPlayer,
}

struct JavascriptMethod {
    this: JsValue,
    function: JsValue,
}

impl ExternalInterfaceMethod for JavascriptMethod {
    fn call(&self, context: &mut UpdateContext<'_, '_>, args: &[ExternalValue]) -> ExternalValue {
        let old_context = CURRENT_CONTEXT.with(|v| {
            v.replace(Some(unsafe {
                std::mem::transmute::<&mut UpdateContext, &mut UpdateContext<'static, 'static>>(
                    context,
                )
            } as *mut UpdateContext))
        });
        let result = if let Some(function) = self.function.dyn_ref::<Function>() {
            let args_array = Array::new();
            for arg in args {
                args_array.push(&external_to_js_value(arg.to_owned()));
            }
            if let Ok(result) = function.apply(&self.this, &args_array) {
                js_to_external_value(&result)
            } else {
                ExternalValue::Undefined
            }
        } else {
            ExternalValue::Undefined
        };
        CURRENT_CONTEXT.with(|v| v.replace(old_context));
        result
    }
}

impl JavascriptInterface {
    pub fn new(js_player: JavascriptPlayer) -> Self {
        Self { js_player }
    }

    fn find_method(&self, root: JsValue, name: &str) -> Option<JavascriptMethod> {
        let mut parent = JsValue::UNDEFINED;
        let mut value = root;
        for key in name.split('.') {
            parent = value;
            value = crate::get_property(&parent, &JsValue::from_str(key)).ok()?;
        }
        if value.is_function() {
            Some(JavascriptMethod {
                this: parent,
                function: value,
            })
        } else {
            None
        }
    }
}

impl ExternalInterfaceProvider for JavascriptInterface {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        if let Some(method) = self.find_method(self.js_player.clone().into(), name) {
            return Some(Box::new(method));
        }
        if let Some(window) = web_sys::window() {
            if let Some(method) = self.find_method(window.into(), name) {
                return Some(Box::new(method));
            }
        }

        // Return a dummy method, as `ExternalInterface.call` must return `undefined`, not `null`.
        Some(Box::new(JavascriptMethod {
            this: JsValue::UNDEFINED,
            function: JsValue::UNDEFINED,
        }))
    }

    fn on_callback_available(&self, name: &str) {
        self.js_player.on_callback_available(name);
    }

    fn get_id(&self) -> Option<String> {
        self.js_player.get_object_id()
    }
}

impl FsCommandProvider for JavascriptInterface {
    fn on_fs_command(&self, command: &str, args: &str) -> bool {
        self.js_player
            .on_fs_command(command, args)
            .unwrap_or_default()
    }
}

pub fn js_to_external_value(js: &JsValue) -> ExternalValue {
    if let Some(value) = js.as_f64() {
        ExternalValue::Number(value)
    } else if let Some(value) = js.as_string() {
        ExternalValue::String(value)
    } else if let Some(value) = js.as_bool() {
        ExternalValue::Bool(value)
    } else if let Some(array) = js.dyn_ref::<Array>() {
        let values: Vec<_> = array
            .values()
            .into_iter()
            .flatten()
            .map(|v| js_to_external_value(&v))
            .collect();
        ExternalValue::List(values)
    } else if let Some(object) = js.dyn_ref::<Object>() {
        let mut values = BTreeMap::new();
        for entry in Object::entries(object).values() {
            if let Ok(entry) = entry.and_then(|v| v.dyn_into::<Array>()) {
                if let Some(key) = entry.get(0).as_string() {
                    values.insert(key, js_to_external_value(&entry.get(1)));
                }
            }
        }
        ExternalValue::Object(values)
    } else if js.is_null() {
        ExternalValue::Null
    } else {
        ExternalValue::Undefined
    }
}

pub fn external_to_js_value(external: ExternalValue) -> JsValue {
    match external {
        Value::Undefined => JsValue::UNDEFINED,
        Value::Null => JsValue::NULL,
        Value::Bool(value) => JsValue::from_bool(value),
        Value::Number(value) => JsValue::from_f64(value),
        Value::String(value) => JsValue::from_str(&value),
        Value::Object(object) => {
            let entries = Array::new();
            for (key, value) in object {
                entries.push(&Array::of2(
                    &JsValue::from_str(&key),
                    &external_to_js_value(value),
                ));
            }
            if let Ok(result) = Object::from_entries(&entries) {
                result.into()
            } else {
                JsValue::NULL
            }
        }
        Value::List(values) => {
            let array = Array::new();
            for value in values {
                array.push(&external_to_js_value(value));
            }
            array.into()
        }
    }
}
