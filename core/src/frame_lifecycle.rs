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

use crate::avm2::Avm2;
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject};
use fnv::FnvHashSet;
use tracing::instrument;

/// Which phase of the frame we're currently in.
///
/// AVM2 frames exist in one of five phases: `Enter`, `Construct`, `Update`,
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
    #[default]
    Idle,
}

/// Run one frame according to AVM2 frame order.
/// NOTE: The `each_orphan_movie` calls are in really odd places,
/// but this is needed to match Flash Player's output. There may
/// still be lurking bugs, but the current code matches Flash's
/// output exactly for two complex test cases (see `avm2/orphan_movie*`)
#[instrument(level = "debug", skip_all)]
pub fn run_all_phases_avm2(context: &mut UpdateContext<'_, '_>) {
    let stage = context.stage;

    *context.frame_phase = FramePhase::Enter;

    stage.enter_frame(context);

    let mut ran_framescript = FnvHashSet::default();

    Avm2::each_orphan_movie(context, |movie, context| {
        if movie.initialized() {
            // Orphan frame scripts run in a really weird place.
            // Running them here matches the output we get from Flash Player,
            // where the currentFrame field field hasn't been updated yet
            // (but is updated when we call an enterFrame listener)
            movie.run_frame_scripts(context);
            movie.enter_frame(context);
            ran_framescript.insert(movie.downgrade().as_ptr());
        }
    });

    *context.frame_phase = FramePhase::Construct;

    Avm2::each_orphan_movie(context, |movie, context| {
        if !movie.initialized() {
            movie.run_frame_scripts(context);
            ran_framescript.insert(movie.downgrade().as_ptr());
        }
    });

    stage.construct_frame(context);

    Avm2::each_orphan_movie(context, |movie, context| {
        movie.construct_frame(context);
        if !ran_framescript.contains(&movie.downgrade().as_ptr()) {
            movie.run_frame_scripts(context);
        }
    });

    stage.frame_constructed(context);

    *context.frame_phase = FramePhase::Update;
    stage.run_frame_avm2(context);

    Avm2::each_orphan_movie(context, |movie, context| {
        movie.run_frame_avm2(context);
    });

    *context.frame_phase = FramePhase::FrameScripts;

    stage.run_frame_scripts(context);

    Avm2::each_orphan_movie(context, |movie, context| {
        if !movie.initialized() {
            movie.enter_frame(context);
        }
    });

    *context.frame_phase = FramePhase::Exit;
    stage.exit_frame(context);

    Avm2::each_orphan_movie(context, |movie, context| {
        movie.on_exit_frame(context);
    });

    // We cannot easily remove dead `GcWeak` instances from the orphan list
    // inside `each_orphan_movie`, since the callback may modify the orphan list.
    // Instead, we do one cleanup at the end of the frame.
    Avm2::cleanup_dead_orphans(context);

    *context.frame_phase = FramePhase::Idle;
}

/// Run all previously-executed frame phases on a newly-constructed display
/// object.
///
/// This is a no-op on AVM1, which has it's own catch-up logic.
pub fn catchup_display_object_to_frame<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    dobj: DisplayObject<'gc>,
) {
    if !context.is_action_script_3() {
        return;
    }

    match *context.frame_phase {
        FramePhase::Enter => {
            dobj.enter_frame(context);
        }
        FramePhase::Construct => {
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
