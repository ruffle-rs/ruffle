use crate::avm2::method::{MethodKind, MethodMetadata, MethodPosition};
use crate::string::WString;
use gc_arena::{Collect, MutationContext};

#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub enum CallStackNode<'gc> {
    Meta(MethodMetadata<'gc>),
    Id(u32),
    Empty,
}

impl<'gc> CallStackNode<'gc> {
    /// Creates an empty call stack node. This should only be used for nothing activations.
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Helper method for correctly constructing script init call nodes.
    pub fn script_init() -> Self {
        Self::Meta(MethodMetadata::new_script_init())
    }

    pub fn display(&self, mc: MutationContext<'gc, '_>, output: &mut WString) {
        match self {
            Self::Meta(meta) => {
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
            }
            Self::Id(id) => {
                output.push_utf8("MethodInfo-");
                output.push_utf8(&id.to_string());
            }
            Self::Empty => (),
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
