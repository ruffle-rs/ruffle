use crate::avm2::function::display_function;
use crate::avm2::method::Method;
use crate::string::WString;
use gc_arena::Collect;

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct CallStack<'gc> {
    stack: Vec<Method<'gc>>,
}

impl<'gc> CallStack<'gc> {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push(&mut self, method: Method<'gc>) {
        self.stack.push(method)
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn display(&self, output: &mut WString) {
        for method in self.stack.iter().rev() {
            output.push_utf8("\n\tat ");

            let bound_class = method.bound_class();

            let is_global_init = bound_class.is_some_and(|c| {
                // If the class is a script `global` class and its instance
                // initializer is this method, then this is a script initializer
                c.is_script_traits() && c.instance_init() == Some(*method)
            });

            // Special-case the printed message for script initializers
            if is_global_init {
                let tunit = method.translation_unit();
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
                display_function(output, *method);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
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
