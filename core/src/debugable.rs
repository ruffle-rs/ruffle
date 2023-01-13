use serde::Deserialize;
use serde::Serialize;

use crate::avm1::Activation;
use crate::avm1::ActivationIdentifier;
use crate::context::UpdateContext;
use crate::display_object::TDisplayObject;

pub trait DebugProvider<'gc> {
    /// Dispatch a debugging event to this type
    fn dispatch(&mut self, evt: TargetedMsg, context: &mut UpdateContext<'_, 'gc, '_>) -> Option<DebugMessageOut>;
}

pub enum Debuggable<'gc> {
    MovieClip(MovieClipDebugger<'gc>)
}

impl<'gc> DebugProvider<'gc> for Debuggable<'gc> {
    fn dispatch(&mut self, evt: TargetedMsg, context: &mut UpdateContext<'_, 'gc, '_>) -> Option<DebugMessageOut> {
        match self {
            Self::MovieClip(x) => x.dispatch(evt, context)
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DebugMessageIn {
    Pause,
    Play,

    /// Get a variable, path is a AVM1 style target e.g. "this.foo"
    GetVar { path: String},

    /// Send a command to the object at the given path
    Targeted { path: String, msg: TargetedMsg }
}

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
    SetPropValue { name: String, value: String},

}

#[derive(Clone,  Serialize, Deserialize, Debug)]
pub enum DebugMessageOut {
    State {
        playing: bool,
    },
    BreakpointHit { name: String },
    GetVarResult { value: String },
    DisplayObjectInfo(crate::debugable::DisplayObjectInfo),
}

#[derive(Clone,  Serialize, Deserialize, Debug)]
pub struct MovieClipInfo {
    depth: i32,
    current_frame: u16,
    is_focusable: bool,
    enabled: bool,

}

#[derive(Clone,  Serialize, Deserialize, Debug)]
pub enum DisplayObjectInfo {
    MovieClip(MovieClipInfo)
}

use crate::display_object::MovieClip;
use crate::display_object::TDisplayObjectContainer;
use crate::string::AvmString;
pub struct MovieClipDebugger<'gc> {
    tgt: MovieClip<'gc>,
}

impl<'gc> MovieClipDebugger<'gc> {
    pub fn with(tgt: MovieClip<'gc>) -> Self {
        Self {
            tgt,
        }
    }
}

impl<'gc> DebugProvider<'gc> for MovieClipDebugger<'gc> {
    fn dispatch(&mut self, evt: TargetedMsg, context: &mut UpdateContext<'_, 'gc, '_>) -> Option<DebugMessageOut> {
        match evt {
            TargetedMsg::GetInfo => {
                {
                    let mut activation = Activation::from_stub(
                            context.reborrow(),
                    ActivationIdentifier::root("[Foobar]"),
                );


                    use crate::avm1::TObject;
                    let obj = self.tgt.object().coerce_to_object(&mut activation);
                    println!("obj = {:?}", obj);
                    let keys = obj.get_keys(&mut activation);
                    println!("keys = {:?}", keys);

                }

                Some(DebugMessageOut::DisplayObjectInfo(DisplayObjectInfo::MovieClip(MovieClipInfo {
                    depth: self.tgt.depth(),
                    current_frame: self.tgt.current_frame(),
                    is_focusable: self.tgt.is_focusable(),
                    enabled: self.tgt.enabled(),
                })))
            }
            TargetedMsg::GetChildren => {
                for x in self.tgt.as_container().unwrap().iter_render_list() {
                    println!("{:?}", x.name())
                }
                None
            }
            TargetedMsg::GetProps => {
                let mut activation = Activation::from_stub(
                        context.reborrow(),
                    ActivationIdentifier::root("[Foobar]"),
                );

                use crate::avm1::TObject;
                let obj = self.tgt.object().coerce_to_object(&mut activation);
                let keys = obj.get_keys(&mut activation);
                println!("keys = {:?}", keys);

                None
            }
            TargetedMsg::GetPropValue { name } => {
                let mut activation = Activation::from_stub(
                        context.reborrow(),
                    ActivationIdentifier::root("[Foobar]"),
                );

                use crate::avm1::TObject;
                let obj = self.tgt.object().coerce_to_object(&mut activation);
                let val = obj.get(AvmString::new_utf8(activation.context.gc_context, name), &mut activation);
                println!("keys = {:?}", val);

                None
            }
            TargetedMsg::SetPropValue { name, value } => {
                let mut activation = Activation::from_stub(
                        context.reborrow(),
                    ActivationIdentifier::root("[Foobar]"),
                );

                use crate::avm1::TObject;
                let obj = self.tgt.object().coerce_to_object(&mut activation);
                obj.set(
                        AvmString::new_utf8(activation.context.gc_context, name),
                        AvmString::new_utf8(activation.context.gc_context, value).into(),
                &mut activation).unwrap();
                None
            }
            _ => None,
        }
    }
}