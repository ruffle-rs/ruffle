//! Represents AVM1 scope chain resolution.

use std::cell::Ref;
use gc_arena::{GcCell, MutationContext};
use crate::avm1::{Object, Value};

/// Represents a scope chain for an AVM1 activation.
#[derive(Debug)]
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

    /// Construct a child scope of another scope.
    pub fn from_parent_scope(parent: GcCell<'gc, Self>, mc: MutationContext<'gc, '_>) -> Scope<'gc> {
        Scope {
            parent: Some(parent.clone()),
            values: GcCell::allocate(mc, Object::bare_object())
        }
    }

    /// Construct a child scope with a given object
    /// 
    /// Rejected titles: `from_oyako_scope`
    pub fn from_parent_scope_with_object(parent: GcCell<'gc, Self>, with_object: GcCell<'gc, Object<'gc>>) -> Scope<'gc> {
        Scope {
            parent: Some(parent.clone()),
            values: with_object
        }
    }

    /// Returns a reference to the current local scope object.
    pub fn locals(&self) -> Ref<Object<'gc>> {
        self.values.read()
    }

    /// Returns a reference to the parent scope object.
    pub fn parent(&self) -> Option<Ref<Scope<'gc>>> {
        match self.parent {
            Some(ref p) => Some(p.read()),
            None => None
        }
    }

    /// Resolve a particular value in the scope chain.
    pub fn resolve(&self, name: &str) -> Value<'gc> {
        if self.locals().has_property(name) {
            return self.locals().get(name);
        }
        
        if let Some(scope) = self.parent() {
            return scope.resolve(name);
        }

        Value::Undefined
    }

    /// Check if a particular property in the scope chain is defined.
    pub fn is_defined(&self, name: &str) -> bool {
        if self.locals().has_property(name) {
            return true;
        }

        if let Some(scope) = self.parent() {
            return scope.is_defined(name);
        }

        false
    }

    /// Set a particular value in the current scope (and all child scopes)
    pub fn define(&self, name: &str, value: Value<'gc>, mc: MutationContext<'gc, '_>) {
        self.values.write(mc).set(name, value);
    }
}