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
use crate::vminterface::AvmType;

/// Which phase of the frame we're currently in.
///
/// AVM2 frames exist in one of six phases: `Destroy`, `Enter`, `Construct`,
/// `Update`, `FrameScripts`, or `Exit`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FramePhase {
    /// We're destroying children of existing display objects.
    ///
    /// All `RemoveObject` tags should execute at this time.
    ///
    /// NOTE: Strictly speaking, this should occur at the end of the prior
    /// frame after rendering. However, our current frame architecture does not
    /// allow us to create a separate phase for rendering. Hence, we run the
    /// prior frame's `Destroy` phase on the next frame. In practice, the only
    /// code that might be able to see this would be code that runs in the
    /// `Idle` phase.
    Destroy,

    /// We're entering the next frame.
    ///
    /// When we enter a new frame, movie clips increment their current frame
    /// number. Once this phase ends, we fire `enterFrame` on the broadcast
    /// list.
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
    /// of `Destroy`, `Enter`, `FrameScripts` (`DoAction` tags), and
    /// `Construct`.
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
    /// This frame phase exists in both AVM1 and AVM2. It encompasses all
    /// non-update processing, such as handling and dispatching events or
    /// rendering the stage. It is also the default frame phase.
    Idle,
}

impl Default for FramePhase {
    fn default() -> Self {
        FramePhase::Idle
    }
}

/// Run one frame according to AVM1 frame order.
pub fn run_all_phases_avm1<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
    // In AVM1, we only ever execute the update phase, and all the work that
    // would ordinarily be phased is instead run all at once in whatever order
    // the SWF requests it.
    *context.frame_phase = FramePhase::Update;

    // AVM1 execution order is determined by the global execution list, based on instantiation order.
    for clip in context.avm1.clip_exec_iter() {
        if clip.removed() {
            // Clean up removed objects from this frame or a previous frame.
            // Can be safely removed while iterating here, because the iterator advances
            // to the next node before returning the current node.
            context.avm1.remove_from_exec_list(context.gc_context, clip);
        } else {
            clip.run_frame(context);
        }
    }

    // Fire "onLoadInit" events.
    context
        .load_manager
        .movie_clip_on_load(context.action_queue);

    *context.frame_phase = FramePhase::Idle;
}

/// Run one frame according to AVM2 frame order.
pub fn run_all_phases_avm2<'gc>(context: &mut UpdateContext<'_, 'gc, '_>) {
    let stage = context.stage;

    //As mentioned in the doc comment for `Destroy`, because frame rendering
    //happens during `Idle`, we have to wait until the next frame to remove the
    //prior frame's display objects. Otherwise, if we `Destroy` later on in the
    //frame, then we'll accidentally run the next frame's `RemoveObject` tags
    //too early and timeline animations will visibly flicker.
    *context.frame_phase = FramePhase::Destroy;
    stage.destroy_frame(context);

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
    match (*context.frame_phase, context.avm_type()) {
        (_, AvmType::Avm1) => {}
        //NOTE: We currently do not have test coverage to justify `Enter`
        //running `construct_frame`. However, `Idle` *does* need frame
        //construction to happen, because event handlers expect to be able to
        //construct new movie clips and see their grandchildren. So I suspect
        //that constructing symbols in `enterFrame` works the same way.
        (FramePhase::Enter, AvmType::Avm2) | (FramePhase::Construct, AvmType::Avm2) => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
        }
        (FramePhase::Update, AvmType::Avm2) => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
            dobj.run_frame_avm2(context);
        }
        (FramePhase::FrameScripts, AvmType::Avm2) => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
            dobj.run_frame_avm2(context);
            dobj.run_frame_scripts(context);
        }
        (FramePhase::Exit, AvmType::Avm2) | (FramePhase::Idle, AvmType::Avm2) => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
            dobj.run_frame_avm2(context);
            dobj.run_frame_scripts(context);
            dobj.exit_frame(context);
        }
        (FramePhase::Destroy, AvmType::Avm2) => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
            dobj.run_frame_avm2(context);
            dobj.run_frame_scripts(context);
            dobj.exit_frame(context);
            dobj.destroy_frame(context);
        }
    }
}
