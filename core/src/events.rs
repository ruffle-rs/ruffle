use num_enum::{IntoPrimitive, TryFromPrimitive};

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum PlayerEvent {
    MouseMove { x: f64, y: f64 },
    MouseUp { x: f64, y: f64 },
    MouseDown { x: f64, y: f64 },
    MouseLeft,
}

/// The events that an AVM1 button can fire.
///
/// In Flash, these are created using `on` code on the button instance:
/// ```ignore
/// on(release) {
///     trace("Button clicked");
/// }
/// ```
#[derive(Debug)]
#[allow(dead_code)]
pub enum ButtonEvent {
    Press,
    Release,
    RollOut,
    RollOver,
    KeyPress(KeyCode),
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
    KeyPress { key_code: KeyCode },
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

/// Flash virtual keycode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum KeyCode {
    Unknown = 0,
    Backspace = 8,
    Return = 13,
    Shift = 16,
    Control = 17,
    Alt = 18,
    CapsLock = 20,
    Escape = 27,
    Space = 32,
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
    Pause = 19,
    ScrollLock = 145,
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
}
