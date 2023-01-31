use crate::context::UpdateContext;
use crate::debug::debug_message_out::DebugMessageOut;
use crate::debug::player_message::PlayerMsg;
use crate::display_object::DisplayObject;
use crate::display_object::TDisplayObject;

pub mod avm1_debugger;
pub mod avm1_message;
pub mod debug_message_in;
pub mod debug_message_out;
pub mod debug_provider;
pub mod debug_value;
pub mod debuggable;
pub mod display_object_info;
pub mod movie_clip_debugger;
pub mod player_message;
pub mod targeted_message;

use crate::display_object::TDisplayObjectContainer;
use debug_provider::DebugProvider;

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

/// Walk a path, returning the display object at that point in the depth-tree, if it exists
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
            let dp = path.split('/').collect::<Vec<_>>();
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
