//! Interactive object enumtrait

use crate::display_object::avm2_button::Avm2Button;
use crate::display_object::edit_text::EditText;
use crate::display_object::movie_clip::MovieClip;
use crate::display_object::stage::Stage;
use bitflags::bitflags;
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

bitflags! {
    /// Boolean state flags used by `InteractiveObject`.
    #[derive(Collect)]
    #[collect(require_static)]
    struct InteractiveObjectFlags: u8 {
        /// Whether this `InteractiveObject` accepts mouse and other user
        /// events.
        const MOUSE_ENABLED = 1 << 0;

        /// Whether this `InteractiveObject` accepts double-clicks.
        const DOUBLE_CLICK_ENABLED = 1 << 1;
    }
}

#[derive(Collect, Clone, Debug)]
#[collect(no_drop)]
pub struct InteractiveObjectBase {
    flags: InteractiveObjectFlags,
}

impl Default for InteractiveObjectBase {
    fn default() -> Self {
        Self {
            flags: InteractiveObjectFlags::MOUSE_ENABLED,
        }
    }
}

#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum InteractiveObject<'gc> {
        Stage(Stage<'gc>),
        Avm2Button(Avm2Button<'gc>),
        MovieClip(MovieClip<'gc>),
        EditText(EditText<'gc>),
    }
)]
pub trait TInteractiveObject<'gc>:
    'gc + Clone + Copy + Collect + Debug + Into<InteractiveObject<'gc>>
{
    fn base(&self) -> Ref<InteractiveObjectBase>;

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<InteractiveObjectBase>;

    /// Check if the interactive object accepts user input.
    fn mouse_enabled(self) -> bool {
        self.base()
            .flags
            .contains(InteractiveObjectFlags::MOUSE_ENABLED)
    }

    /// Set if the interactive object accepts user input.
    fn set_mouse_enabled(self, mc: MutationContext<'gc, '_>, value: bool) {
        self.base_mut(mc)
            .flags
            .set(InteractiveObjectFlags::MOUSE_ENABLED, value)
    }

    /// Check if the interactive object accepts double-click events.
    fn double_click_enabled(self) -> bool {
        self.base()
            .flags
            .contains(InteractiveObjectFlags::DOUBLE_CLICK_ENABLED)
    }

    // Set if the interactive object accepts double-click events.
    fn set_double_click_enabled(self, mc: MutationContext<'gc, '_>, value: bool) {
        self.base_mut(mc)
            .flags
            .set(InteractiveObjectFlags::DOUBLE_CLICK_ENABLED, value)
    }
}
