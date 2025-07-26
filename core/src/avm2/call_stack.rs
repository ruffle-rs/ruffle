use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::function::display_function;
use crate::avm2::method::Method;
use crate::avm2::AvmString;
use crate::avm2::Namespace;
use crate::string::WString;
use gc_arena::Collect;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct CallNode<'gc> {
    method: Method<'gc>,
    class: Option<Class<'gc>>,
    /// The default XML namespace for this method frame, if set.
    default_xml_namespace: Option<Namespace<'gc>>,
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

    pub fn push(&mut self, method: Method<'gc>, class: Option<Class<'gc>>) {
        self.stack.push(CallNode {
            method,
            class,
            default_xml_namespace: None,
        });
    }

    pub fn pop(&mut self) -> Option<CallNode<'gc>> {
        self.stack.pop()
    }

    pub fn display(&self, output: &mut WString) {
        for call in self.stack.iter().rev() {
            output.push_utf8("\n\tat ");

            let is_global_init = call.class.is_some_and(|c| {
                // If the class is a script `global` class and its instance
                // initializer is this method, then this is a script initializer
                c.is_script_traits() && c.instance_init() == Some(call.method)
            });

            // Special-case the printed message for script initializers
            if is_global_init {
                let tunit = call.method.translation_unit();
                let name = if let Some(name) = tunit.name() {
                    name.to_utf8_lossy().to_string()
                } else {
                    "<No name>".to_string()
                };

                // NOTE: We intentionally diverge from Flash Player's output
                // here - everything with the [] brackets is extra information
                // added by Ruffle
                output.push_utf8(&format!("global$init() [TU={name}]"));
            } else {
                display_function(output, call.method, call.class);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Set the default XML namespace for the current method frame.
    /// This is called by the DxnsLate opcode.
    pub fn set_default_xml_namespace(&mut self, namespace: Namespace<'gc>) {
        if let Some(call_node) = self.stack.last_mut() {
            call_node.default_xml_namespace = Some(namespace);
        }
    }

    /// Get the default XML namespace by walking up the call stack.
    /// This matches avmplus's MethodFrame::findDxns behavior.
    pub fn find_default_xml_namespace(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Namespace<'gc>> {
        // Walk backwards through the call stack (most recent first)
        for call_node in self.stack.iter().rev() {
            // If this frame has an explicit DXNS declaration, return it
            if let Some(ns) = call_node.default_xml_namespace {
                return Some(ns);
            }

            // If this method has the SET_DXNS flag, it means it has a local
            // default xml namespace declaration, so the default should be the
            // unnamed namespace (empty string) rather than inheriting from global scope
            if call_node.method.sets_dxns() {
                return Some(Namespace::package(
                    AvmString::new_utf8(activation.gc(), ""),
                    activation.avm2().root_api_version,
                    activation.strings(),
                ));
            }
        }
        None
    }
}

impl Default for CallStack<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CallStack<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = WString::new();
        self.display(&mut output);
        write!(f, "{output}")
    }
}
