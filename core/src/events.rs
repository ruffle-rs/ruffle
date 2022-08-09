use crate::display_object::InteractiveObject;
use ruffle_types::events::{ButtonKeyCode, MouseWheelDelta};
use swf::ClipEventFlag;

/// Whether this button event was handled by some child.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClipEventResult {
    NotHandled,
    Handled,
}

/// An event type that can be handled by a movie clip instance.
///
/// Clip events come in three flavors: broadcast, anycast and targeted. An
/// anycast event is provided to the first `DisplayObject` that claims it, in
/// render list order. Targeted events are sent to a particular object and are
/// lost if not handled by the object. Broadcast events are delivered to all
/// objects in the display list tree.
///
/// These events are consumed both by display objects themselves as well as
/// event handlers in AVM1 and AVM2. These have slightly different event
/// handling semantics:
///
///  * AVM1 delivers broadcasts via `ClipEvent` or system listeners
///  * AVM2 delivers broadcasts to all registered `EventDispatcher`s
///  * Anycast events are not delivered to AVM2
///  * Targeted events are supported and consumed by both VMs
///  * AVM2 additionally supports bubble/capture, which AVM1 and
///    `InteractiveObject` itself does not support
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClipEvent<'gc> {
    Construct,
    Data,

    /// Mouse moved out of a display object while the primary button is held
    /// down.
    ///
    /// This is a targeted equivalent to `MouseMove` and is available in both
    /// AVM1 and AVM2. In AVM2, it is dispatched identically to `RollOut`, with
    /// the only difference being that the `buttonDown` flag is set to true.
    DragOut {
        to: Option<InteractiveObject<'gc>>,
    },

    /// Mouse moved into of a display object while the primary button is held
    /// down.
    ///
    /// This is a targeted equivalent to `MouseMove` and is available in both
    /// AVM1 and AVM2. In AVM2, it is dispatched identically to `RollOver`,
    /// with the only difference being that the `buttonDown` flag is set to
    /// true.
    DragOver {
        from: Option<InteractiveObject<'gc>>,
    },
    EnterFrame,
    Initialize,
    KeyUp,
    KeyDown,
    KeyPress {
        key_code: ButtonKeyCode,
    },
    Load,

    /// Mouse button was released.
    ///
    /// This is an anycast event.
    MouseUp,

    /// Mouse button was released inside this current display object.
    ///
    /// This is a targeted equivalent to `MouseUp` and corresponds directly to
    /// the AVM2 `mouseUp` event, which has no AVM1 equivalent. The target of
    /// this event is determined by the position of the mouse cursor.
    MouseUpInside,

    /// Mouse button was pressed.
    ///
    /// This is an anycast event.
    MouseDown,

    /// Mouse was moved.
    ///
    /// This is an anycast event.
    MouseMove,

    /// Mouse was moved inside this current display object.
    ///
    /// This is a targeted equivalent to `MouseMove` to support AVM2's
    /// `mouseMove` event, since AVM2 cannot consume anycast events.
    MouseMoveInside,

    /// Mouse button was pressed inside this current display object.
    ///
    /// This is a targeted equivalent to `MouseDown` and is available in both
    /// AVM1 and AVM2. The target of this event is determined by the position
    /// of the mouse cursor.
    Press,

    /// Mouse moved out of a display object.
    ///
    /// This is a targeted equivalent to `MouseMove` and is available in both
    /// AVM1 and AVM2. Confusingly, it covers both `mouseOut` and `rollOut`,
    /// the difference being that the former bubbles, while the latter only
    /// fires when the cursor has left the parent *and* it's children.
    ///
    /// The parameter `to` is the current object that is now under the cursor.
    RollOut {
        to: Option<InteractiveObject<'gc>>,
    },

    /// Mouse moved into a display object.
    ///
    /// This is a targeted equivalent to `MouseMove` and is available in both
    /// AVM1 and AVM2. Confusingly, it covers both `mouseOver` and `rollOver`,
    /// the difference being that the former bubbles, while the latter only
    /// fires when the cursor has left the parent *and* it's children.
    ///
    /// The parameter `from` is the previous object that was under the cursor
    /// before this one.
    RollOver {
        from: Option<InteractiveObject<'gc>>,
    },

    /// Mouse button was released inside a previously-pressed display object.
    ///
    /// This is a targeted equivalent to `MouseUp` and is available in both
    /// AVM1 and AVM2. The target of this event is the last target of the
    /// `Press` event.
    Release,

    /// Mouse button was released outside a previously-pressed display object.
    ///
    /// This is a targeted equivalent to `MouseUp` and is available in both
    /// AVM1 and AVM2. The target of this event is the last target of the
    /// `Press` event.
    ReleaseOutside,
    Unload,

    /// Mouse wheel was turned over a particular display object.
    ///
    /// This is a targeted event with no anycast equivalent. It is targeted to
    /// any interactive object under the mouse cursor, including the stage
    /// itself. Only AVM2 can recieve these events.
    MouseWheel {
        delta: MouseWheelDelta,
    },
}

impl<'gc> ClipEvent<'gc> {
    /// Method names for button event handles.
    pub const BUTTON_EVENT_METHODS: [&'static str; 7] = [
        "onDragOver",
        "onDragOut",
        "onPress",
        "onRelease",
        "onReleaseOutside",
        "onRollOut",
        "onRollOver",
    ];

    pub const BUTTON_EVENT_FLAGS: ClipEventFlag = ClipEventFlag::from_bits_truncate(
        ClipEventFlag::DRAG_OUT.bits()
            | ClipEventFlag::DRAG_OVER.bits()
            | ClipEventFlag::KEY_PRESS.bits()
            | ClipEventFlag::PRESS.bits()
            | ClipEventFlag::ROLL_OUT.bits()
            | ClipEventFlag::ROLL_OVER.bits()
            | ClipEventFlag::RELEASE.bits()
            | ClipEventFlag::RELEASE_OUTSIDE.bits(),
    );

    /// Returns the `swf::ClipEventFlag` corresponding to this event type.
    pub const fn flag(self) -> Option<ClipEventFlag> {
        match self {
            ClipEvent::Construct => Some(ClipEventFlag::CONSTRUCT),
            ClipEvent::Data => Some(ClipEventFlag::DATA),
            ClipEvent::DragOut { .. } => Some(ClipEventFlag::DRAG_OUT),
            ClipEvent::DragOver { .. } => Some(ClipEventFlag::DRAG_OVER),
            ClipEvent::EnterFrame => Some(ClipEventFlag::ENTER_FRAME),
            ClipEvent::Initialize => Some(ClipEventFlag::INITIALIZE),
            ClipEvent::KeyDown => Some(ClipEventFlag::KEY_DOWN),
            ClipEvent::KeyPress { .. } => Some(ClipEventFlag::KEY_PRESS),
            ClipEvent::KeyUp => Some(ClipEventFlag::KEY_UP),
            ClipEvent::Load => Some(ClipEventFlag::LOAD),
            ClipEvent::MouseDown => Some(ClipEventFlag::MOUSE_DOWN),
            ClipEvent::MouseMove => Some(ClipEventFlag::MOUSE_MOVE),
            ClipEvent::MouseUp => Some(ClipEventFlag::MOUSE_UP),
            ClipEvent::Press => Some(ClipEventFlag::PRESS),
            ClipEvent::RollOut { .. } => Some(ClipEventFlag::ROLL_OUT),
            ClipEvent::RollOver { .. } => Some(ClipEventFlag::ROLL_OVER),
            ClipEvent::Release => Some(ClipEventFlag::RELEASE),
            ClipEvent::ReleaseOutside => Some(ClipEventFlag::RELEASE_OUTSIDE),
            ClipEvent::Unload => Some(ClipEventFlag::UNLOAD),
            ClipEvent::MouseWheel { .. }
            | ClipEvent::MouseMoveInside
            | ClipEvent::MouseUpInside => None,
        }
    }

    /// Indicates that the event should be propagated down to children.
    pub const fn propagates(self) -> bool {
        matches!(
            self,
            Self::MouseUp
                | Self::MouseDown
                | Self::MouseMove
                | Self::KeyPress { .. }
                | Self::KeyDown
                | Self::KeyUp
        )
    }

    /// Indicates whether this is an event type used by Buttons (i.e., on that can be used in an `on` handler in Flash).
    pub const fn is_button_event(self) -> bool {
        if let Some(flag) = self.flag() {
            flag.intersects(Self::BUTTON_EVENT_FLAGS)
        } else {
            false
        }
    }

    /// Indicates whether this is a keyboard event type (keyUp, keyDown, keyPress).
    pub const fn is_key_event(self) -> bool {
        matches!(self, Self::KeyDown | Self::KeyUp | Self::KeyPress { .. })
    }

    /// Returns the method name of the event handler for this event.
    ///
    /// `ClipEvent::Data` returns `None` rather than `onData` because its behavior
    /// differs from the other events: the method must fire before the SWF-defined
    /// event handler, so we'll explicitly call `onData` in the appropriate places.
    pub const fn method_name(self) -> Option<&'static str> {
        match self {
            ClipEvent::Construct => None,
            ClipEvent::Data => None,
            ClipEvent::DragOut { .. } => Some("onDragOut"),
            ClipEvent::DragOver { .. } => Some("onDragOver"),
            ClipEvent::EnterFrame => Some("onEnterFrame"),
            ClipEvent::Initialize => None,
            ClipEvent::KeyDown => Some("onKeyDown"),
            ClipEvent::KeyPress { .. } => None,
            ClipEvent::KeyUp => Some("onKeyUp"),
            ClipEvent::Load => Some("onLoad"),
            ClipEvent::MouseDown => Some("onMouseDown"),
            ClipEvent::MouseMove => Some("onMouseMove"),
            ClipEvent::MouseUp => Some("onMouseUp"),
            ClipEvent::Press => Some("onPress"),
            ClipEvent::RollOut { .. } => Some("onRollOut"),
            ClipEvent::RollOver { .. } => Some("onRollOver"),
            ClipEvent::Release => Some("onRelease"),
            ClipEvent::ReleaseOutside => Some("onReleaseOutside"),
            ClipEvent::Unload => Some("onUnload"),
            ClipEvent::MouseWheel { .. }
            | ClipEvent::MouseMoveInside
            | ClipEvent::MouseUpInside => None,
        }
    }
}
