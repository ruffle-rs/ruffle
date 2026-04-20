//! Represents AVM1 scope chain resolution.

use crate::avm1::activation::Activation;
use crate::avm1::callable_value::CallableValue;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, Value};
use crate::display_object::TDisplayObject;
use crate::string::AvmString;
use gc_arena::{Collect, Gc, Mutation};

/// Indicates what kind of scope a scope is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeClass {
    /// Scope represents global scope.
    Global,

    /// Target represents timeline scope. All timeline actions execute with
    /// the current clip object in lieu of a local scope, and the timeline scope
    /// can be changed via `tellTarget`.
    Target,

    /// Scope represents local scope and is inherited when a closure is defined.
    Local,

    /// Scope represents an object added to the scope chain with `with`.
    /// It is not inherited when closures are defined.
    With,
}

/// Represents a scope chain for an AVM1 activation.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Scope<'gc> {
    parent: Option<Gc<'gc, Scope<'gc>>>,
    #[collect(require_static)]
    class: ScopeClass,
    values: Object<'gc>,
}

impl<'gc> Scope<'gc> {
    /// Construct a global scope (one without a parent).
    pub fn from_global_object(globals: Object<'gc>) -> Self {
        Scope {
            parent: None,
            class: ScopeClass::Global,
            values: globals,
        }
    }

    /// Construct a child scope of another scope.
    pub fn new_local_scope(parent: Gc<'gc, Self>, mc: &Mutation<'gc>) -> Self {
        Scope {
            parent: Some(parent),
            class: ScopeClass::Local,
            values: Object::new_without_proto(mc),
        }
    }

    /// Construct a scope for use with `tellTarget` code where the timeline
    /// scope has been replaced with another given object.
    pub fn new_target_scope(
        parent: Gc<'gc, Self>,
        clip: Object<'gc>,
        mc: &Mutation<'gc>,
    ) -> Gc<'gc, Self> {
        let mut scope = (*parent).clone();

        if scope.class == ScopeClass::Target {
            scope.values = clip;
        } else {
            scope.parent = scope.parent.map(|p| Self::new_target_scope(p, clip, mc));
        }

        Gc::new(mc, scope)
    }

    /// Construct a with scope to be used as the scope during a with block.
    ///
    /// A with block adds an object to the top of the scope chain, so unqualified
    /// references will try to resolve on that object first.
    pub fn new_with_scope(parent_scope: Gc<'gc, Self>, with_object: Object<'gc>) -> Self {
        Scope {
            parent: Some(parent_scope),
            class: ScopeClass::With,
            values: with_object,
        }
    }

    /// Construct an arbitrary scope.
    pub fn new(parent: Gc<'gc, Self>, class: ScopeClass, with_object: Object<'gc>) -> Self {
        Scope {
            parent: Some(parent),
            class,
            values: with_object,
        }
    }

    /// Returns a reference to the current local scope object.
    pub fn locals(&self) -> &Object<'gc> {
        &self.values
    }

    /// Returns a reference to the current local scope object.
    pub fn locals_cell(&self) -> Object<'gc> {
        self.values
    }

    /// Returns a reference to the parent scope object.
    pub fn parent(&self) -> Option<Gc<'gc, Scope<'gc>>> {
        self.parent
    }

    /// Produces first the scope itself, then its ancestors
    pub fn ancestors(scope: Gc<'gc, Scope<'gc>>) -> impl Iterator<Item = Gc<'gc, Scope<'gc>>> {
        core::iter::successors(Some(scope), |scope| scope.parent())
    }

    /// Returns the class.
    pub fn class(&self) -> ScopeClass {
        self.class
    }

    /// Resolve a particular value in the scope chain and the object which this value would expect as its `this` parameter if called.
    ///
    /// Because scopes are object chains, the same rules for `Object::get`
    /// still apply here. This function is allowed to yield `None` to indicate
    /// that the result will be calculated on the AVM stack.
    pub fn resolve(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<CallableValue<'gc>, Error<'gc>> {
        self.resolve_recursive(name, activation, true)
    }

    /// Recursively resolve a value on the scope chain
    /// See [`Scope::resolve`] for details
    fn resolve_recursive(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
        top_level: bool,
    ) -> Result<CallableValue<'gc>, Error<'gc>> {
        if self.locals().has_property(activation, name) {
            return self
                .locals()
                .get(name, activation)
                .map(|v| CallableValue::Callable(self.locals_cell(), v));
        }
        if let Some(scope) = self.parent() {
            let res = scope.resolve(name, activation)?;

            // If we failed to find the value in the scope chain, but it *would* resolve on `self.locals()` if it wasn't
            // a removed clip, then try resolving on root instead
            if top_level
                && matches!(res, CallableValue::UnCallable(Value::Undefined))
                && self.locals().has_own_property(activation, name)
            {
                return activation
                    .base_clip()
                    .avm1_root()
                    .object1_or_bare(activation.gc())
                    .get(name, activation)
                    .map(|v| CallableValue::Callable(self.locals_cell(), v));
            }

            Ok(res)
        } else {
            Ok(CallableValue::UnCallable(Value::Undefined))
        }
    }

    /// Update a particular value in the scope chain.
    ///
    /// Traverses the scope chain in search of a value. If it's found, it's overwritten.
    /// The traversal stops at Target scopes, which represents the movie clip timeline
    /// the code is executing in.
    /// If the value is not found, it is defined on this Target scope.
    pub fn set(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let removed = self
            .values
            .as_display_object()
            .map(|o| o.avm1_removed())
            .unwrap_or_default();

        if !removed
            && (self.class == ScopeClass::Target || self.locals().has_property(activation, name))
        {
            // Value found on this object, so overwrite it.
            // Or we've hit the executing movie clip, so create it here.
            self.locals().set(name, value, activation)
        } else if let Some(scope) = self.parent() {
            // Traverse the scope chain in search of the value.
            scope.set(name, value, activation)
        } else {
            // This probably shouldn't happen -- all AVM1 code runs in reference to some MovieClip,
            // so we should always have a MovieClip scope.
            // Define on the top-level scope.
            debug_assert!(false, "Scope::set: No top-level movie clip scope");
            self.locals().set(name, value, activation)
        }
    }

    /// Define a named local variable on the scope.
    ///
    /// If the property does not already exist on the local scope, it will be created.
    /// Otherwise, the existing property will be set to `value`. This does not crawl the scope
    /// chain. Any properties with the same name deeper in the scope chain will be shadowed.
    pub fn define_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        // When defining a local in a with scope, we first need to check if that local already exists on the with target
        // If it does, then the property of the target itself should be modified
        // If it doesn't, then the property should be defined in the first non-with parent scope
        if let (ScopeClass::With, Some(parent)) = (self.class, self.parent) {
            // Does this property already exist on the target?
            if self.locals().has_own_property(activation, name) {
                self.locals().set(name, value, activation)
            } else {
                // Otherwise, carry up the scope chain
                parent.define_local(name, value, activation)
            }
        } else {
            self.locals().set(name, value, activation)
        }
    }

    /// Create a local property on the activation.
    ///
    /// This inserts a value as a stored property on the local scope. If the property already
    /// exists, it will be forcefully overwritten. Used internally to initialize objects.
    pub fn force_define_local(&self, name: AvmString<'gc>, value: Value<'gc>, mc: &Mutation<'gc>) {
        self.locals()
            .define_value(mc, name, value, Attribute::empty());
    }

    /// Delete a value from scope.
    pub fn delete(&self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        if self.locals().has_property(activation, name) {
            return self.locals().delete(activation, name);
        }

        if let Some(scope) = self.parent() {
            return scope.delete(activation, name);
        }

        false
    }
}
