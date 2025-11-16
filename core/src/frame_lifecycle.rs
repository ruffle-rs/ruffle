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

use crate::avm2::{Avm2, EventObject};
use crate::avm2_stub_method_context;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, MovieClip, TDisplayObject};
use crate::loader::LoadManager;
use crate::orphan_manager::OrphanManager;
use tracing::instrument;

/// Which phase of the frame we're currently in.
///
/// AVM2 frames exist in one of four phases: `Enter`, `Construct`,
/// `FrameScripts`, or `Exit`. An additional `Idle` phase covers rendering and
/// event processing.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
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
    #[default]
    Idle,
}

/// Run one frame according to AVM2 frame order.
/// NOTE: The `each_orphan_movie` calls are in really odd places,
/// but this is needed to match Flash Player's output. There may
/// still be lurking bugs, but the current code matches Flash's
/// output exactly for two complex test cases (see `avm2/orphan_movie*`)
#[instrument(level = "debug", skip_all)]
pub fn run_all_phases_avm2(context: &mut UpdateContext<'_>) {
    let stage = context.stage;

    if !stage.movie().is_action_script_3() {
        return;
    }

    *context.frame_phase = FramePhase::Enter;
    OrphanManager::each_orphan_obj(context, |orphan, context| {
        orphan.enter_frame(context);
    });
    stage.enter_frame(context);

    *context.frame_phase = FramePhase::Construct;
    OrphanManager::each_orphan_obj(context, |orphan, context| {
        orphan.construct_frame(context);
    });
    stage.construct_frame(context);
    broadcast_frame_constructed(context);

    *context.frame_phase = FramePhase::FrameScripts;
    OrphanManager::each_orphan_obj(context, |orphan, context| {
        orphan.run_frame_scripts(context);
    });
    stage.run_frame_scripts(context);
    MovieClip::run_frame_script_cleanup(context);

    *context.frame_phase = FramePhase::Exit;
    broadcast_frame_exited(context);

    // We cannot easily remove dead `GcWeak` instances from the orphan list
    // inside `each_orphan_movie`, since the callback may modify the orphan list.
    // Instead, we do one cleanup at the end of the frame.
    // This performs special handling of clips which became orphaned as
    // a result of a RemoveObject tag - see `cleanup_dead_orphans` for details.
    context.orphan_manager.cleanup_dead_orphans(context.gc());

    *context.frame_phase = FramePhase::Idle;
}

/// Like `run_all_phases_avm2`, but specialized for the "nested frame" triggered
/// by a goto. This is different enough to not be worth combining into a single
/// method with extra parameters.
///
/// During a goto, we run frame construction, framescripts, and frame exits for the *entire stage*.
/// This even extends to orphans - for example, calling `gotoAndStop` on an orphan will
/// cause frame construction to get run for the *current frame* of other objects on the timeline
/// (even if the goto was called from an enterFrame event handler).
pub fn run_inner_goto_frame<'gc>(
    context: &mut UpdateContext<'gc>,
    removed_frame_scripts: &[DisplayObject<'gc>],
    initial_clip: MovieClip<'gc>,
) {
    if initial_clip.swf_version() <= 9 && initial_clip.movie().is_action_script_3() {
        avm2_stub_method_context!(
            context,
            "flash.display.MovieClip",
            "goto",
            "with SWF 9 movie"
        );

        // We skip the next `enter_frame` call, so that we will still run the framescripts
        // queued for our target frame.
        initial_clip.base().set_skip_next_enter_frame(true);

        return;
    }

    let stage = context.stage;
    let old_phase = *context.frame_phase;

    // When performing goto, frame scripts behave the same as when entering a new frame
    // so no separate cleanup is performed on ones registered during frame script phase
    context.frame_script_cleanup_queue.clear();

    // Note - we do *not* call `enter_frame` or dispatch an `enterFrame` event

    *context.frame_phase = FramePhase::Construct;
    OrphanManager::each_orphan_obj(context, |orphan, context| {
        orphan.construct_frame(context);
    });
    stage.construct_frame(context);
    broadcast_frame_constructed(context);

    *context.frame_phase = FramePhase::FrameScripts;
    stage.run_frame_scripts(context);
    OrphanManager::each_orphan_obj(context, |orphan, context| {
        orphan.run_frame_scripts(context);
    });

    for child in removed_frame_scripts {
        child.run_frame_scripts(context);
    }

    *context.frame_phase = FramePhase::Exit;
    broadcast_frame_exited(context);

    // We cannot easily remove dead `GcWeak` instances from the orphan list
    // inside `each_orphan_movie`, since the callback may modify the orphan list.
    // Instead, we do one cleanup at the end of the frame.
    // This performs special handling of clips which became orphaned as
    // a result of a RemoveObject tag - see `cleanup_dead_orphans` for details.
    context.orphan_manager.cleanup_dead_orphans(context.gc());

    *context.frame_phase = old_phase;
}

/// Broadcast a `frameConstructed` event to all `DisplayObject`s.
pub fn broadcast_frame_constructed<'gc>(context: &mut UpdateContext<'gc>) {
    let frame_constructed_evt = EventObject::bare_default_event(context, "frameConstructed");
    let dobject_constr = context.avm2.classes().display_object;
    Avm2::broadcast_event(context, frame_constructed_evt, dobject_constr);
}

/// Broadcast a `exitFrame` event to all `DisplayObject`s.
pub fn broadcast_frame_exited<'gc>(context: &mut UpdateContext<'gc>) {
    let exit_frame_evt = EventObject::bare_default_event(context, "exitFrame");
    let dobject_constr = context.avm2.classes().display_object;
    Avm2::broadcast_event(context, exit_frame_evt, dobject_constr);

    LoadManager::run_exit_frame(context);
}

/// Run all previously-executed frame phases on a newly-constructed display
/// object.
///
/// This is a no-op on AVM1, which has it's own catch-up logic.
pub fn catchup_display_object_to_frame<'gc>(
    context: &mut UpdateContext<'gc>,
    dobj: DisplayObject<'gc>,
) {
    if !dobj.movie().is_action_script_3() {
        return;
    }

    match *context.frame_phase {
        FramePhase::Enter => {
            dobj.enter_frame(context);
        }
        FramePhase::Construct | FramePhase::FrameScripts | FramePhase::Exit | FramePhase::Idle => {
            dobj.enter_frame(context);
            dobj.construct_frame(context);
        }
    }
}
