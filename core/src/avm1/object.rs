use crate::avm1::Value;
use gc_arena::Collect;
use std::collections::HashMap;

#[derive(Clone, Debug, Collect, Default)]
#[collect(empty_drop)]
pub struct Object<'gc> {
    values: HashMap<String, Value<'gc>>,
}

impl<'gc> Object<'gc> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Value<'gc> {
        self.values
            .get(name)
            .map_or(Value::Undefined, |v| v.to_owned())
    }

    pub fn set(&mut self, name: &str, value: Value<'gc>) {
        self.values.insert(name.to_owned(), value);
    }
}
