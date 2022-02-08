use crate::avm2::method::{MethodKind, MethodMetadata, MethodPosition};
use crate::string::WString;
use gc_arena::{Collect, MutationContext};

#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct CallStackNode<'gc> {
    meta: Option<MethodMetadata<'gc>>,
    method_id: Option<u32>,
}

impl<'gc> CallStackNode<'gc> {
    /// Creates an empty call stack node. This should only be used for nothing activations.
    pub fn empty() -> Self {
        Self {
            meta: None,
            method_id: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.meta.is_none() && self.method_id.is_none()
    }

    /// Helper method for correctly constructing script init call nodes.
    pub fn script_init() -> Self {
        Self {
            meta: Some(MethodMetadata::new_script_init()),
            method_id: None,
        }
    }

    /// Creates a new CallStackNode.
    ///
    /// It is an error for both `meta` and `method_id` to be None at the same time.
    pub fn new(meta: Option<MethodMetadata<'gc>>, method_id: Option<u32>) -> Self {
        debug_assert!(
            meta.is_some() || method_id.is_some(),
            "At least one argument must have a value"
        );
        Self { meta, method_id }
    }

    pub fn display(&self, mc: MutationContext<'gc, '_>, output: &mut WString) {
        if let Some(meta) = self.meta {
            output.push_str(&meta.class_name().to_qualified_name(mc));
            if meta.position() == MethodPosition::ClassTrait {
                output.push_char('$');
            }
            let prefix = match meta.kind() {
                MethodKind::Getter => "/get ",
                MethodKind::Setter => "/set ",
                MethodKind::Regular => "/",
                MethodKind::Initializer => "",
            };
            output.push_utf8(prefix);
            if meta.name().namespace().is_namespace() {
                output.push_str(&meta.name().to_qualified_name(mc));
            } else {
                output.push_str(&meta.name().local_name());
            }
        } else if let Some(method_id) = self.method_id {
            output.push_utf8("MethodInfo-");
            output.push_utf8(&method_id.to_string());
        }
        output.push_utf8("()");
    }
}

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct StackTrace<'gc> {
    stack: Vec<CallStackNode<'gc>>,
}

impl<'gc> StackTrace<'gc> {
    pub fn new(stack: Vec<CallStackNode<'gc>>) -> Self {
        Self { stack }
    }

    pub fn display(&self, mc: MutationContext<'gc, '_>, output: &mut WString) {
        for call in self.stack.iter() {
            if !call.is_empty() {
                output.push_utf8("\n\tat ");
                call.display(mc, output);
            }
        }
    }
}
