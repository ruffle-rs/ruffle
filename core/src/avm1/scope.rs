//! Represents AVM1 scope chain resolution.

use crate::avm1::activation::Activation;
use crate::avm1::callable_value::CallableValue;
use crate::avm1::error::Error;
use crate::avm1::object::stage_object::resolve_path_property;
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
    #[collect(require_static)]
    case_sensitive: bool,
}

impl<'gc> Scope<'gc> {
    /// Construct a global scope (one without a parent).
    pub fn from_global_object(globals: Object<'gc>, case_sensitive: bool) -> Self {
        Scope {
            parent: None,
            class: ScopeClass::Global,
            values: globals,
            case_sensitive,
        }
    }

    /// Construct a child scope of another scope.
    pub fn new_local_scope(parent: Gc<'gc, Self>, mc: &Mutation<'gc>) -> Self {
        Scope {
            parent: Some(parent),
            class: ScopeClass::Local,
            values: Object::new_without_proto(mc),
            case_sensitive: parent.case_sensitive,
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
            case_sensitive: parent_scope.case_sensitive,
        }
    }

    /// Construct an arbitrary scope.
    pub fn new(parent: Gc<'gc, Self>, class: ScopeClass, with_object: Object<'gc>) -> Self {
        Scope {
            parent: Some(parent),
            class,
            values: with_object,
            case_sensitive: parent.case_sensitive,
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
        core::iter::successors(Some(scope), |scope| scope.parent)
    }

    /// Returns the class.
    pub fn class(&self) -> ScopeClass {
        self.class
    }

    /// Returns whether the global environment this scope chain belongs to is case-sensitive.
    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
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
        let mut scope = self;
        loop {
            // The current target (`this` if it's a clip, otherwise the caller's target) stands in
            // for a removed clip that no longer takes part in variable resolution.
            let values = match scope.values.as_display_object() {
                Some(clip) if clip.avm1_removed() => activation.target_clip_or_root().object1(),
                _ => Some(scope.values),
            };

            if let Some(values) = values
                && values.has_property(activation, name)
            {
                return values
                    .get(name, activation)
                    .map(|v| CallableValue::Callable(values, v));
            }

            let Some(parent) = scope.parent else { break };
            scope = Gc::as_ref(parent);
        }

        debug_assert_eq!(scope.class, ScopeClass::Global);
        // Resolve path properties (`_global`, `_root`, `_levelN`, etc.) when the scope chain has no clip at all (playerglobal).
        let value = resolve_path_property(activation.target_clip_or_root(), name, activation)
            .unwrap_or(Value::Undefined);
        Ok(CallableValue::UnCallable(value))
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
            .is_some_and(|o| o.avm1_removed());

        if !removed
            && (self.class == ScopeClass::Target || self.locals().has_property(activation, name))
        {
            // Value found on this object, so overwrite it.
            // Or we've hit the executing movie clip, so create it here.
            self.locals().set(name, value, activation)
        } else if let Some(parent) = self.parent {
            // Traverse the scope chain in search of the value.
            parent.set(name, value, activation)
        } else {
            debug_assert_eq!(self.class, ScopeClass::Global);
            // This should only happen for playerglobals; define it on the top level scope.
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

        if let Some(parent) = self.parent {
            return parent.delete(activation, name);
        }

        false
    }
}
