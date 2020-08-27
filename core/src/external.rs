use crate::avm1::Object as Avm1Object;
use crate::avm1::Value as Avm1Value;
use gc_arena::{Collect, CollectionContext};
use std::collections::HashMap;

#[derive(Collect)]
#[collect(no_drop)]
pub enum Callback<'gc> {
    Avm1 {
        this: Avm1Value<'gc>,
        method: Avm1Object<'gc>,
    },
}

pub trait ExternalInterfaceProvider {}

#[derive(Default)]
pub struct ExternalInterface<'gc> {
    providers: Vec<Box<dyn ExternalInterfaceProvider>>,
    callbacks: HashMap<String, Callback<'gc>>,
}

unsafe impl Collect for ExternalInterface<'_> {
    #[inline]
    fn trace(&self, cc: CollectionContext) {
        self.callbacks.trace(cc);
    }
}

impl<'gc> ExternalInterface<'gc> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_provider(&mut self, provider: Box<dyn ExternalInterfaceProvider>) {
        self.providers.push(provider);
    }

    pub fn add_callback(&mut self, name: String, callback: Callback<'gc>) {
        self.callbacks.insert(name, callback);
    }

    pub fn available(&self) -> bool {
        !self.providers.is_empty()
    }
}
