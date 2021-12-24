use crate::avm2::function::Executable;
use crate::string::WString;
use gc_arena::{Collect, MutationContext};

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub enum CallNode<'gc> {
    GlobalInit,
    Method(Executable<'gc>),
}

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct CallStack<'gc> {
    stack: Vec<CallNode<'gc>>,
}

impl<'gc> CallStack<'gc> {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, exec: Executable<'gc>) {
        self.stack.push(CallNode::Method(exec))
    }

    pub fn push_global_init(&mut self) {
        self.stack.push(CallNode::GlobalInit)
    }

    pub fn pop(&mut self) -> Option<CallNode<'gc>> {
        self.stack.pop()
    }

    pub fn display(&self, mc: MutationContext<'gc, '_>, output: &mut WString) {
        for call in self.stack.iter().rev() {
            output.push_utf8("\n\tat ");
            match call {
                CallNode::GlobalInit => output.push_utf8("global$init()"),
                CallNode::Method(exec) => output.push_str(&exec.full_name(mc)),
            }
        }
    }
}

impl<'gc> Default for CallStack<'gc> {
    fn default() -> Self {
        Self::new()
    }
}
