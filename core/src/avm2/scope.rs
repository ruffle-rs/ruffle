//! Represents AVM2 scope chain resolution.

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::{Multiname, Namespace};
use core::fmt;
use gc_arena::barrier::field;
use gc_arena::lock::RefLock;
use gc_arena::{Collect, Gc, Mutation};

use super::property_map::PropertyMap;

/// Represents a Scope that can be on either a ScopeChain or local ScopeStack.
#[derive(Collect, Clone, Copy, Debug)]
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

/// Internal container that a ScopeChain uses
#[derive(Collect, Clone, Debug)]
#[collect(no_drop)]
struct ScopeContainer<'gc> {
    /// The scopes of this ScopeChain
    scopes: Vec<Scope<'gc>>,

    /// The cache of this ScopeChain. A value of None indicates that caching is disabled
    /// for this ScopeChain.
    cache: Option<RefLock<PropertyMap<'gc, Object<'gc>>>>,
}

impl<'gc> ScopeContainer<'gc> {
    fn new(scopes: Vec<Scope<'gc>>) -> Self {
        let cache = (!scopes.iter().any(|scope| scope.with)).then(RefLock::default);
        Self { scopes, cache }
    }

    fn get(&self, index: usize) -> Option<Scope<'gc>> {
        self.scopes.get(index).cloned()
    }

    /// Like `get`, but panics if the scope index is out of bounds.
    pub fn get_unchecked(&self, index: usize) -> Scope<'gc> {
        self.scopes[index]
    }

    fn is_empty(&self) -> bool {
        self.scopes.is_empty()
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
    container: Option<Gc<'gc, ScopeContainer<'gc>>>,
    domain: Domain<'gc>,
}

impl fmt::Debug for ScopeChain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeChain")
            .field("container", &self.container)
            .finish()
    }
}

impl<'gc> ScopeChain<'gc> {
    /// Creates a brand new ScopeChain with a domain. The domain should be the current domain in use.
    pub fn new(domain: Domain<'gc>) -> Self {
        Self {
            container: None,
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
        match self.container {
            Some(container) => {
                // The new ScopeChain is created by cloning the scopes of this ScopeChain,
                // and pushing the new scopes on top of that.
                let mut cloned = container.scopes.clone();
                cloned.extend_from_slice(new_scopes);
                Self {
                    container: Some(Gc::new(mc, ScopeContainer::new(cloned))),
                    domain: self.domain,
                }
            }
            None => {
                // We are chaining on top of an empty ScopeChain, so we don't actually
                // need to chain anything.
                Self {
                    container: Some(Gc::new(mc, ScopeContainer::new(new_scopes.to_vec()))),
                    domain: self.domain,
                }
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<Scope<'gc>> {
        self.container.and_then(|container| container.get(index))
    }

    /// Like `get`, but panics if the container doesn't exist or
    /// the scope index is out of bounds.
    pub fn get_unchecked(&self, index: usize) -> Scope<'gc> {
        self.container.unwrap().get_unchecked(index)
    }

    pub fn is_empty(&self) -> bool {
        self.container
            .map(|container| container.is_empty())
            .unwrap_or(true)
    }

    /// Returns the domain associated with this ScopeChain.
    pub fn domain(&self) -> Domain<'gc> {
        self.domain
    }

    fn find_internal(
        &self,
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<(Option<Namespace<'gc>>, Object<'gc>)>, Error<'gc>> {
        if let Some(container) = self.container {
            // We skip the scope at depth 0 (the global scope). The global scope will be checked in a different phase.
            for scope in container.scopes.iter().skip(1).rev() {
                // NOTE: We are manually searching the vtable's traits so we can figure out which namespace the trait
                // belongs to.
                let values = scope.values();
                let vtable = values.vtable();
                if let Some((namespace, _)) = vtable.get_trait_with_ns(multiname) {
                    return Ok(Some((Some(namespace), values)));
                }

                // Wasn't in the objects traits, let's try dynamic properties if this is a with scope.
                if scope.with() && values.has_own_property(multiname) {
                    // NOTE: We return the QName as `None` to indicate that we should never cache this result.
                    // We NEVER cache the result of dynamic properties.
                    return Ok(Some((None, values)));
                }
            }
        }
        // That didn't work... let's try searching the domain now.
        if let Some((qname, script)) = self.domain.get_defining_script(multiname)? {
            return Ok(Some((
                Some(qname.namespace()),
                script.globals(activation.context)?,
            )));
        }
        Ok(None)
    }

    pub fn find(
        &self,
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Object<'gc>>, Error<'gc>> {
        // First we check the cache of our container
        if let Some(container) = self.container {
            if let Some(cache) = &container.cache {
                if let Some(cached) = cache.borrow().get_for_multiname(multiname) {
                    return Ok(Some(*cached));
                }
            }
        }
        let found = self.find_internal(multiname, activation)?;
        if let (Some((Some(ns), obj)), Some(container)) = (found, self.container) {
            // We found a value that hasn't been cached yet, so let's try to cache it now
            let cache = field!(Gc::write(activation.gc(), container), ScopeContainer, cache);
            if let Some(cache) = cache.as_write() {
                let name = multiname
                    .local_name()
                    .expect("Resolvable multinames should always have a local name");
                cache
                    .unlock()
                    .borrow_mut()
                    .insert_with_namespace(ns, name, obj);
            }
        }
        Ok(found.map(|o| o.1))
    }

    pub fn get_entry_for_multiname(
        &self,
        multiname: &Multiname<'gc>,
    ) -> Option<Option<(Class<'gc>, u32)>> {
        if let Some(container) = self.container {
            for (index, scope) in container.scopes.iter().enumerate().skip(1).rev() {
                if scope.with() {
                    // If this is a `with` scope, stop here because
                    // dynamic properties could be added at any time
                    return Some(None);
                }

                let values = scope.values();
                if values.has_trait(multiname) {
                    return Some(Some((values.instance_class(), index as u32)));
                }
            }
        }

        // Nothing was found, and we can be sure that nothing will be
        // found here at all (there were no `with` scopes).
        None
    }

    pub fn resolve(
        &self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>> {
        if let Some(object) = self.find(name, activation)? {
            Ok(Some(object.get_property(name, activation)?))
        } else {
            Ok(None)
        }
    }
}

/// Searches for a scope in the scope stack by a multiname.
///
/// The `global` parameter indicates whether we are on global$init (script initializer).
/// When the `global` parameter is true, the scope at depth 0 is considered the global scope, and is skipped.
pub fn search_scope_stack<'gc>(
    scopes: &[Scope<'gc>],
    multiname: &Multiname<'gc>,
    global: bool,
) -> Result<Option<Object<'gc>>, Error<'gc>> {
    for (depth, scope) in scopes.iter().enumerate().rev() {
        if depth == 0 && global {
            continue;
        }
        let values = scope.values();

        if values.has_trait(multiname) {
            return Ok(Some(values));
        } else if scope.with() {
            // We search the dynamic properties if this is a with scope.
            if values.has_own_property(multiname) {
                return Ok(Some(values));
            }
        }
    }
    Ok(None)
}
