use swf::ClipEventFlag;

#[derive(Debug)]
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
    },
    MouseDown {
        x: f64,
        y: f64,
    },
    MouseLeft,
    MouseWheel {
        delta: MouseWheelDelta,
    },
    TextInput {
        codepoint: char,
    },
}

/// The distance scrolled by the mouse wheel.
#[derive(Debug, PartialEq, Clone, Copy)]
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

/// Whether this button event was handled by some child.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClipEventResult {
    NotHandled,
    Handled,
}

/// An event type that can be handled by a movie clip
/// instance.
/// TODO: Move this representation in the swf crate?
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClipEvent {
    Construct,
    Data,
    DragOut,
    DragOver,
    EnterFrame,
    Initialize,
    KeyUp,
    KeyDown,
    KeyPress { key_code: ButtonKeyCode },
    Load,
    MouseUp,
    MouseDown,
    MouseMove,
    Press,
    RollOut,
    RollOver,
    Release,
    ReleaseOutside,
    Unload,
}

impl ClipEvent {
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
    pub const fn flag(self) -> ClipEventFlag {
        match self {
            ClipEvent::Construct => ClipEventFlag::CONSTRUCT,
            ClipEvent::Data => ClipEventFlag::DATA,
            ClipEvent::DragOut => ClipEventFlag::DRAG_OUT,
            ClipEvent::DragOver => ClipEventFlag::DRAG_OVER,
            ClipEvent::EnterFrame => ClipEventFlag::ENTER_FRAME,
            ClipEvent::Initialize => ClipEventFlag::INITIALIZE,
            ClipEvent::KeyDown => ClipEventFlag::KEY_DOWN,
            ClipEvent::KeyPress { .. } => ClipEventFlag::KEY_PRESS,
            ClipEvent::KeyUp => ClipEventFlag::KEY_UP,
            ClipEvent::Load => ClipEventFlag::LOAD,
            ClipEvent::MouseDown => ClipEventFlag::MOUSE_DOWN,
            ClipEvent::MouseMove => ClipEventFlag::MOUSE_MOVE,
            ClipEvent::MouseUp => ClipEventFlag::MOUSE_UP,
            ClipEvent::Press => ClipEventFlag::PRESS,
            ClipEvent::RollOut => ClipEventFlag::ROLL_OUT,
            ClipEvent::RollOver => ClipEventFlag::ROLL_OVER,
            ClipEvent::Release => ClipEventFlag::RELEASE,
            ClipEvent::ReleaseOutside => ClipEventFlag::RELEASE_OUTSIDE,
            ClipEvent::Unload => ClipEventFlag::UNLOAD,
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
        self.flag().contains(Self::BUTTON_EVENT_FLAGS)
    }

    /// Indicates whether this is a keyboard event type (keyUp, keyDown, keyPress).
    pub const fn is_key_event(self) -> bool {
        matches!(self, Self::KeyDown | Self::KeyUp | Self::KeyPress { .. })
    }

    /// Returns the method name of the event handler for this event.
    pub const fn method_name(self) -> Option<&'static str> {
        match self {
            ClipEvent::Construct => None,
            ClipEvent::Data => Some("onData"),
            ClipEvent::DragOut => Some("onDragOut"),
            ClipEvent::DragOver => Some("onDragOver"),
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
            ClipEvent::RollOut => Some("onRollOut"),
            ClipEvent::RollOver => Some("onRollOver"),
            ClipEvent::Release => Some("onRelease"),
            ClipEvent::ReleaseOutside => Some("onReleaseOutside"),
            ClipEvent::Unload => Some("onUnload"),
        }
    }
}

/// Flash virtual keycode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, FromPrimitive)]
pub enum KeyCode {
    Unknown = 0,
    Backspace = 8,
    Tab = 9,
    Return = 13,
    Shift = 16,
    Control = 17,
    Alt = 18,
    Pause = 19,
    CapsLock = 20,
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

/// Key codes for SWF4 keyPress button handlers. These are annoyingly different than
/// `Key.isDown` key codes.
/// TODO: After 18, these are mostly ASCII... should we just use u8? How are different
/// keyboard layouts/languages handled?
/// SWF19 pp. 198-199
#[derive(Debug, PartialEq, Eq, Copy, Clone, FromPrimitive)]
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
