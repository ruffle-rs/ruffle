use crate::display_object::InteractiveObject;
use swf::ClipEventFlag;

#[derive(Debug, Clone, Copy)]
pub enum PlayerEvent {
    KeyDown {
        key_code: KeyCode,
        key_char: Option<char>,
    },
    KeyUp {
        key_code: KeyCode,
        key_char: Option<char>,
    },
    MouseMove {
        x: f64,
        y: f64,
    },
    MouseUp {
        x: f64,
        y: f64,
        button: MouseButton,
    },
    MouseDown {
        x: f64,
        y: f64,
        button: MouseButton,
        index: Option<usize>,
    },
    MouseLeave,
    MouseWheel {
        delta: MouseWheelDelta,
    },
    GamepadButtonDown {
        button: GamepadButton,
    },
    GamepadButtonUp {
        button: GamepadButton,
    },
    TextInput {
        codepoint: char,
    },
    TextControl {
        code: TextControlCode,
    },
    FocusGained,
    FocusLost,
}

/// The distance scrolled by the mouse wheel.
#[derive(Debug, Clone, Copy)]
pub enum MouseWheelDelta {
    Lines(f64),
    Pixels(f64),
}

impl MouseWheelDelta {
    const MOUSE_WHEEL_SCALE: f64 = 100.0;

    /// Returns the number of lines that this delta represents.
    pub fn lines(self) -> f64 {
        // TODO: Should we always return an integer here?
        match self {
            Self::Lines(delta) => delta,
            Self::Pixels(delta) => delta / Self::MOUSE_WHEEL_SCALE,
        }
    }
}

impl PartialEq for MouseWheelDelta {
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Lines(s), Self::Lines(r))
            | (Self::Pixels(s), Self::Pixels(r))
            | (Self::Pixels(s), Self::Lines(r))
            | (Self::Lines(s), Self::Pixels(r))
                if s.is_nan() && r.is_nan() =>
            {
                true
            }
            (Self::Lines(s), Self::Lines(r)) => s == r,
            (Self::Pixels(s), Self::Pixels(r)) => s == r,
            (Self::Pixels(s), Self::Lines(r)) => *s == r * Self::MOUSE_WHEEL_SCALE,
            (Self::Lines(s), Self::Pixels(r)) => s * Self::MOUSE_WHEEL_SCALE == *r,
        }
    }
}

impl Eq for MouseWheelDelta {}

/// Whether this button event was handled by some child.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClipEventResult {
    NotHandled,
    Handled,
}

impl From<bool> for ClipEventResult {
    fn from(value: bool) -> Self {
        if value {
            Self::Handled
        } else {
            Self::NotHandled
        }
    }
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

    /// Left mouse button was released.
    ///
    /// This is an anycast event.
    MouseUp,

    /// Right mouse button was released.
    ///
    /// Analogous to [`ClipEvent::MouseUp`], but for right mouse button.
    RightMouseUp,

    /// Middle mouse button was released.
    ///
    /// Analogous to [`ClipEvent::MouseUp`], but for middle mouse button.
    MiddleMouseUp,

    /// Left mouse button was released inside this current display object.
    ///
    /// This is a targeted equivalent to `MouseUp` and corresponds directly to
    /// the AVM2 `mouseUp` event, which has no AVM1 equivalent. The target of
    /// this event is determined by the position of the mouse cursor.
    MouseUpInside,

    /// Right mouse button was released inside this current display object.
    ///
    /// Analogous to [`ClipEvent::MouseUpInside`], but for right mouse button.
    RightMouseUpInside,

    /// Middle mouse button was released inside this current display object.
    ///
    /// Analogous to [`ClipEvent::MouseUpInside`], but for middle mouse button.
    MiddleMouseUpInside,

    /// Left mouse button was pressed.
    ///
    /// This is an anycast event.
    MouseDown,

    /// Right mouse button was pressed.
    ///
    /// Analogous to [`ClipEvent::MouseDown`], but for right mouse button.
    RightMouseDown,

    /// Middle mouse button was pressed.
    ///
    /// Analogous to [`ClipEvent::MouseDown`], but for middle mouse button.
    MiddleMouseDown,

    /// Mouse was moved.
    ///
    /// This is an anycast event.
    MouseMove,

    /// Mouse was moved inside this current display object.
    ///
    /// This is a targeted equivalent to `MouseMove` to support AVM2's
    /// `mouseMove` event, since AVM2 cannot consume anycast events.
    MouseMoveInside,

    /// Left mouse button was pressed inside this current display object.
    ///
    /// This is a targeted equivalent to `MouseDown` and is available in both
    /// AVM1 and AVM2. The target of this event is determined by the position
    /// of the mouse cursor.
    Press {
        /// The index of this click in a click sequence performed in a quick succession.
        ///
        /// For instance the value of 0 indicates it's a single click,
        /// the number of 1 indicates it's a double click, etc.
        index: usize,
    },

    /// Right mouse button was pressed inside this current display object.
    ///
    /// Analogous to [`ClipEvent::Press`], but for right mouse button.
    RightPress,

    /// Middle mouse button was pressed inside this current display object.
    ///
    /// Analogous to [`ClipEvent::Press`], but for middle mouse button.
    MiddlePress,

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

    /// Left mouse button was released inside a previously-pressed display object.
    ///
    /// This is a targeted equivalent to `MouseUp` and is available in both
    /// AVM1 and AVM2. The target of this event is the last target of the
    /// `Press` event.
    Release {
        /// The index of this click, same as the index of the last [`ClipEvent::Press`] event.
        index: usize,
    },

    /// Right mouse button was released inside a previously-pressed display object.
    ///
    /// Analogous to [`ClipEvent::Release`], but for right mouse button.
    RightRelease,

    /// Middle mouse button was released inside a previously-pressed display object.
    ///
    /// Analogous to [`ClipEvent::Release`], but for middle mouse button.
    MiddleRelease,

    /// Left mouse button was released outside a previously-pressed display object.
    ///
    /// This is a targeted equivalent to `MouseUp` and is available in both
    /// AVM1 and AVM2. The target of this event is the last target of the
    /// `Press` event.
    ReleaseOutside,

    /// Right mouse button was released outside a previously-pressed display object.
    ///
    /// Analogous to [`ClipEvent::ReleaseOutside`], but for right mouse button.
    RightReleaseOutside,

    /// Middle mouse button was released outside a previously-pressed display object.
    ///
    /// Analogous to [`ClipEvent::ReleaseOutside`], but for middle mouse button.
    MiddleReleaseOutside,

    Unload,

    /// Mouse wheel was turned over a particular display object.
    ///
    /// This is a targeted event with no anycast equivalent. It is targeted to
    /// any interactive object under the mouse cursor, including the stage
    /// itself. Only AVM2 can receive these events.
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
            ClipEvent::Press { .. } => Some(ClipEventFlag::PRESS),
            ClipEvent::RollOut { .. } => Some(ClipEventFlag::ROLL_OUT),
            ClipEvent::RollOver { .. } => Some(ClipEventFlag::ROLL_OVER),
            ClipEvent::Release { .. } => Some(ClipEventFlag::RELEASE),
            ClipEvent::ReleaseOutside => Some(ClipEventFlag::RELEASE_OUTSIDE),
            ClipEvent::Unload => Some(ClipEventFlag::UNLOAD),
            _ => None,
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
            ClipEvent::Press { .. } => Some("onPress"),
            ClipEvent::RollOut { .. } => Some("onRollOut"),
            ClipEvent::RollOver { .. } => Some("onRollOver"),
            ClipEvent::Release { .. } => Some("onRelease"),
            ClipEvent::ReleaseOutside => Some("onReleaseOutside"),
            ClipEvent::Unload => Some("onUnload"),
            _ => None,
        }
    }
}

/// Control inputs to a text field
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub enum TextControlCode {
    MoveLeft,
    MoveLeftWord,
    MoveLeftLine,
    MoveLeftDocument,
    MoveRight,
    MoveRightWord,
    MoveRightLine,
    MoveRightDocument,
    SelectLeft,
    SelectLeftWord,
    SelectLeftLine,
    SelectLeftDocument,
    SelectRight,
    SelectRightWord,
    SelectRightLine,
    SelectRightDocument,
    SelectAll,
    Copy,
    Paste,
    Cut,
    Backspace,
    BackspaceWord,
    Enter,
    Delete,
    DeleteWord,
}

impl TextControlCode {
    /// Indicates whether this is an event that edits the text content
    pub fn is_edit_input(self) -> bool {
        matches!(
            self,
            Self::Paste
                | Self::Cut
                | Self::Enter
                | Self::Backspace
                | Self::BackspaceWord
                | Self::Delete
                | Self::DeleteWord
        )
    }
}

/// Flash virtual keycode.
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, FromPrimitive)]
pub enum KeyCode {
    Unknown = 0,
    MouseLeft = 1,
    MouseRight = 2,
    MouseMiddle = 4,
    Backspace = 8,
    Tab = 9,
    Return = 13,
    Command = 15,
    Shift = 16,
    Control = 17,
    Alt = 18,
    Pause = 19,
    CapsLock = 20,
    Numpad = 21,
    Escape = 27,
    Space = 32,
    PgUp = 33,
    PgDown = 34,
    End = 35,
    Home = 36,
    Left = 37,
    Up = 38,
    Right = 39,
    Down = 40,
    Insert = 45,
    Delete = 46,
    Key0 = 48,
    Key1 = 49,
    Key2 = 50,
    Key3 = 51,
    Key4 = 52,
    Key5 = 53,
    Key6 = 54,
    Key7 = 55,
    Key8 = 56,
    Key9 = 57,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    Numpad0 = 96,
    Numpad1 = 97,
    Numpad2 = 98,
    Numpad3 = 99,
    Numpad4 = 100,
    Numpad5 = 101,
    Numpad6 = 102,
    Numpad7 = 103,
    Numpad8 = 104,
    Numpad9 = 105,
    Multiply = 106,
    Plus = 107,
    NumpadEnter = 108,
    NumpadMinus = 109,
    NumpadPeriod = 110,
    NumpadSlash = 111,
    F1 = 112,
    F2 = 113,
    F3 = 114,
    F4 = 115,
    F5 = 116,
    F6 = 117,
    F7 = 118,
    F8 = 119,
    F9 = 120,
    F10 = 121,
    F11 = 122,
    F12 = 123,
    F13 = 124,
    F14 = 125,
    F15 = 126,
    F16 = 127, // undocumented
    F17 = 128, // undocumented
    F18 = 129, // undocumented
    F19 = 130, // undocumented
    F20 = 131, // undocumented
    F21 = 132, // undocumented
    F22 = 133, // undocumented
    F23 = 134, // undocumented
    F24 = 135, // undocumented
    NumLock = 144,
    ScrollLock = 145,
    Semicolon = 186,
    Equals = 187,
    Comma = 188,
    Minus = 189,
    Period = 190,
    Slash = 191,
    Grave = 192,
    LBracket = 219,
    Backslash = 220,
    RBracket = 221,
    Apostrophe = 222,
}

impl KeyCode {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

/// Subset of `KeyCode` that contains only mouse buttons.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Unknown = KeyCode::Unknown as isize,
    Left = KeyCode::MouseLeft as isize,
    Right = KeyCode::MouseRight as isize,
    Middle = KeyCode::MouseMiddle as isize,
}

impl From<MouseButton> for KeyCode {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Unknown => Self::Unknown,
            MouseButton::Left => Self::MouseLeft,
            MouseButton::Right => Self::MouseRight,
            MouseButton::Middle => Self::MouseMiddle,
        }
    }
}

/// Key codes for SWF4 keyPress button handlers. These are annoyingly different than
/// `Key.isDown` key codes.
/// TODO: After 18, these are mostly ASCII... should we just use u8? How are different
/// keyboard layouts/languages handled?
/// SWF19 pp. 198-199
#[derive(Debug, PartialEq, Eq, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum ButtonKeyCode {
    Unknown = 0,
    Left = 1,
    Right = 2,
    Home = 3,
    End = 4,
    Insert = 5,
    Delete = 6,
    Backspace = 8,
    Return = 13,
    Up = 14,
    Down = 15,
    PgUp = 16,
    PgDown = 17,
    Tab = 18,
    Escape = 19,
    Space = 32,
    Exclamation = 33,
    DoubleQuote = 34,
    NumberSign = 35,
    Dollar = 36,
    Percent = 37,
    Ampersand = 38,
    SingleQuote = 39,
    LParen = 40,
    RParen = 41,
    Asterisk = 42,
    Plus = 43,
    Comma = 44,
    Minus = 45,
    Period = 46,
    Slash = 47,
    Zero = 48,
    One = 49,
    Two = 50,
    Three = 51,
    Four = 52,
    Five = 53,
    Six = 54,
    Seven = 55,
    Eight = 56,
    Nine = 57,
    Colon = 58,
    Semicolon = 59,
    LessThan = 60,
    Equals = 61,
    GreaterThan = 62,
    Question = 63,
    At = 64,
    UppercaseA = 65,
    UppercaseB = 66,
    UppercaseC = 67,
    UppercaseD = 68,
    UppercaseE = 69,
    UppercaseF = 70,
    UppercaseG = 71,
    UppercaseH = 72,
    UppercaseI = 73,
    UppercaseJ = 74,
    UppercaseK = 75,
    UppercaseL = 76,
    UppercaseM = 77,
    UppercaseN = 78,
    UppercaseO = 79,
    UppercaseP = 80,
    UppercaseQ = 81,
    UppercaseR = 82,
    UppercaseS = 83,
    UppercaseT = 84,
    UppercaseU = 85,
    UppercaseV = 86,
    UppercaseW = 87,
    UppercaseX = 88,
    UppercaseY = 89,
    UppercaseZ = 90,
    LBracket = 91,
    Backslash = 92,
    RBracket = 93,
    Caret = 94,
    Underscore = 95,
    Backquote = 96,
    A = 97,
    B = 98,
    C = 99,
    D = 100,
    E = 101,
    F = 102,
    G = 103,
    H = 104,
    I = 105,
    J = 106,
    K = 107,
    L = 108,
    M = 109,
    N = 110,
    O = 111,
    P = 112,
    Q = 113,
    R = 114,
    S = 115,
    T = 116,
    U = 117,
    V = 118,
    W = 119,
    X = 120,
    Y = 121,
    Z = 122,
    LBrace = 123,
    Pipe = 124,
    RBrace = 125,
    Tilde = 126,
}

impl ButtonKeyCode {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }

    pub fn to_u8(&self) -> u8 {
        num_traits::ToPrimitive::to_u8(self).unwrap_or_default()
    }
}

pub fn key_code_to_button_key_code(key_code: KeyCode) -> Option<ButtonKeyCode> {
    let out = match key_code {
        KeyCode::Left => ButtonKeyCode::Left,
        KeyCode::Right => ButtonKeyCode::Right,
        KeyCode::Home => ButtonKeyCode::Home,
        KeyCode::End => ButtonKeyCode::End,
        KeyCode::Insert => ButtonKeyCode::Insert,
        KeyCode::Delete => ButtonKeyCode::Delete,
        KeyCode::Backspace => ButtonKeyCode::Backspace,
        KeyCode::Return => ButtonKeyCode::Return,
        KeyCode::Up => ButtonKeyCode::Up,
        KeyCode::Down => ButtonKeyCode::Down,
        KeyCode::PgUp => ButtonKeyCode::PgUp,
        KeyCode::PgDown => ButtonKeyCode::PgDown,
        KeyCode::Escape => ButtonKeyCode::Escape,
        KeyCode::Tab => ButtonKeyCode::Tab,
        _ => return None,
    };
    Some(out)
}

#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}
