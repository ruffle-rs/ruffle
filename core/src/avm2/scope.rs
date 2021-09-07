//! Represents AVM2 scope chain resolution.

use crate::avm2::activation::Activation;
use crate::avm2::domain::Domain;
use crate::avm2::names::Multiname;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, Gc, MutationContext};
use std::ops::Deref;

/// Finds a scope containing a definition
fn find_scopes<'gc>(
    scopes: &[Object<'gc>],
    name: &Multiname<'gc>,
) -> Result<Option<Object<'gc>>, Error> {
    for scope in scopes.iter().rev() {
        if let Some(_) = scope.resolve_multiname(name)? {
            return Ok(Some(*scope));
        }
    }

    Ok(None)
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
    scopes: Option<Gc<'gc, Vec<Object<'gc>>>>,
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
    pub fn chain(&self, mc: MutationContext<'gc, '_>, new_scopes: &[Object<'gc>]) -> Self {
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

    pub fn get(&self, index: usize) -> Option<Object<'gc>> {
        self.scopes
            .and_then(|scopes| scopes.deref().get(index).cloned())
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
            if let Some(scope) = find_scopes(scopes.deref(), name)? {
                return Ok(Some(scope));
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
        // First search our scopes
        if let Some(scopes) = self.scopes {
            if let Some(scope) = find_scopes(scopes.deref(), name)? {
                return Ok(Some(scope.get_property(scope, &name, activation)?));
            }
        }

        // That didn't work... let's try searching the domain now.
        if let Some((qname, mut script)) = self.domain.get_defining_script(name)? {
            let script_scope = script.globals(&mut activation.context)?;

            Ok(Some(script_scope.get_property(
                script_scope,
                &qname.into(),
                activation,
            )?))
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
    scopes: Vec<Object<'gc>>,
}

impl<'gc> ScopeStack<'gc> {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn push(&mut self, scope: Object<'gc>) {
        self.scopes.push(scope);
    }

    pub fn pop(&mut self) -> Option<Object<'gc>> {
        self.scopes.pop()
    }

    pub fn get(&self, index: usize) -> Option<Object<'gc>> {
        self.scopes.get(index).cloned()
    }

    pub fn scopes(&self) -> &[Object<'gc>] {
        &self.scopes
    }

    pub fn find(&self, name: &Multiname<'gc>) -> Result<Option<Object<'gc>>, Error> {
        find_scopes(&self.scopes, name)
    }
}
