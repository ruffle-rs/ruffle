//! Application Domains

use crate::avm2::activation::Activation;
use crate::avm2::names::QName;
use crate::avm2::object::TObject;
use crate::avm2::script::Script;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashMap;

/// Represents a set of scripts and movies that share traits across different
/// script-global scopes.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Domain<'gc> {
    /// A list of all exported definitions and the script that exported them.
    defs: HashMap<QName<'gc>, GcCell<'gc, Script<'gc>>>,

    /// The parent domain.
    parent: Option<GcCell<'gc, Domain<'gc>>>,
}

impl<'gc> Domain<'gc> {
    /// Create a new domain with no parent.
    ///
    /// This is intended exclusively for creating the player globals domain,
    /// hence the name.
    pub fn global_domain(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Domain<'gc>> {
        GcCell::allocate(
            mc,
            Domain {
                defs: HashMap::new(),
                parent: None,
            },
        )
    }

    /// Create a new domain with a given parent.
    pub fn movie_domain(
        mc: MutationContext<'gc, '_>,
        parent: GcCell<'gc, Domain<'gc>>,
    ) -> GcCell<'gc, Domain<'gc>> {
        GcCell::allocate(
            mc,
            Domain {
                defs: HashMap::new(),
                parent: Some(parent),
            },
        )
    }

    /// Determine if something has been defined within the current domain.
    pub fn has_definition(&self, name: QName<'gc>) -> bool {
        if self.defs.contains_key(&name) {
            return true;
        }

        if let Some(parent) = self.parent {
            return parent.read().has_definition(name);
        }

        false
    }

    /// Return the script that has exported a given definition.
    pub fn get_defining_script(&self, name: QName<'gc>) -> Option<GcCell<'gc, Script<'gc>>> {
        if self.defs.contains_key(&name) {
            return self.defs.get(&name).cloned();
        }

        if let Some(parent) = self.parent {
            return parent.read().get_defining_script(name);
        }

        None
    }

    /// Retrieve a value from this domain.
    pub fn get_defined_value(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: QName<'gc>,
    ) -> Result<Value<'gc>, Error> {
        let script = self
            .get_defining_script(name.clone())
            .ok_or_else(|| format!("MovieClip Symbol {} does not exist", name.local_name()))?;
        let mut globals = script.read().globals();

        globals.get_property(globals, &name, activation)
    }

    /// Export a definition from a script into the current application domain.
    ///
    /// This returns an error if the name is already defined in the current or
    /// any parent domains.
    pub fn export_definition(
        &mut self,
        name: QName<'gc>,
        script: GcCell<'gc, Script<'gc>>,
    ) -> Result<(), Error> {
        if self.has_definition(name.clone()) {
            return Err(format!(
                "VerifyError: Attempted to redefine existing name {}",
                name.local_name()
            )
            .into());
        }

        self.defs.insert(name, script);

        Ok(())
    }
}
