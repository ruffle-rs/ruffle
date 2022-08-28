//! Frame events management
//!
//! This module aids in keeping track of which frame execution phase we are in.
//!
//! For AVM2 code, display objects execute a series of discrete phases, and
//! each object is notified about the current frame phase in rendering order.
//! When objects are created, they are 'caught up' to the current frame phase
//! to ensure correct order of operations.
//!
//! AVM1 code (presumably, either on an AVM1 stage or within an `AVM1Movie`)
//! runs in one phase, with timeline operations executing with all phases
//! inline in the order that clips were originally created.

use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};

/// Which phase of the frame we're currently in.
///
/// AVM2 frames exist in one of five phases: `Enter`, `Construct`, `Update`,
/// `FrameScripts`, or `Exit`. An additional `Idle` phase covers rendering and
/// event processing.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FramePhase {
    /// We're entering the next frame.
    ///
    /// When movie clips enter a new frame, they must do two things:
    ///
    ///  - Remove all children that should not exist on the next frame.
    ///  - Increment their current frame number.
    ///
    /// Once this phase ends, we fire `enterFrame` on the broadcast list.
    Enter,

    /// We're constructing children of existing display objects.
    ///
    /// All `PlaceObject` tags should execute at this time.
    ///
    /// Once we construct the frame, we fire `frameConstructed` on the
    /// broadcast list.
    Construct,

    /// We're updating all display objects on the stage.
    ///
    /// This roughly corresponds to `run_frame`; and should encompass all time
    /// based display object changes that are not encompassed by the other
    /// phases.
    ///
    /// This frame phase also exists in AVM1 frames. In AVM1, it does the work
    /// of `Enter`, `FrameScripts` (`DoAction` tags), and `Construct`.
    Update,

    /// We're running all queued frame scripts.
    ///
    /// Frame scripts are the AS3 equivalent of old-style `DoAction` tags. They
    /// are queued in the `Update` phase if the current timeline frame number
    /// differs from the prior frame's one.
    FrameScripts,

    /// We're finishing frame processing.
    ///
    /// When we exit a completed frame, we fire `exitFrame` on the broadcast
    /// list.
    Exit,

    /// We're not currently executing any frame code.
    ///
    /// At this point in time, event handlers are expected to run. No frame
    /// catch-up work should execute.
    Idle,
}

impl Default for FramePhase {
    fn default() -> Self {
        FramePhase::Idle
    }
}

/// Run one frame according to AVM2 frame order.
pub fn run_all_phases_avm2<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
    let stage = context.stage;

    *context.frame_phase = FramePhase::Enter;
    stage.enter_frame(context);

    *context.frame_phase = FramePhase::Construct;
    stage.construct_frame(context);
    stage.frame_constructed(context);

    *context.frame_phase = FramePhase::Update;
    stage.run_frame_avm2(context);

    *context.frame_phase = FramePhase::FrameScripts;
    stage.run_frame_scripts(context);

    *context.frame_phase = FramePhase::Exit;
    stage.exit_frame(context);

    *context.frame_phase = FramePhase::Idle;
}

/// Run all previously-executed frame phases on a newly-constructed display
/// object.
///
/// This is a no-op on AVM1, which has it's own catch-up logic.
pub fn catchup_display_object_to_frame<'gc>(
    context: &mut UpdateContext<'_, 'gc, '_>,
    dobj: DisplayObject<'gc>,
) {
    if !context.is_action_script_3() {
        return;
    }

    match *context.frame_phase {
        //NOTE: We currently do not have test coverage to justify `Enter`
        //running `construct_frame`. However, `Idle` *does* need frame
        //construction to happen, because event handlers expect to be able to
        //construct new movie clips and see their grandchildren. So I suspect
        //that constructing symbols in `enterFrame` works the same way.
        FramePhase::Enter | FramePhase::Construct => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
        }
        FramePhase::Update | FramePhase::FrameScripts | FramePhase::Exit | FramePhase::Idle => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
            dobj.run_frame_avm2(context);
        }
    }
}
