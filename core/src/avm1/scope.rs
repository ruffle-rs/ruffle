//! Represents AVM1 scope chain resolution.

use std::cell::Ref;
use gc_arena::{GcCell, MutationContext};
use crate::avm1::{Object, Value};

/// Represents a scope chain for an AVM1 activation.
pub struct Scope<'gc> {
    parent: Option<GcCell<'gc, Scope<'gc>>>,
    values: GcCell<'gc, Object<'gc>>
}

unsafe impl<'gc> gc_arena::Collect for Scope<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.parent.trace(cc);
        self.values.trace(cc);
    }
}

impl<'gc> Scope<'gc> {
    /// Construct a global scope (one without a parent).
    pub fn from_global_object(globals: GcCell<'gc, Object<'gc>>) -> Scope<'gc> {
        Scope {
            parent: None,
            values: globals
        }
    }

    /// Construct a child scope (one with a parent).
    pub fn from_parent_scope(parent: GcCell<'gc, Self>, mc: MutationContext<'gc, '_>) -> Scope<'gc> {
        Scope {
            parent: Some(parent.clone()),
            values: GcCell::allocate(mc, Object::bare_object())
        }
    }

    /// Returns a reference to the current local scope object.
    pub fn locals(&self) -> Ref<Object<'gc>> {
        self.values.read()
    }

    /// Resolve a particular value in the scope chain.
    pub fn resolve(&self, name: &str) -> Value<'gc> {
        let mut current_values = Some(self.values);

        while let Some(scope) = current_values {
            if scope.read().has_property(name) {
                return scope.read().get(name);
            }

            current_values = self.parent.map(|p| p.read().values);
        }

        Value::Undefined
    }

    /// Check if a particular property in the scope chain is defined.
    pub fn is_defined(&self, name: &str) -> bool {
        let mut current_values = Some(self.values);

        while let Some(scope) = current_values {
            if scope.read().has_property(name) {
                return true;
            }

            current_values = self.parent.map(|p| p.read().values);
        }

        false
    }

    /// Set a particular value in the current scope (and all child scopes)
    pub fn define(&self, name: &str, value: Value<'gc>, mc: MutationContext<'gc, '_>) {
        self.values.write(mc).set(name, value);
    }
}