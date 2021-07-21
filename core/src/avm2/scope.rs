//! Represents AVM2 scope chain resolution.

use crate::avm2::activation::Activation;
use crate::avm2::names::Multiname;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::Ref;

/// Indicates what kind of scope a scope is.
#[derive(Copy, Clone, Debug, PartialEq, Collect)]
#[collect(no_drop)]
pub enum ScopeClass {
    /// Scope represents global or closure scope.
    GlobalOrClosure,

    /// Scope represents an object added to the scope chain with `with`.
    /// It is not inherited when closures are defined. Furthermore, a `with`
    /// scope gains the ability to be searched for dynamic properties.
    With,
}

/// Represents a scope chain for an AVM2 activation.
#[derive(Debug, Collect)]
#[collect(no_drop)]
pub struct Scope<'gc> {
    parent: Option<GcCell<'gc, Scope<'gc>>>,
    class: ScopeClass,
    values: Object<'gc>,
}

impl<'gc> Scope<'gc> {
    /// Push a scope onto the stack, producing a new scope chain that's one
    /// item longer.
    pub fn push_scope(
        scope_stack: Option<GcCell<'gc, Scope<'gc>>>,
        object: Object<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> GcCell<'gc, Self> {
        GcCell::allocate(
            mc,
            Self {
                parent: scope_stack,
                class: ScopeClass::GlobalOrClosure,
                values: object,
            },
        )
    }

    /// Construct a with scope to be used as the scope during a with block.
    ///
    /// A with block adds an object to the top of the scope chain, so unqualified
    /// references will try to resolve on that object first.
    pub fn push_with(
        scope_stack: Option<GcCell<'gc, Scope<'gc>>>,
        with_object: Object<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> GcCell<'gc, Self> {
        GcCell::allocate(
            mc,
            Scope {
                parent: scope_stack,
                class: ScopeClass::With,
                values: with_object,
            },
        )
    }

    pub fn pop_scope(&self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.parent
    }

    /// Returns a reference to the current local scope object.
    pub fn locals(&self) -> &Object<'gc> {
        &self.values
    }

    /// Returns a reference to the current global scope object.
    ///
    /// By convention, the global scope is at the bottom of the scope stack.
    pub fn globals(&self) -> Object<'gc> {
        if let Some(parent) = self.parent {
            parent.read().globals()
        } else {
            self.values
        }
    }

    /// Returns a reference to the current local scope object for mutation.
    pub fn locals_mut(&mut self) -> &mut Object<'gc> {
        &mut self.values
    }

    /// Returns a reference to the parent scope object.
    pub fn parent(&self) -> Option<Ref<Scope<'gc>>> {
        match self.parent {
            Some(ref p) => Some(p.read()),
            None => None,
        }
    }

    pub fn parent_cell(&self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.parent
    }

    /// Find an object that contains a given property in the scope stack.
    ///
    /// This function yields `None` if no such scope exists.
    pub fn find(
        &self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<Object<'gc>>, Error> {
        if let Some(qname) = self.locals().resolve_multiname(name)? {
            if self.locals().has_property(&qname)? {
                return Ok(Some(*self.locals()));
            }
        }

        if let Some(scope) = self.parent() {
            return scope.find(name, activation);
        }

        if let Some(domain) = self.locals().as_application_domain() {
            let script = domain.get_defining_script(name)?;

            if let Some((_qname, mut script)) = script {
                return Ok(Some(script.globals(&mut activation.context)?));
            }
        }

        Ok(None)
    }

    /// Resolve a particular value in the scope chain.
    ///
    /// This function yields `None` if no such scope exists to provide the
    /// property's value.
    pub fn resolve(
        &self,
        name: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Option<Value<'gc>>, Error> {
        if let Some(qname) = self.locals().resolve_multiname(name)? {
            if self.locals().has_property(&qname)? {
                return Ok(Some(self.values.get_property(
                    self.values,
                    &qname,
                    activation,
                )?));
            }
        }

        if let Some(parent) = self.parent {
            return parent.read().resolve(name, activation);
        }

        if let Some(domain) = self.locals().as_application_domain() {
            let script = domain.get_defining_script(name)?;

            if let Some((qname, mut script)) = script {
                let script_scope = script.globals(&mut activation.context)?;

                return Ok(Some(script_scope.get_property(
                    script_scope,
                    &qname,
                    activation,
                )?));
            }
        }

        //TODO: Should undefined variables halt execution?
        Ok(None)
    }
}
