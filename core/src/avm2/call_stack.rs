use crate::avm2::class::Class;
use crate::avm2::function::Executable;
use crate::string::{AvmString, WString};
use gc_arena::{Collect, Gc, MutationContext};
use std::cell::Ref;

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
            let (prefix, name) = match call {
                CallNode::GlobalInit => (None, "global$init".into()),
                CallNode::Method(exec) => {
                    if let Some(superclass) = exec.bound_superclass() {
                        let class_def = superclass.inner_class_definition();
                        let name = class_def.read().name().to_qualified_name(mc);
                        output.push_str(&name);
                        resolve_executable_name(mc, Some(class_def.read()), exec)
                    } else {
                        resolve_executable_name(mc, None, exec)
                    }
                }
            };

            if let Some(prefix) = prefix {
                output.push_utf8(prefix)
            }
            output.push_str(&name);
            output.push_utf8("()");
        }
    }
}

/// Resolves an executable to its name, possibly using its bound superclass.
fn resolve_executable_name<'gc>(
    mc: MutationContext<'gc, '_>,
    class_def: Option<Ref<Class<'gc>>>,
    exec: &Executable<'gc>,
) -> (Option<&'static str>, AvmString<'gc>) {
    match exec {
        Executable::Native(nm) => (Some("/"), nm.method().name.into()),
        Executable::Action(bm) => {
            if let Some(class_def) = class_def {
                // First check if this is an instance initializer or class initializer
                // We know that this is an ABC class, therefore all methods must be bytecode.
                if Gc::ptr_eq(class_def.class_init().into_bytecode().unwrap(), bm.method()) {
                    return (Some("$"), "cinit".into());
                } else if Gc::ptr_eq(
                    class_def.instance_init().into_bytecode().unwrap(),
                    bm.method(),
                ) {
                    return (None, AvmString::default());
                }
                // Now that we know it isn't either of those, search class traits & instance traits for the method
                for t in class_def.class_traits() {
                    if let Some(m) = t.as_method() {
                        let bytecode = m.into_bytecode().unwrap();
                        if Gc::ptr_eq(bytecode, bm.method()) {
                            return (Some("$/"), t.name().local_name());
                        }
                    }
                }

                for t in class_def.instance_traits() {
                    if let Some(m) = t.as_method() {
                        let bytecode = m.into_bytecode().unwrap();
                        if Gc::ptr_eq(bytecode, bm.method()) {
                            return (Some("/"), t.name().local_name());
                        }
                    }
                }
            }
            // If we were unable to resolve this method to any name, use MethodInfo-{abc_index} by default.
            (
                None,
                AvmString::new_utf8(mc, format!("MethodInfo-{}", bm.method().abc_method)),
            )
        }
    }
}

impl<'gc> Default for CallStack<'gc> {
    fn default() -> Self {
        Self::new()
    }
}
