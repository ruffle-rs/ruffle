use crate::avm1::activation::Activation;
use crate::avm1::{Object, ObjectPtr, TObject, Value};
use crate::display_object::{TDisplayObject, TDisplayObjectContainer};
use crate::string::AvmString;
use std::fmt::Write;

macro_rules! print_string {
    ($self:expr,$string:expr) => {
        $self.output.push('\"');
        for c in $string.chars() {
            let c = c.unwrap_or(char::REPLACEMENT_CHARACTER);
            let escape = match u8::try_from(c as u32) {
                Ok(b'"') => "\\\"",
                Ok(b'\\') => "\\\\",
                Ok(b'\n') => "\\n",
                Ok(b'\r') => "\\r",
                Ok(b'\t') => "\\t",
                Ok(0x08) => "\\b",
                Ok(0x0C) => "\\f",
                _ => {
                    $self.output.push(c);
                    continue;
                }
            };
            $self.output.push_str(escape);
        }
        $self.output.push('\"');
    };
}

#[allow(dead_code)]
pub struct VariableDumper<'a> {
    objects: Vec<*const ObjectPtr>,
    depth: u32,
    output: String,
    indent: &'a str,
}

impl<'a> VariableDumper<'a> {
    pub fn new(indent: &'a str) -> Self {
        Self {
            objects: Vec::new(),
            depth: 0,
            output: String::new(),
            indent,
        }
    }

    #[allow(dead_code)]
    pub fn dump<'gc>(
        value: &Value<'gc>,
        indent: &str,
        activation: &mut Activation<'_, 'gc>,
    ) -> String {
        let mut dumper = VariableDumper::new(indent);
        dumper.print_value(value, activation);
        dumper.output
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    fn object_id(&mut self, object: &Object) -> (usize, bool) {
        let ptr = object.as_ptr();

        for (i, other) in self.objects.iter().enumerate() {
            if *other == ptr {
                return (i, false);
            }
        }

        let id = self.objects.len();
        self.objects.push(ptr);
        (id, true)
    }

    fn indent(&mut self) {
        for _ in 0..self.depth {
            self.output.push_str(self.indent);
        }
    }

    pub fn print_string(&mut self, string: AvmString<'_>) {
        print_string!(self, string);
    }

    pub fn print_object<'gc>(
        &mut self,
        object: &Object<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        let (id, new) = self.object_id(object);
        self.output.push_str("[object #");
        self.output.push_str(&id.to_string());
        self.output.push(']');

        if new {
            self.print_properties(object, activation);
        }
    }

    pub fn print_property<'gc>(
        &mut self,
        object: &Object<'gc>,
        key: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        match object.get(key, activation) {
            Ok(value) => {
                self.print_value(&value, activation);
            }
            Err(e) => {
                self.output.push_str("Error: \"");
                self.output.push_str(&e.to_string());
                self.output.push('\"');
            }
        }
    }

    pub fn print_properties<'gc>(
        &mut self,
        object: &Object<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        let keys = object.get_keys(activation, false);
        if keys.is_empty() {
            self.output.push_str(" {}");
        } else {
            self.output.push_str(" {\n");
            self.depth += 1;

            for key in keys.into_iter() {
                self.indent();
                self.output.push_str(&key.to_utf8_lossy());
                self.output.push_str(": ");
                self.print_property(object, key, activation);
                self.output.push('\n');
            }

            self.depth -= 1;
            self.indent();
            self.output.push('}');
        }
    }

    pub fn print_value<'gc>(&mut self, value: &Value<'gc>, activation: &mut Activation<'_, 'gc>) {
        match value {
            Value::Undefined => self.output.push_str("undefined"),
            Value::Null => self.output.push_str("null"),
            Value::Bool(value) => self.output.push_str(&value.to_string()),
            Value::Number(value) => self.output.push_str(&value.to_string()),
            Value::String(value) => {
                self.print_string(*value);
            }
            Value::Object(object) => {
                self.print_object(object, activation);
            }
            Value::MovieClip(_) => {
                let obj = value.coerce_to_object(activation);
                self.print_object(&obj, activation);
            }
        }
    }

    pub fn print_variables<'gc>(
        &mut self,
        header: &str,
        name: &str,
        object: &Object<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        let keys = object.get_keys(activation, false);
        if keys.is_empty() {
            return;
        }

        self.output.push_str(header);
        self.output.push('\n');
        self.depth += 1;

        for key in keys.into_iter() {
            self.output.push_str(name);
            self.output.push('.');
            let _ = write!(self.output, "{key}");
            self.output.push_str(" = ");
            self.print_property(object, key, activation);
            self.output.push('\n');
        }

        self.depth -= 1;
        self.output.push('\n');
    }
}

pub struct VariableDumperJson {
    output: String,
}

impl VariableDumperJson {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn output(&mut self) -> &str {
        self.output.push('}');
        &self.output
    }

    fn print_property<'gc>(
        &mut self,
        object: &Object<'gc>,
        key: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        match object.get(key, activation) {
            Ok(value) => {
                self.print_value(&value, activation);
            }
            Err(e) => {
                self.output.push_str("\"Error\": \"");
                self.output.push_str(&e.to_string());
                self.output.push('\"');
            }
        }
    }

    fn print_properties<'gc>(
        &mut self,
        object: &Object<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        let keys = object.get_keys(activation, false);
        if keys.is_empty() {
            self.output.push_str("{}");
        } else {
            self.output.push('{');

            let mut b = false;
            for key in keys.into_iter() {
                if b {
                    self.output.push(',');
                } else {
                    b = true;
                }
                self.output.push('\"');
                self.output.push_str(&key.to_utf8_lossy());
                self.output.push_str("\":");
                self.print_property(object, key, activation);
            }

            self.output.push('}');
        }
    }

    fn print_value<'gc>(&mut self, value: &Value<'gc>, activation: &mut Activation<'_, 'gc>) {
        match value {
            Value::Undefined => self.output.push_str("[]"), // "undefined" is not valid json, {} is for empty objects, [] ? (is not conflicting with "NewObject" for "Array")
            Value::Null => self.output.push_str("null"),
            Value::Bool(value) => self.output.push_str(&value.to_string()),
            Value::Number(value) => self.output.push_str(&value.to_string()),
            Value::String(value) => {
                print_string!(self, *value);
            }
            Value::Object(object) => {
                self.print_properties(object, activation);
            }
            Value::MovieClip(_) => {
                let obj = value.coerce_to_object(activation);
                self.print_properties(&obj, activation); //object's properties
            }
        }
    }

    fn print_variables<'gc>(
        &mut self,
        name: &str,
        object: &Object<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        let keys = object.get_keys(activation, false);
        if keys.is_empty() {
            return;
        }

        if !self.output.is_empty() {
            self.output.push(',');
        } else {
            self.output.push('{');
        }
        self.output.push('\"');
        self.output.push_str(name);
        self.output.push_str("\":{");

        let mut b = false;
        for key in keys.into_iter() {
            if b {
                self.output.push(',');
            } else {
                b = true;
            }

            let _ = write!(self.output, "\"{key}\":");
            self.print_property(object, key, activation);
        }

        self.output.push('}');
    }

    pub fn print_activation(&mut self, activation: &mut Activation<'_, '_>) {
        self.print_variables(
            "_global",
            &activation.context.avm1.global_object(),
            activation,
        );
        for display_object in activation.context.stage.iter_render_list() {
            let level = display_object.depth();
            let object = display_object.object().coerce_to_object(activation);
            self.print_variables(&format!("_level{level}"), &object, activation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::error::Error;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::ScriptObject;

    #[test]
    fn dump_undefined() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            assert_eq!(
                VariableDumper::dump(&Value::Undefined, " ", activation),
                "undefined"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_null() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            assert_eq!(VariableDumper::dump(&Value::Null, " ", activation), "null");
            Ok(())
        })
    }

    #[test]
    fn dump_bool() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            assert_eq!(VariableDumper::dump(&true.into(), " ", activation), "true");
            assert_eq!(
                VariableDumper::dump(&false.into(), " ", activation),
                "false"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_number() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            assert_eq!(VariableDumper::dump(&1000.into(), " ", activation), "1000");
            assert_eq!(
                VariableDumper::dump(&(-0.05).into(), " ", activation),
                "-0.05"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_string() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            assert_eq!(VariableDumper::dump(&"".into(), " ", activation), "\"\"");
            assert_eq!(
                VariableDumper::dump(&"HELLO WORLD".into(), " ", activation),
                "\"HELLO WORLD\""
            );
            assert_eq!(
                VariableDumper::dump(
                    &"Escape \"this\" string\nplease! \u{0008}\u{000C}\n\r\t\"\\".into(),
                    " ",
                    activation,
                ),
                "\"Escape \\\"this\\\" string\\nplease! \\b\\f\\n\\r\\t\\\"\\\\\""
            );
            Ok(())
        })
    }

    #[test]
    fn dump_empty_object() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            let object = ScriptObject::new(activation.context.gc_context, None);
            assert_eq!(
                VariableDumper::dump(&object.into(), " ", activation),
                "[object #0] {}"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_object() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            let object = ScriptObject::new(activation.context.gc_context, None);
            let child = ScriptObject::new(activation.context.gc_context, None);
            object.set("self", object.into(), activation)?;
            object.set("test", "value".into(), activation)?;
            object.set("child", child.into(), activation)?;
            child.set("parent", object.into(), activation)?;
            child.set("age", 6.into(), activation)?;
            assert_eq!(
                VariableDumper::dump(&object.into(), " ", activation),
                "[object #0] {\n child: [object #1] {\n  age: 6\n  parent: [object #0]\n }\n test: \"value\"\n self: [object #0]\n}",
            );
            Ok(())
        })
    }

    #[test]
    fn dump_variables() {
        with_avm(19, |activation, _root| -> Result<(), Error> {
            let object = ScriptObject::new(activation.context.gc_context, None);
            let child = ScriptObject::new(activation.context.gc_context, None);
            object.set("self", object.into(), activation)?;
            object.set("test", "value".into(), activation)?;
            object.set("child", child.into(), activation)?;
            child.set("parent", object.into(), activation)?;
            child.set("age", 6.into(), activation)?;
            let mut dumper = VariableDumper::new(" ");
            dumper.print_variables("Variables:", "object", &object.into(), activation);
            assert_eq!(
                dumper.output,
                "Variables:\nobject.child = [object #0] {\n  age: 6\n  parent: [object #1] {\n   child: [object #0]\n   test: \"value\"\n   self: [object #1]\n  }\n }\nobject.test = \"value\"\nobject.self = [object #1]\n\n"
            );
            Ok(())
        })
    }
}
