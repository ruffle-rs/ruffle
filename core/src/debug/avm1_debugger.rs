use crate::avm1::Activation;
use crate::avm1::TObject;
use crate::avm1::Value as Avm1Value;
use crate::context::UpdateContext;
use crate::debug::avm1_message::Avm1Msg;
use crate::debug::debug_message_out::DebugMessageOut;
use crate::debug::debug_value::DValue;
use crate::string::AvmString;
use swf::avm1::types::Action;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum Avm1ExecutionState {
    #[default]
    Running,
    Paused,
    StepInto,
    StepOut,
}

#[derive(Default, Clone)]
pub struct Avm1Debugger {
    /// What is the current execution state
    execution_state: Avm1ExecutionState,

    /// The current list of pending breakpoints
    ///
    /// When a function that is in this list is called, `execution_state` will move to `Paused`
    pending_breakpoints: Vec<String>,
}

impl Avm1Debugger {
    pub const fn new() -> Self {
        Self {
            execution_state: Avm1ExecutionState::Running,
            pending_breakpoints: Vec::new(),
        }
    }

    /// Should the vm be paused
    pub fn pause_execution(&self) -> bool {
        self.execution_state == Avm1ExecutionState::Paused
    }

    /// Update the current debugger state based on the action to be executed
    pub fn preprocess_action(&mut self, act: Action) {
        if self.execution_state == Avm1ExecutionState::StepInto {
            println!("Executed {:?}", act);
            self.execution_state = Avm1ExecutionState::Paused;
        } else if self.execution_state == Avm1ExecutionState::StepOut && act == Action::Return {
            self.execution_state = Avm1ExecutionState::Paused;
        }
        //TODO: send msg for this
    }

    /// Preprocess a given function call to update debugger state
    pub fn preprocess_call<'gc>(&mut self, context: &mut UpdateContext<'_, 'gc>, name: String) {
        //println!("call = {}, bps = {:?}", name, self.pending_breakpoints);
        if self.pending_breakpoints.contains(&name) || name == "_debugbreak" {
            self.execution_state = Avm1ExecutionState::Paused;

            let msg = DebugMessageOut::BreakpointHit { name };
            context.debugger.submit_debug_message(msg);
        }
    }
}

pub fn handle_avm1_debug_events<'gc>(activation: &mut Activation<'_, 'gc>) {
    while let Some(msg) = activation.context.debugger.get_debug_event_avm1() {
        match msg {
            Avm1Msg::StepInto => {
                activation.debug_state_mut().execution_state = Avm1ExecutionState::StepInto;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::StepOut => {
                activation.debug_state_mut().execution_state = Avm1ExecutionState::StepOut;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Break => {
                activation.debug_state_mut().execution_state = Avm1ExecutionState::Paused;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::GetStack => {
                //TODO: resp for this
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::BreakFunction { name } => {
                activation.debug_state_mut().pending_breakpoints.push(name);
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::BreakFunctionDelete { name } => {
                if let Some(pos) = activation
                    .debug_state_mut()
                    .pending_breakpoints
                    .iter()
                    .position(|p| p == &name)
                {
                    activation.debug_state_mut().pending_breakpoints.remove(pos);
                }
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Continue => {
                activation.debug_state_mut().execution_state = Avm1ExecutionState::Running;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Push { val } => {
                let val = val.as_avm1(&mut activation.context);
                activation.context.avm1.push(val);
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Pop => {
                activation.context.avm1.pop();
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::GetBreakpoints => {
                let msg = DebugMessageOut::BreakpointList {
                    bps: activation.debug_state_mut().pending_breakpoints.clone(),
                };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::GetVariable { path } => {
                let res = activation.get_variable(AvmString::new_utf8(
                    activation.context.gc_context,
                    path.clone(),
                ));
                if let Ok(res) = res {
                    let val: Avm1Value<'gc> = res.into();
                    println!("val = {:?}", val);

                    activation.context.debugger.submit_debug_message(
                        DebugMessageOut::GetValueResult {
                            path,
                            value: val.into(),
                        },
                    );
                } else {
                    activation
                        .context
                        .debugger
                        .submit_debug_message(DebugMessageOut::GenericResult { success: false });
                }
            }
            Avm1Msg::SetVariable { path, value } => {
                let value = value.as_avm1(&mut activation.context);
                let res = activation.set_variable(
                    AvmString::new_utf8(activation.context.gc_context, path.clone()),
                    value,
                );
                activation
                    .context
                    .debugger
                    .submit_debug_message(DebugMessageOut::GenericResult {
                        success: res.is_ok(),
                    });
            }
            Avm1Msg::GetSubprops { path } => {
                let res = activation.get_variable(AvmString::new_utf8(
                    activation.context.gc_context,
                    path.clone(),
                ));
                if let Ok(res) = res {
                    let res: Avm1Value<'gc> = res.into();

                    if let Avm1Value::Object(o) = res {
                        let mut props = Vec::new();

                        for child in o.get_keys(activation) {
                            props.push(child.to_utf8_lossy().to_string());
                        }

                        activation.context.debugger.submit_debug_message(
                            DebugMessageOut::GetSubpropsResult { path, props },
                        );
                    } else {
                        activation.context.debugger.submit_debug_message(
                            DebugMessageOut::GetSubpropsResult {
                                path,
                                props: vec![],
                            },
                        );
                    }
                } else {
                    activation
                        .context
                        .debugger
                        .submit_debug_message(DebugMessageOut::GenericResult { success: false });
                }
            }
            Avm1Msg::GetBacktrace => {
                let mut bt = Vec::new();

                let mut id = Some(&activation.id);
                while let Some(p) = id {
                    bt.push(p.name().to_string());
                    id = p.parent();
                }
                activation
                    .context
                    .debugger
                    .submit_debug_message(DebugMessageOut::GetBacktraceResult { backtrace: bt });
            }
            Avm1Msg::GetRegisters => {
                let mut regs = Vec::<DValue>::new();

                if let Some(r) = &activation.local_registers() {
                    let r = r.read();
                    for i in 0..r.len() {
                        regs.push(DValue::from(*r.get(i).unwrap()));
                    }
                }

                activation
                    .context
                    .debugger
                    .submit_debug_message(DebugMessageOut::GetRegisterResult { regs });
            }
            Avm1Msg::StepOver => {}
            Avm1Msg::GetConstantPool => {}
            Avm1Msg::GetLocals => {
                let scope = activation.scope();
                let locals = scope.locals();
                let mut props = Vec::new();

                for child in locals.get_keys(activation) {
                    props.push(child.to_utf8_lossy().to_string());
                }
                activation
                    .context
                    .debugger
                    .submit_debug_message(DebugMessageOut::GetLocalsResult { locals: props });
            }
            Avm1Msg::GetGlobals => {
                let mut root_scope = activation.scope();
                while let Some(parent_scope) = root_scope.parent() {
                    root_scope = parent_scope;
                }
                let root_object = root_scope.locals();

                let mut props = Vec::new();

                for child in root_object.get_keys(activation) {
                    props.push(child.to_utf8_lossy().to_string());
                }
                activation
                    .context
                    .debugger
                    .submit_debug_message(DebugMessageOut::GetGlobalsResult { globals: props });
            }
        }
    }
}
