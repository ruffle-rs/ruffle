mod display_object;
mod handle;

use crate::context::UpdateContext;
use crate::debug_ui::display_object::DisplayObjectWindow;
use crate::debug_ui::handle::DisplayObjectHandle;
use hashbrown::HashMap;

#[derive(Default)]
pub struct DebugUi {
    display_objects: HashMap<DisplayObjectHandle, DisplayObjectWindow>,
    queued_messages: Vec<Message>,
}

#[derive(Debug)]
pub enum Message {
    TrackDisplayObject(DisplayObjectHandle),
    TrackStage,
}

impl DebugUi {
    pub fn show(&mut self, egui_ctx: &egui::Context, context: &mut UpdateContext) {
        let mut messages = std::mem::take(&mut self.queued_messages);
        self.display_objects.retain(|object, window| {
            let object = object.fetch(context);
            window.show(egui_ctx, context, object, &mut messages)
        });
        for message in messages {
            match message {
                Message::TrackDisplayObject(object) => self.track_display_object(object),
                Message::TrackStage => {
                    self.track_display_object(DisplayObjectHandle::new(context, context.stage))
                }
            }
        }
    }

    pub fn queue_message(&mut self, message: Message) {
        self.queued_messages.push(message);
    }

    pub fn track_display_object(&mut self, handle: DisplayObjectHandle) {
        self.display_objects.insert(handle, Default::default());
    }
}
