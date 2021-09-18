//! Represents AVM2 scope chain resolution.

use crate::avm2::activation::Activation;
use crate::avm2::domain::Domain;
use crate::avm2::names::Multiname;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, Gc, MutationContext};
use std::ops::Deref;

/// Represents a Scope that can be on either a ScopeChain or local ScopeStack.
#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct Scope<'gc> {
    /// The underlying object of this Scope
    values: Object<'gc>,

    /// Indicates whether or not this is a `with` scope.
    ///
    /// A `with` scope allows searching the dynamic properties of
    /// this scope.
    with: bool,
}

impl<'gc> Scope<'gc> {
    /// Creates a new regular Scope
    pub fn new(values: Object<'gc>) -> Self {
        Self {
            values,
            with: false,
        }
    }

    /// Creates a new `with` Scope
    pub fn new_with(values: Object<'gc>) -> Self {
        Self { values, with: true }
    }

    pub fn with(&self) -> bool {
        self.with
    }

    pub fn values(&self) -> Object<'gc> {
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
/// ScopeChain, or if we havn't created one yet (like during script initialization), you can
/// create an empty ScopeChain with only a Domain. A ScopeChain should **always** have a Domain.
///
/// ScopeChain's are copy-on-write, meaning when we chain new scopes on top of a ScopeChain, we
/// actually create a completely brand new ScopeChain. The Domain of the ScopeChain we are chaining
/// on top of will be used for the new ScopeChain.
#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct ScopeChain<'gc> {
    scopes: Option<Gc<'gc, Vec<Scope<'gc>>>>,
    domain: Domain<'gc>,
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
    pub fn chain(&self, mc: MutationContext<'gc, '_>, new_scopes: &[Scope<'gc>]) -> Self {
        if new_scopes.is_empty() {
            // If we are not actually adding any new scopes, we don't need to do anything.
            return *self;
        }
        // TODO: This current implementation is a bit expensive, but it is exactly what avmplus does, so it's good enough for now.
        match self.scopes {
            Some(scopes) => {
                // The new ScopeChain is created by cloning the scopes of this ScopeChain,
                // and pushing the new scopes on top of that.
                let mut cloned = scopes.deref().clone();
                cloned.extend_from_slice(new_scopes);
                Self {
                    scopes: Some(Gc::allocate(mc, cloned)),
                    domain: self.domain,
                }
            }
            None => {
                // We are chaining on top of an empty ScopeChain, so we don't actually
                // need to chain anything.
                Self {
                    scopes: Some(Gc::allocate(mc, new_scopes.to_vec())),
                    domain: self.domain,
                }
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<Scope<'gc>> {
        self.scopes.and_then(|scopes| scopes.get(index).cloned())
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
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<Object<'gc>>, Error> {
        // First search our scopes
        if let Some(scopes) = self.scopes {
            for (depth, scope) in scopes.iter().enumerate().rev() {
                let values = scope.values();
                if let Some(qname) = values.resolve_multiname(name)? {
                    // We search the dynamic properties if either conditions are met:
                    // 1. Scope is a `with` scope
                    // 2. We are at depth 0 (global scope)
                    //
                    // But no matter what, we always search traits first.
                    if values.has_trait(&qname)?
                        || ((scope.with() || depth == 0) && values.has_property(&qname)?)
                    {
                        return Ok(Some(values));
                    }
                }
            }
        }
        // That didn't work... let's try searching the domain now.
        if let Some((_qname, mut script)) = self.domain.get_defining_script(name)? {
            return Ok(Some(script.globals(&mut activation.context)?));
        }
        Ok(None)
    }

    pub fn resolve(
        &self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<Value<'gc>>, Error> {
        if let Some(object) = self.find(name, activation)? {
            Ok(Some(object.get_property(object, name, activation)?))
        } else {
            Ok(None)
        }
    }
}

/// Represents a ScopeStack to be used in the AVM2 activation. A new ScopeStack should be created
/// per activation. A ScopeStack allows mutations, such as pushing new scopes, or popping scopes off.
/// A ScopeStack should only ever be accessed by the activation it was created in.
#[derive(Debug, Collect, Clone)]
#[collect(no_drop)]
pub struct ScopeStack<'gc> {
    scopes: Vec<Scope<'gc>>,
}

impl<'gc> ScopeStack<'gc> {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn push(&mut self, scope: Scope<'gc>) {
        self.scopes.push(scope);
    }

    pub fn pop(&mut self) -> Option<Scope<'gc>> {
        self.scopes.pop()
    }

    pub fn get(&self, index: usize) -> Option<Scope<'gc>> {
        self.scopes.get(index).cloned()
    }

    pub fn scopes(&self) -> &[Scope<'gc>] {
        &self.scopes
    }

    /// Searches for a scope in this ScopeStack by a multiname.
    ///
    /// The `global` parameter indicates whether we are on global$init (script initializer).
    /// When the `global` parameter is true, the scope at depth 0 is considered the global scope, and is
    /// searched for dynamic properties.
    pub fn find(&self, name: &Multiname<'gc>, global: bool) -> Result<Option<Object<'gc>>, Error> {
        for (depth, scope) in self.scopes.iter().enumerate().rev() {
            let values = scope.values();
            if let Some(qname) = values.resolve_multiname(name)? {
                // We search the dynamic properties if either conditions are met:
                // 1. Scope is a `with` scope
                // 2. We are at depth 0 AND we are at global$init (script initializer).
                if values.has_trait(&qname)?
                    || ((scope.with() || (global && depth == 0)) && values.has_property(&qname)?)
                {
                    return Ok(Some(values));
                }
            }
        }
        Ok(None)
    }
}
