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
use crate::display_object::TDisplayObject;

/// Which phase of the frame we're currently in.
///
/// Each part of the frame phase is the phase we're going to execute *next*;
/// e.g. we don't go from `Enter` to `Construct`
pub enum FramePhase {
    /// We're about to enter the next frame.
    ///
    /// When we enter a new frame, movie clips increment their current frame
    /// number. Once this phase ends, we fire `enterFrame` on the broadcast
    /// list.
    ///
    /// When the player is not executing a frame advancing update, it should be
    /// in this phase.
    Enter,

    /// We're about to construct children of existing display objects.
    ///
    /// All `PlaceObject` tags should execute at this time.
    ///
    /// Once we construct the frame, we fire `frameConstructed` on the
    /// broadcast list.
    Construct,

    /// We're about to update all display objects on the stage.
    ///
    /// This roughly corresponds to `run_frame`; and should encompass all time
    /// based display object changes that are not encompassed by the other
    /// phases.
    Update,

    /// We're about to run all queued frame scripts.
    ///
    /// Frame scripts are the AS3 equivalent of old-style `DoAction` tags. They
    /// run when a movie clip enters a given timeline frame.
    FrameScripts,

    /// We're about to finish frame processing.
    ///
    /// When we exit a completed frame, we fire `exitFrame` on the broadcast
    /// list.
    Exit,

    /// We're about to destroy children of existing display objects.
    ///
    /// All `RemoveObject` tags should execute at this time.
    Destroy,
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

    *context.frame_phase = FramePhase::Enter;
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

    *context.frame_phase = FramePhase::Destroy;
    stage.destroy_frame(context);

    *context.frame_phase = FramePhase::Enter;
}
