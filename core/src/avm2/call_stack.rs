use crate::avm2::function::Executable;
use crate::avm2::Error;
use crate::string::WString;
use gc_arena::{Collect, MutationContext};

const MAX_CALLSTACK: usize = 500;

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
        Self {
            stack: Vec::with_capacity(MAX_CALLSTACK),
        }
    }

    pub fn push(&mut self, exec: Executable<'gc>) -> Result<(), Error> {
        self.push_node(CallNode::Method(exec))
    }

    pub fn push_global_init(&mut self) -> Result<(), Error> {
        self.push_node(CallNode::GlobalInit)
    }

    fn push_node(&mut self, node: CallNode<'gc>) -> Result<(), Error> {
        if self.stack.len() >= MAX_CALLSTACK {
            return Err("Error: Error #1023: Stack overflow occurred.".into());
        }
        self.stack.push(node);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<CallNode<'gc>> {
        self.stack.pop()
    }

    pub fn display(&self, mc: MutationContext<'gc, '_>, output: &mut WString) {
        for call in self.stack.iter().rev() {
            output.push_utf8("\n\tat ");
            match call {
                CallNode::GlobalInit => output.push_utf8("global$init()"),
                CallNode::Method(exec) => exec.write_full_name(mc, output),
            }
        }
    }
}

impl<'gc> Default for CallStack<'gc> {
    fn default() -> Self {
        Self::new()
    }
}
