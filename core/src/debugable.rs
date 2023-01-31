use serde::Deserialize;
use serde::Serialize;

use crate::avm1::Activation;
use crate::avm1::ActivationIdentifier;
use crate::avm1::TObject;
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;
use crate::display_object::TDisplayObject;

/// A value that can be recieved as part of a debug command
/// This is separate from the AVM* values as it cannot hold a Gc ptr and must be serializable
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DValue {
    String(String),
    Int(i32),
    Number(f64),
    Bool(bool),
    Null,
    Undefined,
    Object { kind: String },
}

use crate::avm1::Value as Avm1Value;

impl DValue {
    fn as_avm1<'gc>(&self) -> Avm1Value<'gc> {
        match self {
            // Objects can only be sent for now, not recieved
            Self::Null | Self::Object { .. } => Avm1Value::Null,
            Self::Undefined => Avm1Value::Undefined,
            Self::Int(v) => Avm1Value::Number(*v as f64),
            Self::Number(v) => Avm1Value::Number(*v),
            Self::Bool(b) => Avm1Value::Bool(*b),
            Self::String(s) => panic!(),
        }
    }
}

impl<'gc> From<Avm1Value<'gc>> for DValue {
    fn from(value: Avm1Value<'gc>) -> Self {
        match value {
            Avm1Value::Undefined => Self::Undefined,
            Avm1Value::Null => Self::Null,
            Avm1Value::Bool(b) => Self::Bool(b),
            Avm1Value::Number(n) => {
                if let Ok(i) = i32::from_str_radix(&n.to_string(), 10) {
                    Self::Int(i)
                } else {
                    Self::Number(n)
                }
            },
            //TODO: send wstrs
            Avm1Value::String(s) => Self::String(s.to_utf8_lossy().to_string()),

            Avm1Value::Object(o) => Self::Object { kind: format!("{:?}", o) },
        }
    }
}

pub trait DebugProvider<'gc> {
    /// Dispatch a debugging event to this type
    fn dispatch(
        &mut self,
        evt: TargetedMsg,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> Option<DebugMessageOut>;
}

pub enum Debuggable<'gc> {
    MovieClip(MovieClipDebugger<'gc>),
}

impl<'gc> DebugProvider<'gc> for Debuggable<'gc> {
    fn dispatch(
        &mut self,
        evt: TargetedMsg,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> Option<DebugMessageOut> {
        match self {
            Self::MovieClip(x) => x.dispatch(evt, context),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DebugMessageIn {
    /// Send a command to the object at the given path
    Targeted {
        path: String,
        msg: TargetedMsg,
    },

    /// Send a command to the player
    Player {
        msg: PlayerMsg,
    },


    /// Send a command to AVM1
    Avm1 {
        msg: Avm1Msg,
    }
}

/// Debug messages that are handled by the AVM1 VM
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Avm1Msg {
    /// Execute until the end of the current scope
    StepOut,

    /// Execute a single avm1 opcode, following calls
    StepInto,

    /// Execute a single avm1 opcode, without following calls
    StepOver,
    
    /// Continue exeuction until the current activation returns
    SteoOut,

    /// Break execution at the current position
    Break,

    /// Get the current state of the AVM1 stack
    GetStack,

    /// Resume execution
    Continue,

    /// Get the current state of the constant pool
    GetConstantPool,

    /// Break on calling the given function
    BreakFunction { name: String },

    /// Remove the function breakpoint with the given name
    BreakFunctionDelete { name: String },

    /// Get all the breakpoints
    GetBreakpoints,

    /// Push a value onto the stack
    Push { val: DValue },

    /// Pop the top value off of the stack
    Pop,

    /// Get the value at the given path
    GetVariable { path: String },

    /// Set the value at the given path
    SetVariable { path: String, value: DValue },

    /// Get the sub-properties of the value the given path
    GetSubprops { path: String },
}

/// Debug messages that are handled in the player
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PlayerMsg {
    /// Pause the player
    Pause,

    /// Resume the player
    Play,
}

/// Debug messages that are handled by a specificly targeted display object
///TODo rename?
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TargetedMsg {
    /// Get information about this object display object
    GetInfo,

    /// Get children of this display object
    GetChildren,

    /// Get properties on this object
    GetProps,

    /// Get the value of the given prop
    GetPropValue { name: String },

    /// Set the value of the given prop
    SetPropValue { name: String, value: String },

    /// Stop this clip
    /// TODO: this only works on clips, should we have a custom(str) msg that allows do-specific behaviour, or should they all be in this enum with a msg that allows getting which ones are available
    Stop,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DebugMessageOut {
    /// A generic success / fail message
    GenericResult { success: bool },

    /// Result sent when a value is retrieved
    GetValueResult { path: String, value: DValue },

    /// Result sent when requesing sub-properties of an object
    GetSubpropsResult { path: String, props: Vec<String> },

    State { playing: bool },
    BreakpointHit { name: String },
    GetVarResult { value: String },
    DisplayObjectInfo(crate::debugable::DisplayObjectInfo),
    GetPropsResult { keys: Vec<String> },
    BreakpointList { bps: Vec<String> },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MovieClipInfo {
    depth: i32,
    current_frame: u16,
    is_focusable: bool,
    enabled: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DisplayObjectInfo {
    MovieClip(MovieClipInfo),
}

use crate::display_object::MovieClip;
use crate::display_object::TDisplayObjectContainer;
use crate::prelude::Depth;
use crate::string::AvmString;
pub struct MovieClipDebugger<'gc> {
    tgt: MovieClip<'gc>,
}

impl<'gc> MovieClipDebugger<'gc> {
    pub fn with(tgt: MovieClip<'gc>) -> Self {
        Self { tgt }
    }
}

impl<'gc> DebugProvider<'gc> for MovieClipDebugger<'gc> {
    fn dispatch(
        &mut self,
        evt: TargetedMsg,
        context: &mut UpdateContext<'_, 'gc>,
    ) -> Option<DebugMessageOut> {
        match evt {
            TargetedMsg::Stop => {
                self.tgt.stop(context);
                Some(DebugMessageOut::GenericResult { success: true })
            }
            TargetedMsg::GetInfo => {
                {
                    let mut activation = Activation::from_stub(
                        context.reborrow(),
                        ActivationIdentifier::root("[Foobar]"),
                    );

                    let obj = self.tgt.object().coerce_to_object(&mut activation);
                    println!("obj = {:?}", obj);
                    let keys = obj.get_keys(&mut activation);
                    println!("keys = {:?}", keys);
                }

                Some(DebugMessageOut::DisplayObjectInfo(
                    DisplayObjectInfo::MovieClip(MovieClipInfo {
                        depth: self.tgt.depth(),
                        current_frame: self.tgt.current_frame(),
                        is_focusable: self.tgt.is_focusable(),
                        enabled: self.tgt.enabled(),
                    }),
                ))
            }
            TargetedMsg::GetChildren => {
                for x in self.tgt.as_container().unwrap().iter_render_list() {
                    println!("{:?}", x.name())
                }
                Some(DebugMessageOut::GenericResult { success: true })
            }
            TargetedMsg::GetProps => {
                let mut activation = Activation::from_stub(
                    context.reborrow(),
                    ActivationIdentifier::root("[Foobar]"),
                );

                let obj = self.tgt.object().coerce_to_object(&mut activation);
                let keys = obj.get_keys(&mut activation);
                println!("keys = {:?}", keys);

                let mut out_keys: Vec<String> = Vec::new();
                for key in &keys {
                    out_keys.push(key.to_utf8_lossy().to_string());
                }

                Some(DebugMessageOut::GetPropsResult { keys: out_keys })
            }
            TargetedMsg::GetPropValue { name } => {
                let mut activation = Activation::from_stub(
                    context.reborrow(),
                    ActivationIdentifier::root("[Foobar]"),
                );

                let obj = self.tgt.object().coerce_to_object(&mut activation);
                let val = obj.get(
                    AvmString::new_utf8(activation.context.gc_context, name),
                    &mut activation,
                );
                println!("keys = {:?}", val);

                Some(DebugMessageOut::GenericResult { success: true })
            }
            TargetedMsg::SetPropValue { name, value } => {
                let mut activation = Activation::from_stub(
                    context.reborrow(),
                    ActivationIdentifier::root("[Foobar]"),
                );

                let obj = self.tgt.object().coerce_to_object(&mut activation);
                obj.set(
                    AvmString::new_utf8(activation.context.gc_context, name),
                    AvmString::new_utf8(activation.context.gc_context, value).into(),
                    &mut activation,
                )
                .unwrap();
                Some(DebugMessageOut::GenericResult { success: true })
            }
        }
    }
}

/// Process pending player debug events
pub fn handle_player_debug_events<'gc>(context: &mut UpdateContext<'_, 'gc>) {
    while let Some(dbg_in) = context.debugger.get_debug_event_player() {
        match dbg_in {
            PlayerMsg::Pause => {
                context
                    .player
                    .upgrade()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .set_is_playing(false);
                let msg = DebugMessageOut::State { playing: true };
                context.debugger.submit_debug_message(msg);
            }
            PlayerMsg::Play => {
                println!("Handling play");
                context
                    .player
                    .upgrade()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .set_is_playing(false);
                let msg = DebugMessageOut::State { playing: false };
                context.debugger.submit_debug_message(msg);
            }
        }
    }
}

/// Walk a depth-path, returning the dispaly object at that point in the depth-tree, if it exists
fn walk_depthpath<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    path: &[Depth],
) -> Option<DisplayObject<'gc>> {
    let mut root = context.stage.root_clip();

    // Walk the path
    for depth in path.iter().copied() {
        // If we have a container
        //TODO: this wont work with buttons for now
        if let Some(cont) = root.as_container() {
            // Get the child at that depth
            if let Some(depth_child) = cont.child_by_depth(depth) {
                root = depth_child;
            } else {
                println!("no child");
                // No child at that depth, exit
                return None;
            }
        } else {
            print!("Not cont");
            // Not a container, can't get a depth-child
            return None;
        }
    }

    Some(root)
}

/// Walk a depth-path, returning the dispaly object at that point in the depth-tree, if it exists
fn walk_path<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    path: &[&str],
) -> Option<DisplayObject<'gc>> {
    let mut root = context.stage.root_clip();

    // Walk the path
    for depth in path.iter() {
        // If we have a container
        //TODO: this wont work with buttons for now
        if let Some(cont) = root.as_container() {
            // Get the child at that depth
            if let Some(child) =
                cont.child_by_name(ruffle_wstr::WStr::from_units(depth.as_bytes()), true)
            {
                root = child;
            } else {
                println!("no child");
                // No child at that depth, exit
                return None;
            }
        } else {
            print!("Not cont");
            // Not a container, can't get a depth-child
            return None;
        }
    }

    Some(root)
}

pub fn handle_targeted_debug_events<'gc>(context: &mut UpdateContext<'_, 'gc>) {
    while let Some((path, msg)) = context.debugger.get_debug_event_targeted() {
        let d_o = if path == "/" {
            context.stage.root_clip()
        } else {
            //let dp = path.split("/").map(|x| Depth::from_str(x).unwrap()).collect::<Vec<_>>();
            //let d_o = walk_depthpath(context, &dp);
            let dp = path.split("/").collect::<Vec<_>>();
            println!("path = {:?}", dp);
            let d_o = walk_path(context, &dp);
            d_o.unwrap()
        };

        let evt = d_o.as_debuggable().unwrap().dispatch(msg, context);
        if let Some(evt) = evt {
            context.debugger.submit_debug_message(evt);
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Avm1ExecutionState {
    Running,
    Paused,
    StepInto,
    StepOut,
}
use swf::avm1::types::Action;

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

            let msg = DebugMessageOut::BreakpointHit { name: name.clone() };
            context.debugger.submit_debug_message(msg);
        }
    }
}

//TODO: add this to activation
pub static mut AVM1_DBG_STATE: Avm1Debugger = Avm1Debugger::new();

pub fn handle_avm1_debug_events<'gc>(activation: &mut Activation<'_, 'gc>) {
    let dbg = unsafe { &mut AVM1_DBG_STATE };

    while let Some(msg) = activation.context.debugger.get_debug_event_avm1() {
        match msg {
            Avm1Msg::StepInto => {
                dbg.execution_state = Avm1ExecutionState::StepInto;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::StepOut => {
                dbg.execution_state = Avm1ExecutionState::StepOut;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Break => {
                dbg.execution_state = Avm1ExecutionState::Paused;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::GetStack => {
                //TODO: resp for this
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::BreakFunction { name } => {
                dbg.pending_breakpoints.push(name);
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::BreakFunctionDelete { name } => {
                if let Some(pos) = dbg.pending_breakpoints.iter().position(|p| p == &name) {
                    dbg.pending_breakpoints.remove(pos);
                }
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Continue => {
                dbg.execution_state = Avm1ExecutionState::Running;
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Push { val } => {
                activation.context.avm1.push(val.as_avm1());
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::Pop => {
                activation.context.avm1.pop();
                let msg = DebugMessageOut::GenericResult { success: true };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::GetBreakpoints => {
                let msg = DebugMessageOut::BreakpointList { bps: dbg.pending_breakpoints.clone() };
                activation.context.debugger.submit_debug_message(msg);
            }
            Avm1Msg::GetVariable { path } => {
                let res = activation.get_variable(AvmString::new_utf8(activation.context.gc_context, path.clone()));
                if let Ok(res) = res {
                    let val: Avm1Value<'gc> = res.into();
                    println!("val = {:?}", val);

                    activation.context.debugger.submit_debug_message(DebugMessageOut::GetValueResult { path, value: val.into() });
                } else {
                    activation.context.debugger.submit_debug_message(DebugMessageOut::GenericResult { success: false });
                }
            }
            Avm1Msg::SetVariable { path, value } => {
                let res = activation.set_variable(AvmString::new_utf8(activation.context.gc_context, path.clone()), value.as_avm1());
                activation.context.debugger.submit_debug_message(DebugMessageOut::GenericResult { success: res.is_ok() });
            }
            Avm1Msg::GetSubprops { path } => {
                let res = activation.get_variable(AvmString::new_utf8(activation.context.gc_context, path.clone()));
                if let Ok(res) = res {
                    let res: Avm1Value<'gc> = res.into();

                    if let Avm1Value::Object(o) = res {
                        let mut props = Vec::new();

                        for child in o.get_keys(activation) {
                            props.push(child.to_utf8_lossy().to_string());
                        }

                        activation.context.debugger.submit_debug_message(DebugMessageOut::GetSubpropsResult { path, props });
                    } else {
                        activation.context.debugger.submit_debug_message(DebugMessageOut::GetSubpropsResult { path, props: vec![] });
                    }
                } else {
                    activation.context.debugger.submit_debug_message(DebugMessageOut::GenericResult { success: false });
                }
            }
            _ => {},
        }
    }
}
