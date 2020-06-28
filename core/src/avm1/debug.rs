use crate::avm1::stack_frame::StackFrame;
use crate::avm1::{Object, ObjectPtr, TObject, Value};
use crate::context::UpdateContext;

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
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> String {
        let mut dumper = VariableDumper::new(indent);
        dumper.print_value(value, activation, context);
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

    pub fn print_string(&mut self, string: &str) {
        self.output.push_str("\"");

        for c in string.chars() {
            if c == '"' {
                self.output.push_str("\\\"");
            } else if c == '\\' {
                self.output.push_str("\\\\");
            } else if c == '\n' {
                self.output.push_str("\\n");
            } else if c == '\r' {
                self.output.push_str("\\r");
            } else if c == '\t' {
                self.output.push_str("\\t");
            } else if c == '\u{0008}' {
                self.output.push_str("\\b");
            } else if c == '\u{000C}' {
                self.output.push_str("\\f");
            } else {
                self.output.push(c);
            }
        }

        self.output.push_str("\"");
    }

    pub fn print_object<'gc>(
        &mut self,
        object: &Object<'gc>,
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        let (id, new) = self.object_id(object);
        self.output.push_str("[object #");
        self.output.push_str(&id.to_string());
        self.output.push_str("]");

        if new {
            self.print_properties(object, activation, context);
        }
    }

    pub fn print_property<'gc>(
        &mut self,
        object: &Object<'gc>,
        key: &str,
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        match object.get(&key, activation, context) {
            Ok(value) => {
                self.print_value(&value, activation, context);
            }
            Err(e) => {
                self.output.push_str("Error: \"");
                self.output.push_str(&e.to_string());
                self.output.push_str("\"");
            }
        }
    }

    pub fn print_properties<'gc>(
        &mut self,
        object: &Object<'gc>,
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        let keys = object.get_keys(activation);
        if keys.is_empty() {
            self.output.push_str(" {}");
        } else {
            self.output.push_str(" {\n");
            self.depth += 1;

            for key in keys.into_iter() {
                self.indent();
                self.output.push_str(&key);
                self.output.push_str(": ");
                self.print_property(object, &key, activation, context);
                self.output.push_str("\n");
            }

            self.depth -= 1;
            self.indent();
            self.output.push_str("}");
        }
    }

    pub fn print_value<'gc>(
        &mut self,
        value: &Value<'gc>,
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        match value {
            Value::Undefined => self.output.push_str("undefined"),
            Value::Null => self.output.push_str("null"),
            Value::Bool(value) => self.output.push_str(&value.to_string()),
            Value::Number(value) => self.output.push_str(&value.to_string()),
            Value::String(value) => {
                self.print_string(value);
            }
            Value::Object(object) => {
                self.print_object(object, activation, context);
            }
        }
    }

    pub fn print_variables<'gc>(
        &mut self,
        header: &str,
        name: &str,
        object: &Object<'gc>,
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        let keys = object.get_keys(activation);
        if keys.is_empty() {
            return;
        }

        self.output.push_str(header);
        self.output.push_str("\n");
        self.depth += 1;

        for key in keys.into_iter() {
            self.output.push_str(&format!("{}.{}", name, key));
            self.output.push_str(" = ");
            self.print_property(object, &key, activation, context);
            self.output.push_str("\n");
        }

        self.depth -= 1;
        self.output.push_str("\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::error::Error;
    use crate::avm1::function::Executable;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::ScriptObject;
    use enumset::EnumSet;

    fn throw_error<'gc>(
        _activation: &mut StackFrame<'_, 'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        Err(Error::PrototypeRecursionLimit)
    }

    #[test]
    fn dump_undefined() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            assert_eq!(
                VariableDumper::dump(&Value::Undefined, " ", activation, context),
                "undefined"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_null() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            assert_eq!(
                VariableDumper::dump(&Value::Null, " ", activation, context),
                "null"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_bool() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            assert_eq!(
                VariableDumper::dump(&Value::Bool(true), " ", activation, context),
                "true"
            );
            assert_eq!(
                VariableDumper::dump(&Value::Bool(false), " ", activation, context),
                "false"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_number() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            assert_eq!(
                VariableDumper::dump(&Value::Number(1000.0), " ", activation, context),
                "1000"
            );
            assert_eq!(
                VariableDumper::dump(&Value::Number(-0.05), " ", activation, context),
                "-0.05"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_string() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            assert_eq!(
                VariableDumper::dump(&Value::String("".to_string()), " ", activation, context),
                "\"\""
            );
            assert_eq!(
                VariableDumper::dump(
                    &Value::String("HELLO WORLD".to_string()),
                    " ",
                    activation,
                    context
                ),
                "\"HELLO WORLD\""
            );
            assert_eq!(
                VariableDumper::dump(
                    &Value::String(
                        "Escape \"this\" string\nplease! \u{0008}\u{000C}\n\r\t\"\\".to_string()
                    ),
                    " ",
                    activation,
                    context
                ),
                "\"Escape \\\"this\\\" string\\nplease! \\b\\f\\n\\r\\t\\\"\\\\\""
            );
            Ok(())
        })
    }

    #[test]
    fn dump_empty_object() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            let object = ScriptObject::object(context.gc_context, None);
            assert_eq!(
                VariableDumper::dump(&object.into(), " ", activation, context),
                "[object #0] {}"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_object() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            let object = ScriptObject::object(context.gc_context, None);
            let child = ScriptObject::object(context.gc_context, None);
            object.set("self", object.into(), activation, context)?;
            object.set(
                "test",
                Value::String("value".to_string()),
                activation,
                context,
            )?;
            object.set("child", child.into(), activation, context)?;
            child.set("parent", object.into(), activation, context)?;
            child.set("age", Value::Number(6.0), activation, context)?;
            assert_eq!(
                VariableDumper::dump(&object.into(), " ", activation, context),
                "[object #0] {\n child: [object #1] {\n  age: 6\n  parent: [object #0]\n }\n test: \"value\"\n self: [object #0]\n}",
            );
            Ok(())
        })
    }

    #[test]
    fn dump_object_with_error() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            let object = ScriptObject::object(context.gc_context, None);
            object.add_property(
                context.gc_context,
                "broken_value",
                Executable::Native(throw_error),
                None,
                EnumSet::empty(),
            );
            assert_eq!(
                VariableDumper::dump(&object.into(), " ", activation, context),
                "[object #0] {\n broken_value: Error: \"Prototype recursion limit has been exceeded\"\n}"
            );
            Ok(())
        })
    }

    #[test]
    fn dump_variables() {
        with_avm(19, |activation, context, _root| -> Result<(), Error> {
            let object = ScriptObject::object(context.gc_context, None);
            let child = ScriptObject::object(context.gc_context, None);
            object.set("self", object.into(), activation, context)?;
            object.set(
                "test",
                Value::String("value".to_string()),
                activation,
                context,
            )?;
            object.set("child", child.into(), activation, context)?;
            child.set("parent", object.into(), activation, context)?;
            child.set("age", Value::Number(6.0), activation, context)?;
            let mut dumper = VariableDumper::new(" ");
            dumper.print_variables("Variables:", "object", &object.into(), activation, context);
            assert_eq!(
                dumper.output,
                "Variables:\nobject.child = [object #0] {\n  age: 6\n  parent: [object #1] {\n   child: [object #0]\n   test: \"value\"\n   self: [object #1]\n  }\n }\nobject.test = \"value\"\nobject.self = [object #1]\n\n"
            );
            Ok(())
        })
    }
}
