use crate::avm1::activation::Activation;
use crate::avm1::{Object, ObjectPtr, Value};
use crate::string::AvmString;
use std::fmt::Write;

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

    #[expect(dead_code)]
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
            if std::ptr::eq(*other, ptr) {
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
        self.output.push('\"');

        for c in string.chars() {
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
                    self.output.push(c);
                    continue;
                }
            };

            self.output.push_str(escape);
        }

        self.output.push('\"');
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
