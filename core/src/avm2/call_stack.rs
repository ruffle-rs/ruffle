use crate::avm2::function::Executable;
use crate::string::WString;
use gc_arena::Collect;

use super::script::Script;

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub enum CallNode<'gc> {
    GlobalInit(Script<'gc>),
    Method(Executable<'gc>),
}

#[derive(Collect, Clone)]
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

    pub fn push_global_init(&mut self, script: Script<'gc>) {
        self.stack.push(CallNode::GlobalInit(script))
    }

    pub fn pop(&mut self) -> Option<CallNode<'gc>> {
        self.stack.pop()
    }

    pub fn display(&self, output: &mut WString) {
        for call in self.stack.iter().rev() {
            output.push_utf8("\n\tat ");
            match call {
                CallNode::GlobalInit(script) => {
                    let name = if let Some(tuint) = script.translation_unit() {
                        if let Some(name) = tuint.name() {
                            name.to_utf8_lossy().to_string()
                        } else {
                            "<No name>".to_string()
                        }
                    } else {
                        "<No translation unit>".to_string()
                    };

                    // NOTE: We intentionally diverge from Flash Player's output
                    // here - everything with the [] brackets is extra information
                    // added by Ruffle
                    output.push_utf8(&format!("global$init() [TU={}]", name));
                }
                CallNode::Method(exec) => exec.write_full_name(output),
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

impl<'gc> Default for CallStack<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc> std::fmt::Display for CallStack<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = WString::new();
        self.display(&mut output);
        write!(f, "{output}")
    }
}
