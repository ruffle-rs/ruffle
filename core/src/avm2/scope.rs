//! Represents AVM2 scope chain resolution.

use crate::avm2::names::{Multiname, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use enumset::EnumSet;
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

    /// Construct a closure scope to be used as the scope stack when invoking a
    /// function.
    ///
    /// This function filters With scopes from the scope chain. If all scopes
    /// are filtered, this function returns None, representing an empty scope
    /// stack.
    pub fn new_closure_scope(
        mut parent: GcCell<'gc, Self>,
        mc: MutationContext<'gc, '_>,
    ) -> Option<GcCell<'gc, Self>> {
        let mut bottom_scope = None;
        let mut top_scope: Option<GcCell<'gc, Self>> = None;

        loop {
            if parent.read().class != ScopeClass::With {
                let next_scope = GcCell::allocate(
                    mc,
                    Self {
                        parent: None,
                        class: parent.read().class,
                        values: parent.read().values,
                    },
                );

                if bottom_scope.is_none() {
                    bottom_scope = Some(next_scope);
                }

                if let Some(ref scope) = top_scope {
                    scope.write(mc).parent = Some(next_scope);
                }

                top_scope = Some(next_scope);
            }

            let grandparent = parent.read().parent;
            if let Some(grandparent) = grandparent {
                parent = grandparent;
            } else {
                break;
            }
        }

        bottom_scope
    }

    /// Construct an arbitrary scope
    pub fn new(
        parent: GcCell<'gc, Self>,
        class: ScopeClass,
        with_object: Object<'gc>,
    ) -> Scope<'gc> {
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

    /// Returns a reference to the current local scope object for mutation.
    #[allow(dead_code)]
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

    /// Find an object that contains a given property in the scope stack.
    pub fn find(
        &self,
        name: &Multiname,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Option<Object<'gc>>, Error> {
        if let Some(qname) = self.locals().resolve_multiname(name) {
            if self.locals().has_property(&qname) {
                return Ok(Some(*self.locals()));
            }
        }

        if let Some(scope) = self.parent() {
            return scope.find(name, avm, context);
        }

        //TODO: This should actually be an error.
        Ok(None)
    }

    /// Resolve a particular value in the scope chain.
    pub fn resolve(
        &self,
        name: &Multiname,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        if let Some(qname) = self.locals().resolve_multiname(name) {
            if self.locals().has_property(&qname) {
                return self.locals().get_property(&qname, avm, context);
            }
        }

        if let Some(scope) = self.parent() {
            return scope.resolve(name, avm, context);
        }

        //TODO: Should undefined variables halt execution?
        Ok(Value::Undefined.into())
    }

    /// Check if a particular property in the scope chain is defined.
    pub fn is_defined(&self, name: &QName) -> bool {
        if self.locals().has_property(name) {
            return true;
        }

        if let Some(scope) = self.parent() {
            return scope.is_defined(name);
        }

        false
    }

    /// Update a particular value in the scope chain.
    ///
    /// Traverses the scope chain in search of a value. If it's found, it's overwritten.
    /// The traversal stops at Target scopes, which represents the movie clip timeline
    /// the code is executing in.
    /// If the value is not found, it is defined on this Target scope.
    pub fn set(
        &self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error> {
        if self.locals().has_property(name) && self.locals().is_property_overwritable(name) {
            // Value found on this object, so overwrite it.
            // Or we've hit the executing movie clip, so create it here.
            self.locals().set_property(name, value, avm, context)
        } else if let Some(scope) = self.parent() {
            // Traverse the scope chain in search of the value.
            scope.set(name, value, avm, context, this)
        } else {
            // This probably shouldn't happen -- all AVM2 code runs in reference to some movieclip,
            // so we should always have a movieclip scope.
            // Define on the top-level scope.
            debug_assert!(false, "Scope::set: No top-level movie clip scope");
            self.locals().set_property(name, value, avm, context)
        }
    }

    /// Set a particular value in the locals for this scope.
    ///
    /// By convention, the locals for a given function are always defined as
    /// stored (e.g. not virtual) properties on the lowest object in the scope
    /// chain. As a result, this function always force sets a property on the
    /// local object and does not traverse the scope chain.
    pub fn define(&self, name: &QName, value: impl Into<Value<'gc>>, mc: MutationContext<'gc, '_>) {
        self.locals()
            .define_value(mc, name, value.into(), EnumSet::empty());
    }

    /// Delete a value from scope
    pub fn delete(&self, name: &Multiname, mc: MutationContext<'gc, '_>) -> bool {
        if let Some(qname) = self.locals().resolve_multiname(name) {
            if self.locals().has_property(&qname) {
                return self.locals().delete(mc, &qname);
            }
        }

        if let Some(scope) = self.parent() {
            return scope.delete(name, mc);
        }

        false
    }
}
