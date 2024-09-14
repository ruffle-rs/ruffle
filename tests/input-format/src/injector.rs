//! Automated input injector

use crate::format::{AutomatedEvent, MouseButton};
use bitflags::bitflags;
use serde_json::from_reader;
use std::fs::File;
use std::io;
use std::path::Path;

bitflags! {
    /// A set of currently held-down mouse buttons.
    ///
    /// Convertible from `MouseButton`, which is intended to represent ONE
    /// button being held or released.
    #[derive(Clone, Copy)]
    pub struct MouseButtons: u8 {
        const LEFT = 0b00000001;
        const MIDDLE = 0b00000010;
        const RIGHT = 0b00000100;
    }
}

impl From<MouseButton> for MouseButtons {
    fn from(btn: MouseButton) -> MouseButtons {
        match btn {
            MouseButton::Left => MouseButtons::LEFT,
            MouseButton::Middle => MouseButtons::MIDDLE,
            MouseButton::Right => MouseButtons::RIGHT,
        }
    }
}

pub struct InputInjector {
    /// The list of events to inject.
    items: Vec<AutomatedEvent>,

    /// The current event position within that list.
    pos: usize,

    /// The current set of held-down buttons.
    buttons: MouseButtons,
}

impl InputInjector {
    /// Construct an input injector from an input file and a platform-specific
    /// event sink.
    pub fn from_file<P>(path: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;

        Ok(Self {
            items: from_reader(file)?,
            pos: 0,
            buttons: MouseButtons::empty(),
        })
    }

    /// Construct an input injector from an input reader and a platform-specific
    /// event sink.
    pub fn from_reader<R>(reader: R) -> Result<Self, io::Error>
    where
        R: io::Read,
    {
        Ok(Self {
            items: from_reader(reader)?,
            pos: 0,
            buttons: MouseButtons::empty(),
        })
    }

    /// Create an empty input injector with no input to inject.
    ///
    /// Useful to represent a missing input file in cases where providing one
    /// is optional.
    pub fn empty() -> Self {
        Self {
            items: vec![],
            pos: 0,
            buttons: MouseButtons::empty(),
        }
    }

    /// Run the next frame's worth of events.
    pub fn next<Sink>(&mut self, mut event_sink: Sink)
    where
        Sink: FnMut(&AutomatedEvent, MouseButtons),
    {
        let mut pos = self.pos;
        if let Some(events) = self.items.get(pos..) {
            for event in events {
                pos += 1;

                match event {
                    AutomatedEvent::Wait => break,
                    AutomatedEvent::MouseMove { .. }
                    | AutomatedEvent::KeyDown { .. }
                    | AutomatedEvent::KeyUp { .. }
                    | AutomatedEvent::TextInput { .. }
                    | AutomatedEvent::TextControl { .. }
                    | AutomatedEvent::SetClipboardText { .. }
                    | AutomatedEvent::MouseWheel { .. }
                    | AutomatedEvent::FocusGained
                    | AutomatedEvent::FocusLost => {}
                    AutomatedEvent::MouseDown { btn, .. } => {
                        self.buttons |= (*btn).into();
                    }
                    AutomatedEvent::MouseUp { btn, .. } => {
                        let mask: MouseButtons = (*btn).into();
                        self.buttons &= !mask;
                    }
                }

                event_sink(event, self.buttons);
            }
        }

        self.pos = pos;
    }
}
