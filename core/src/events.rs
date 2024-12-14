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

impl ClipEvent<'_> {
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KeyCode(u32);

impl KeyCode {
    pub const UNKNOWN: KeyCode = KeyCode(0);
    pub const MOUSE_LEFT: KeyCode = KeyCode(1);
    pub const MOUSE_RIGHT: KeyCode = KeyCode(2);
    pub const MOUSE_MIDDLE: KeyCode = KeyCode(4);
    pub const BACKSPACE: KeyCode = KeyCode(8);
    pub const TAB: KeyCode = KeyCode(9);
    pub const RETURN: KeyCode = KeyCode(13);
    pub const COMMAND: KeyCode = KeyCode(15);
    pub const SHIFT: KeyCode = KeyCode(16);
    pub const CONTROL: KeyCode = KeyCode(17);
    pub const ALT: KeyCode = KeyCode(18);
    pub const PAUSE: KeyCode = KeyCode(19);
    pub const CAPS_LOCK: KeyCode = KeyCode(20);
    pub const NUMPAD: KeyCode = KeyCode(21);
    pub const ESCAPE: KeyCode = KeyCode(27);
    pub const SPACE: KeyCode = KeyCode(32);
    pub const PG_UP: KeyCode = KeyCode(33);
    pub const PG_DOWN: KeyCode = KeyCode(34);
    pub const END: KeyCode = KeyCode(35);
    pub const HOME: KeyCode = KeyCode(36);
    pub const LEFT: KeyCode = KeyCode(37);
    pub const UP: KeyCode = KeyCode(38);
    pub const RIGHT: KeyCode = KeyCode(39);
    pub const DOWN: KeyCode = KeyCode(40);
    pub const INSERT: KeyCode = KeyCode(45);
    pub const DELETE: KeyCode = KeyCode(46);
    pub const KEY0: KeyCode = KeyCode(48);
    pub const KEY1: KeyCode = KeyCode(49);
    pub const KEY2: KeyCode = KeyCode(50);
    pub const KEY3: KeyCode = KeyCode(51);
    pub const KEY4: KeyCode = KeyCode(52);
    pub const KEY5: KeyCode = KeyCode(53);
    pub const KEY6: KeyCode = KeyCode(54);
    pub const KEY7: KeyCode = KeyCode(55);
    pub const KEY8: KeyCode = KeyCode(56);
    pub const KEY9: KeyCode = KeyCode(57);
    pub const A: KeyCode = KeyCode(65);
    pub const B: KeyCode = KeyCode(66);
    pub const C: KeyCode = KeyCode(67);
    pub const D: KeyCode = KeyCode(68);
    pub const E: KeyCode = KeyCode(69);
    pub const F: KeyCode = KeyCode(70);
    pub const G: KeyCode = KeyCode(71);
    pub const H: KeyCode = KeyCode(72);
    pub const I: KeyCode = KeyCode(73);
    pub const J: KeyCode = KeyCode(74);
    pub const K: KeyCode = KeyCode(75);
    pub const L: KeyCode = KeyCode(76);
    pub const M: KeyCode = KeyCode(77);
    pub const N: KeyCode = KeyCode(78);
    pub const O: KeyCode = KeyCode(79);
    pub const P: KeyCode = KeyCode(80);
    pub const Q: KeyCode = KeyCode(81);
    pub const R: KeyCode = KeyCode(82);
    pub const S: KeyCode = KeyCode(83);
    pub const T: KeyCode = KeyCode(84);
    pub const U: KeyCode = KeyCode(85);
    pub const V: KeyCode = KeyCode(86);
    pub const W: KeyCode = KeyCode(87);
    pub const X: KeyCode = KeyCode(88);
    pub const Y: KeyCode = KeyCode(89);
    pub const Z: KeyCode = KeyCode(90);
    pub const NUMPAD0: KeyCode = KeyCode(96);
    pub const NUMPAD1: KeyCode = KeyCode(97);
    pub const NUMPAD2: KeyCode = KeyCode(98);
    pub const NUMPAD3: KeyCode = KeyCode(99);
    pub const NUMPAD4: KeyCode = KeyCode(100);
    pub const NUMPAD5: KeyCode = KeyCode(101);
    pub const NUMPAD6: KeyCode = KeyCode(102);
    pub const NUMPAD7: KeyCode = KeyCode(103);
    pub const NUMPAD8: KeyCode = KeyCode(104);
    pub const NUMPAD9: KeyCode = KeyCode(105);
    pub const MULTIPLY: KeyCode = KeyCode(106);
    pub const PLUS: KeyCode = KeyCode(107);
    pub const NUMPAD_ENTER: KeyCode = KeyCode(108);
    pub const NUMPAD_MINUS: KeyCode = KeyCode(109);
    pub const NUMPAD_PERIOD: KeyCode = KeyCode(110);
    pub const NUMPAD_SLASH: KeyCode = KeyCode(111);
    pub const F1: KeyCode = KeyCode(112);
    pub const F2: KeyCode = KeyCode(113);
    pub const F3: KeyCode = KeyCode(114);
    pub const F4: KeyCode = KeyCode(115);
    pub const F5: KeyCode = KeyCode(116);
    pub const F6: KeyCode = KeyCode(117);
    pub const F7: KeyCode = KeyCode(118);
    pub const F8: KeyCode = KeyCode(119);
    pub const F9: KeyCode = KeyCode(120);
    pub const F10: KeyCode = KeyCode(121);
    pub const F11: KeyCode = KeyCode(122);
    pub const F12: KeyCode = KeyCode(123);
    pub const F13: KeyCode = KeyCode(124);
    pub const F14: KeyCode = KeyCode(125);
    pub const F15: KeyCode = KeyCode(126);
    pub const F16: KeyCode = KeyCode(127); // undocumented
    pub const F17: KeyCode = KeyCode(128); // undocumented
    pub const F18: KeyCode = KeyCode(129); // undocumented
    pub const F19: KeyCode = KeyCode(130); // undocumented
    pub const F20: KeyCode = KeyCode(131); // undocumented
    pub const F21: KeyCode = KeyCode(132); // undocumented
    pub const F22: KeyCode = KeyCode(133); // undocumented
    pub const F23: KeyCode = KeyCode(134); // undocumented
    pub const F24: KeyCode = KeyCode(135); // undocumented
    pub const NUM_LOCK: KeyCode = KeyCode(144);
    pub const SCROLL_LOCK: KeyCode = KeyCode(145);
    pub const SEMICOLON: KeyCode = KeyCode(186);
    pub const EQUALS: KeyCode = KeyCode(187);
    pub const COMMA: KeyCode = KeyCode(188);
    pub const MINUS: KeyCode = KeyCode(189);
    pub const PERIOD: KeyCode = KeyCode(190);
    pub const SLASH: KeyCode = KeyCode(191);
    pub const GRAVE: KeyCode = KeyCode(192);
    pub const LBRACKET: KeyCode = KeyCode(219);
    pub const BACKSLASH: KeyCode = KeyCode(220);
    pub const RBRACKET: KeyCode = KeyCode(221);
    pub const APOSTROPHE: KeyCode = KeyCode(222);

    #[inline]
    pub const fn from_code(code: u32) -> Self {
        Self(code)
    }

    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Subset of `KeyCode` that contains only mouse buttons.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Unknown,
    Left,
    Right,
    Middle,
}

impl From<MouseButton> for KeyCode {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Unknown => Self::UNKNOWN,
            MouseButton::Left => Self::MOUSE_LEFT,
            MouseButton::Right => Self::MOUSE_RIGHT,
            MouseButton::Middle => Self::MOUSE_MIDDLE,
        }
    }
}

/// Key codes for SWF4 keyPress button handlers.
///
/// These are annoyingly different than `Key.isDown` key codes.
///
/// TODO: After 18, these are mostly ASCII... should we just use u8? How are different
///   keyboard layouts/languages handled?
///   SWF19 pp. 198-199
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

    pub fn from_key_code(key_code: KeyCode) -> Option<Self> {
        Some(match key_code {
            KeyCode::LEFT => ButtonKeyCode::Left,
            KeyCode::RIGHT => ButtonKeyCode::Right,
            KeyCode::HOME => ButtonKeyCode::Home,
            KeyCode::END => ButtonKeyCode::End,
            KeyCode::INSERT => ButtonKeyCode::Insert,
            KeyCode::DELETE => ButtonKeyCode::Delete,
            KeyCode::BACKSPACE => ButtonKeyCode::Backspace,
            KeyCode::RETURN => ButtonKeyCode::Return,
            KeyCode::UP => ButtonKeyCode::Up,
            KeyCode::DOWN => ButtonKeyCode::Down,
            KeyCode::PG_UP => ButtonKeyCode::PgUp,
            KeyCode::PG_DOWN => ButtonKeyCode::PgDown,
            KeyCode::ESCAPE => ButtonKeyCode::Escape,
            KeyCode::TAB => ButtonKeyCode::Tab,
            _ => return None,
        })
    }

    pub fn from_player_event(event: PlayerEvent) -> Option<Self> {
        match event {
            // ASCII characters convert directly to keyPress button events.
            PlayerEvent::TextInput { codepoint }
                if codepoint as u32 >= 32 && codepoint as u32 <= 126 =>
            {
                Some(ButtonKeyCode::from_u8(codepoint as u8).unwrap())
            }

            // Special keys have custom values for keyPress.
            PlayerEvent::KeyDown { key_code, .. } => Self::from_key_code(key_code),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        num_traits::ToPrimitive::to_u8(self).unwrap_or_default()
    }
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
