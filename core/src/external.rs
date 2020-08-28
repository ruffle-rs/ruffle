use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::object::TObject;
use crate::avm1::Object as Avm1Object;
use crate::avm1::Value as Avm1Value;
use crate::context::UpdateContext;
use gc_arena::{Collect, CollectionContext};
use std::collections::HashMap;

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub enum Callback<'gc> {
    Avm1 {
        this: Avm1Value<'gc>,
        method: Avm1Object<'gc>,
    },
}

impl<'gc> Callback<'gc> {
    pub fn call(&self, context: &mut UpdateContext<'_, 'gc, '_>, name: &str) {
        match self {
            Callback::Avm1 { this, method } => {
                let base_clip = *context.levels.get(&0).unwrap();
                let swf_version = context.swf.version();
                let globals = context.avm1.global_object_cell();
                let mut activation = Activation::from_nothing(
                    context.reborrow(),
                    ActivationIdentifier::root("[ExternalInterface]"),
                    swf_version,
                    globals,
                    base_clip,
                );
                let this = this.coerce_to_object(&mut activation);
                let _ = method.call(name, &mut activation, this, None, &[]);
            }
        }
    }
}

pub trait ExternalInterfaceProvider {
    fn call(&self, name: &str) -> Option<()>;
}

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

    pub fn get_callback(&self, name: &str) -> Option<Callback<'gc>> {
        self.callbacks.get(name).cloned()
    }

    pub fn call_external(&self, name: &str) {
        for provider in &self.providers {
            if provider.call(name).is_some() {
                return;
            }
        }
    }

    pub fn available(&self) -> bool {
        !self.providers.is_empty()
    }
}
