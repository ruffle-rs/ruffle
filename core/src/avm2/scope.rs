//! Represents AVM2 scope chain resolution.

use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use core::fmt;
use gc_arena::{Collect, Gc, Mutation};

/// Represents a Scope that can be on either a ScopeChain or local ScopeStack.
#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct Scope<'gc> {
    /// The underlying object of this Scope
    values: Value<'gc>,

    /// Indicates whether or not this is a `with` scope.
    ///
    /// A `with` scope allows searching the dynamic properties of
    /// this scope.
    with: bool,
}

impl<'gc> Scope<'gc> {
    /// Creates a new regular Scope.
    ///
    /// It is the caller's responsibility to ensure that the `values` passed
    /// to this method is not Value::Null or Value::Undefined.
    pub fn new(values: Value<'gc>) -> Self {
        Self {
            values,
            with: false,
        }
    }

    /// Creates a new `with` Scope.
    ///
    /// It is the caller's responsibility to ensure that the `values` passed
    /// to this method is not Value::Null or Value::Undefined.
    pub fn new_with(values: Value<'gc>) -> Self {
        Self { values, with: true }
    }

    pub fn with(&self) -> bool {
        self.with
    }

    pub fn values(&self) -> Value<'gc> {
        self.values
    }
}

/// A ScopeChain "chains" scopes together.
///
/// A ScopeChain is used for "remembering" what a scope looked like. A ScopeChain also
/// contains an associated Domain that should be the domain that was in use during it's
/// initial creation.
///
/// A ScopeChain is either created by chaining new scopes on top of an already existing
/// ScopeChain, or if we haven't created one yet (like during script initialization), you can
/// create an empty ScopeChain with only a Domain. A ScopeChain should **always** have a Domain.
///
/// ScopeChain's are copy-on-write, meaning when we chain new scopes on top of a ScopeChain, we
/// actually create a completely brand new ScopeChain. The Domain of the ScopeChain we are chaining
/// on top of will be used for the new ScopeChain.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct ScopeChain<'gc> {
    scopes: Option<Gc<'gc, Vec<Scope<'gc>>>>,
    domain: Domain<'gc>,
}

impl fmt::Debug for ScopeChain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeChain")
            .field("scopes", &self.scopes)
            .finish()
    }
}

impl<'gc> ScopeChain<'gc> {
    /// Creates a brand new ScopeChain with a domain. The domain should be the current domain in use.
    pub fn new(domain: Domain<'gc>) -> Self {
        Self {
            scopes: None,
            domain,
        }
    }

    /// Creates a new ScopeChain by chaining new scopes on top of this ScopeChain
    pub fn chain(&self, mc: &Mutation<'gc>, new_scopes: &[Scope<'gc>]) -> Self {
        if new_scopes.is_empty() {
            // If we are not actually adding any new scopes, we don't need to do anything.
            return *self;
        }
        // TODO: This current implementation is a bit expensive, but it is exactly what avmplus does, so it's good enough for now.
        match self.scopes {
            Some(scopes) => {
                // The new ScopeChain is created by cloning the scopes of this ScopeChain,
                // and pushing the new scopes on top of that.
                let mut cloned = (*scopes).clone();
                cloned.extend_from_slice(new_scopes);
                Self {
                    scopes: Some(Gc::new(mc, cloned)),
                    domain: self.domain,
                }
            }
            None => {
                // We are chaining on top of an empty ScopeChain, so we don't actually
                // need to chain anything.
                Self {
                    scopes: Some(Gc::new(mc, new_scopes.to_vec())),
                    domain: self.domain,
                }
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<Scope<'gc>> {
        self.scopes.and_then(|scopes| scopes.get(index).copied())
    }

    /// Like `get`, but panics if the scopes doesn't exist or
    /// the scope index is out of bounds.
    pub fn get_unchecked(&self, index: usize) -> Scope<'gc> {
        self.scopes.unwrap()[index]
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.map(|scopes| scopes.is_empty()).unwrap_or(true)
    }

    /// Returns the domain associated with this ScopeChain.
    pub fn domain(&self) -> Domain<'gc> {
        self.domain
    }

    pub fn find(
        &self,
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>> {
        if let Some(scopes) = self.scopes {
            // We skip the scope at depth 0 (the global scope). The global scope will be checked in a different phase.
            for scope in scopes.iter().skip(1).rev() {
                let values = scope.values();
                let vtable = values.vtable(activation);

                // TODO: Can we switch this to `VTable::has_trait`?
                if vtable.get_trait(multiname).is_some() {
                    return Ok(Some(values));
                }

                // Wasn't in the objects traits, let's try dynamic properties if this is a with scope.
                if scope.with() && values.has_own_property(activation, multiname) {
                    return Ok(Some(values));
                }
            }
        }
        // That didn't work... let's try searching the domain now.
        if let Some((_, script)) = self.domain.get_defining_script(multiname) {
            return Ok(Some(script.globals(activation.context)?.into()));
        }
        Ok(None)
    }

    pub fn get_entry_for_multiname(
        &self,
        activation: &mut Activation<'_, 'gc>,
        multiname: &Multiname<'gc>,
    ) -> Option<Option<(Class<'gc>, usize)>> {
        if let Some(scopes) = self.scopes {
            for (index, scope) in scopes.iter().enumerate().skip(1).rev() {
                if scope.with() {
                    // If this is a `with` scope, stop here because
                    // dynamic properties could be added at any time
                    return Some(None);
                }

                let values = scope.values();
                if values.has_trait(activation, multiname) {
                    return Some(Some((values.instance_class(activation), index)));
                }
            }
        }

        // Nothing was found, and we can be sure that nothing will be
        // found here at all (there were no `with` scopes).
        None
    }
}

/// Searches for a scope in the scope stack by a multiname.
///
/// The `global` parameter indicates whether we are on global$init (script initializer).
/// When the `global` parameter is true, the scope at depth 0 is considered the global scope, and is skipped.
pub fn search_scope_stack<'gc>(
    activation: &mut Activation<'_, 'gc>,
    multiname: &Multiname<'gc>,
    global: bool,
) -> Option<Value<'gc>> {
    let classes = activation.context.avm2.classes();

    let scopes = activation.scope_frame();

    for (depth, scope) in scopes.iter().enumerate().rev() {
        if depth == 0 && global {
            continue;
        }
        let values = scope.values();

        if value_has_trait(classes, values, multiname) {
            return Some(values);
        } else if scope.with() {
            // We search the dynamic properties if this is a with scope.
            if value_has_own_property(classes, values, multiname) {
                return Some(values);
            }
        }
    }
    None
}

use crate::avm2::globals::SystemClasses;

// Check if a Value has a trait without using an Activation. Used only in `search_scope_stack`.
fn value_has_trait<'gc>(
    classes: &SystemClasses<'gc>,
    value: Value<'gc>,
    multiname: &Multiname<'gc>,
) -> bool {
    let vtable = match value {
        Value::Bool(_) => classes.boolean.instance_vtable(),
        Value::Number(_) | Value::Integer(_) => classes.number.instance_vtable(),
        Value::String(_) => classes.string.instance_vtable(),
        Value::Object(obj) => obj.vtable(),

        Value::Undefined | Value::Null => {
            unreachable!("Should not have Undefined or Null scope")
        }
    };

    vtable.has_trait(multiname)
}

// Check if a Value has a property without using an Activation. Used only in `search_scope_stack`.
fn value_has_own_property<'gc>(
    classes: &SystemClasses<'gc>,
    value: Value<'gc>,
    multiname: &Multiname<'gc>,
) -> bool {
    let vtable = match value {
        Value::Bool(_) => classes.boolean.instance_vtable(),
        Value::Number(_) | Value::Integer(_) => classes.number.instance_vtable(),
        Value::String(_) => classes.string.instance_vtable(),
        Value::Object(obj) => obj.vtable(),

        Value::Undefined | Value::Null => {
            unreachable!("Should not have Undefined or Null scope")
        }
    };

    match value {
        Value::Object(object) => object.has_own_property(multiname),
        _ => vtable.has_trait(multiname),
    }
}
