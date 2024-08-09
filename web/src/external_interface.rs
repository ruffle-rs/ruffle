use crate::{JavascriptPlayer, CURRENT_CONTEXT};
use js_sys::{Array, Object};
use ruffle_core::context::UpdateContext;
use ruffle_core::external::{
    ExternalInterfaceMethod, ExternalInterfaceProvider, FsCommandProvider, Value as ExternalValue,
    Value,
};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};

#[wasm_bindgen(raw_module = "./ruffle-imports")]
extern "C" {
    #[wasm_bindgen(catch, js_name = "callExternalInterface")]
    pub fn call_external_interface(
        method: &str,
        values: Box<[JsValue]>,
    ) -> Result<JsValue, JsValue>;
}

#[derive(Clone)]
pub struct JavascriptInterface {
    js_player: JavascriptPlayer,
}

struct JavascriptMethod(String);

impl ExternalInterfaceMethod for JavascriptMethod {
    fn call(&self, context: &mut UpdateContext<'_>, args: &[ExternalValue]) -> ExternalValue {
        let old_context = CURRENT_CONTEXT.with(|v| {
            v.replace(Some(unsafe {
                std::mem::transmute::<&mut UpdateContext, &mut UpdateContext<'static>>(context)
            } as *mut UpdateContext))
        });
        let args = args
            .iter()
            .cloned()
            .map(external_to_js_value)
            .collect::<Vec<_>>();
        let result = if let Ok(result) = call_external_interface(&self.0, args.into_boxed_slice()) {
            js_to_external_value(&result)
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
}

impl ExternalInterfaceProvider for JavascriptInterface {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        Some(Box::new(JavascriptMethod(name.to_string())))
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
    } else if js.is_null() {
        ExternalValue::Null
    } else if js.is_undefined() {
        ExternalValue::Undefined
    } else {
        let mut values = BTreeMap::new();
        for entry in Object::entries(&Object::from(js.to_owned())).values() {
            if let Ok(entry) = entry.and_then(|v| v.dyn_into::<Array>()) {
                if let Some(key) = entry.get(0).as_string() {
                    values.insert(key, js_to_external_value(&entry.get(1)));
                }
            }
        }
        ExternalValue::Object(values)
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
