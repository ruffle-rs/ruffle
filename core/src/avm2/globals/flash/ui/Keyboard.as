package flash.ui {
    public final class Keyboard {
        public static native function get capsLock(): Boolean;
        public static native function get hasVirtualKeyboard(): Boolean;
        public static native function get numLock(): Boolean;
        public static native function get physicalKeyboardType(): String;
        public static native function isAccessible(): Boolean;

        public static const BACKSPACE: uint = 8;
        public static const TAB: uint = 9;
        public static const ENTER: uint = 13;
        public static const COMMAND: uint = 15;
        public static const SHIFT: uint = 16;
        public static const CONTROL: uint = 17;
        public static const ALTERNATE: uint = 18;
        public static const CAPS_LOCK: uint = 20;
        public static const NUMPAD: uint = 21;
        public static const ESCAPE: uint = 27;
        public static const SPACE: uint = 32;
        public static const PAGE_UP: uint = 33;
        public static const PAGE_DOWN: uint = 34;
        public static const END: uint = 35;
        public static const HOME: uint = 36;
        public static const LEFT: uint = 37;
        public static const UP: uint = 38;
        public static const RIGHT: uint = 39;
        public static const DOWN: uint = 40;
        public static const INSERT: uint = 45;
        public static const DELETE: uint = 46;
        public static const NUMBER_0: uint = 48;
        public static const NUMBER_1: uint = 49;
        public static const NUMBER_2: uint = 50;
        public static const NUMBER_3: uint = 51;
        public static const NUMBER_4: uint = 52;
        public static const NUMBER_5: uint = 53;
        public static const NUMBER_6: uint = 54;
        public static const NUMBER_7: uint = 55;
        public static const NUMBER_8: uint = 56;
        public static const NUMBER_9: uint = 57;
        public static const A: uint = 65;
        public static const B: uint = 66;
        public static const C: uint = 67;
        public static const D: uint = 68;
        public static const E: uint = 69;
        public static const F: uint = 70;
        public static const G: uint = 71;
        public static const H: uint = 72;
        public static const I: uint = 73;
        public static const J: uint = 74;
        public static const K: uint = 75;
        public static const L: uint = 76;
        public static const M: uint = 77;
        public static const N: uint = 78;
        public static const O: uint = 79;
        public static const P: uint = 80;
        public static const Q: uint = 81;
        public static const R: uint = 82;
        public static const S: uint = 83;
        public static const T: uint = 84;
        public static const U: uint = 85;
        public static const V: uint = 86;
        public static const W: uint = 87;
        public static const X: uint = 88;
        public static const Y: uint = 89;
        public static const Z: uint = 90;
        public static const NUMPAD_0: uint = 96;
        public static const NUMPAD_1: uint = 97;
        public static const NUMPAD_2: uint = 98;
        public static const NUMPAD_3: uint = 99;
        public static const NUMPAD_4: uint = 100;
        public static const NUMPAD_5: uint = 101;
        public static const NUMPAD_6: uint = 102;
        public static const NUMPAD_7: uint = 103;
        public static const NUMPAD_8: uint = 104;
        public static const NUMPAD_9: uint = 105;
        public static const NUMPAD_MULTIPLY: uint = 106;
        public static const NUMPAD_ADD: uint = 107;
        public static const NUMPAD_ENTER: uint = 108;
        public static const NUMPAD_SUBTRACT: uint = 109;
        public static const NUMPAD_DECIMAL: uint = 110;
        public static const NUMPAD_DIVIDE: uint = 111;
        public static const F1: uint = 112;
        public static const F2: uint = 113;
        public static const F3: uint = 114;
        public static const F4: uint = 115;
        public static const F5: uint = 116;
        public static const F6: uint = 117;
        public static const F7: uint = 118;
        public static const F8: uint = 119;
        public static const F9: uint = 120;
        public static const F10: uint = 121;
        public static const F11: uint = 122;
        public static const F12: uint = 123;
        public static const F13: uint = 124;
        public static const F14: uint = 125;
        public static const F15: uint = 126;
        public static const SEMICOLON: uint = 186;
        public static const EQUAL: uint = 187;
        public static const COMMA: uint = 188;
        public static const MINUS: uint = 189;
        public static const PERIOD: uint = 190;
        public static const SLASH: uint = 191;
        public static const BACKQUOTE: uint = 192;
        public static const LEFTBRACKET: uint = 219;
        public static const BACKSLASH: uint = 220;
        public static const RIGHTBRACKET: uint = 221;
        public static const QUOTE: uint = 222;

        [API("669")]
        public static const SETUP: uint = 0x0100001C;
        [API("669")]
        public static const NEXT: uint = 0x0100000E;
        [API("669")]
        public static const MENU: uint = 0x01000012;
        [API("669")]
        public static const CHANNEL_UP: uint = 0x01000004;
        [API("669")]
        public static const EXIT: uint = 0x01000015;
        [API("669")]
        public static const BLUE: uint = 0x01000003;
        [API("669")]
        public static const CHANNEL_DOWN: uint = 0x01000005;
        [API("669")]
        public static const INPUT: uint = 0x0100001B;
        [API("669")]
        public static const DVR: uint = 0x01000019;
        [API("669")]
        public static const SEARCH: uint = 0x0100001F;
        [API("669")]
        public static const MASTER_SHELL: uint = 0x0100001E;
        [API("669")]
        public static const SKIP_BACKWARD: uint = 0x0100000D;
        [API("669")] // [NA] This should be 719, but it's not supported at time of writing
        public static const PLAY_PAUSE: uint = 0x01000020;
        [API("669")]
        public static const HELP: uint = 0x0100001D;
        [API("669")]
        public static const VOD: uint = 0x0100001A;
        [API("669")]
        public static const LIVE: uint = 0x01000010;
        [API("669")]
        public static const RED: uint = 0x01000000;
        [API("669")]
        public static const PREVIOUS: uint = 0x0100000F;
        [API("669")]
        public static const RECORD: uint = 0x01000006;
        [API("669")]
        public static const STOP: uint = 0x01000009;
        [API("669")]
        public static const SUBTITLE: uint = 0x01000018;
        [API("669")]
        public static const PLAY: uint = 0x01000007;
        [API("669")]
        public static const GUIDE: uint = 0x01000014;
        [API("669")]
        public static const YELLOW: uint = 0x01000002;
        [API("669")]
        public static const REWIND: uint = 0x0100000B;
        [API("669")]
        public static const INFO: uint = 0x01000013;
        [API("669")]
        public static const LAST: uint = 0x01000011;
        [API("669")]
        public static const PAUSE: uint = 0x01000008;
        [API("669")]
        public static const AUDIO: uint = 0x01000017;
        [API("669")]
        public static const GREEN: uint = 0x01000001;
        [API("669")]
        public static const FAST_FORWARD: uint = 0x0100000A;
        [API("669")]
        public static const SKIP_FORWARD: uint = 0x0100000C;
        [API("669")]
        public static const BACK: uint = 0x01000016;

        public static const STRING_BEGIN: String = "\uf72a";
        public static const STRING_BREAK: String = "\uf732";
        public static const STRING_CLEARDISPLAY: String = "\uf73a";
        public static const STRING_CLEARLINE: String = "\uf739";
        public static const STRING_DELETE: String = "\uf728";
        public static const STRING_DELETECHAR: String = "\uf73e";
        public static const STRING_DELETELINE: String = "\uf73c";
        public static const STRING_DOWNARROW: String = "\uf701";
        public static const STRING_END: String = "\uf72b";
        public static const STRING_EXECUTE: String = "\uf742";
        public static const STRING_F1: String = "\uf704";
        public static const STRING_F2: String = "\uf705";
        public static const STRING_F3: String = "\uf706";
        public static const STRING_F4: String = "\uf707";
        public static const STRING_F5: String = "\uf708";
        public static const STRING_F6: String = "\uf709";
        public static const STRING_F7: String = "\uf70a";
        public static const STRING_F8: String = "\uf70b";
        public static const STRING_F9: String = "\uf70c";
        public static const STRING_F10: String = "\uf70d";
        public static const STRING_F11: String = "\uf70e";
        public static const STRING_F12: String = "\uf70f";
        public static const STRING_F13: String = "\uf710";
        public static const STRING_F14: String = "\uf711";
        public static const STRING_F15: String = "\uf712";
        public static const STRING_F16: String = "\uf713";
        public static const STRING_F17: String = "\uf714";
        public static const STRING_F18: String = "\uf715";
        public static const STRING_F19: String = "\uf716";
        public static const STRING_F20: String = "\uf717";
        public static const STRING_F21: String = "\uf718";
        public static const STRING_F22: String = "\uf719";
        public static const STRING_F23: String = "\uf71a";
        public static const STRING_F24: String = "\uf71b";
        public static const STRING_F25: String = "\uf71c";
        public static const STRING_F26: String = "\uf71d";
        public static const STRING_F27: String = "\uf71e";
        public static const STRING_F28: String = "\uf71f";
        public static const STRING_F29: String = "\uf720";
        public static const STRING_F30: String = "\uf721";
        public static const STRING_F31: String = "\uf722";
        public static const STRING_F32: String = "\uf723";
        public static const STRING_F33: String = "\uf724";
        public static const STRING_F34: String = "\uf725";
        public static const STRING_F35: String = "\uf726";
        public static const STRING_FIND: String = "\uf745";
        public static const STRING_HELP: String = "\uf746";
        public static const STRING_HOME: String = "\uf729";
        public static const STRING_INSERT: String = "\uf727";
        public static const STRING_INSERTCHAR: String = "\uf73d";
        public static const STRING_INSERTLINE: String = "\uf73b";
        public static const STRING_LEFTARROW: String = "\uf702";
        public static const STRING_MENU: String = "\uf735";
        public static const STRING_MODESWITCH: String = "\uf747";
        public static const STRING_NEXT: String = "\uf740";
        public static const STRING_PAGEDOWN: String = "\uf72d";
        public static const STRING_PAGEUP: String = "\uf72c";
        public static const STRING_PAUSE: String = "\uf730";
        public static const STRING_PREV: String = "\uf73f";
        public static const STRING_PRINT: String = "\uf738";
        public static const STRING_PRINTSCREEN: String = "\uf72e";
        public static const STRING_REDO: String = "\uf744";
        public static const STRING_RESET: String = "\uf733";
        public static const STRING_RIGHTARROW: String = "\uf703";
        public static const STRING_SCROLLLOCK: String = "\uf72f";
        public static const STRING_SELECT: String = "\uf741";
        public static const STRING_STOP: String = "\uf734";
        public static const STRING_SYSREQ: String = "\uf731";
        public static const STRING_SYSTEM: String = "\uf737";
        public static const STRING_UNDO: String = "\uf743";
        public static const STRING_UPARROW: String = "\uf700";
        public static const STRING_USER: String = "\uf736";

        public static const KEYNAME_UPARROW: String = "Up";
        public static const KEYNAME_DOWNARROW: String = "Down";
        public static const KEYNAME_LEFTARROW: String = "Left";
        public static const KEYNAME_RIGHTARROW: String = "Right";
        public static const KEYNAME_F1: String = "F1";
        public static const KEYNAME_F2: String = "F2";
        public static const KEYNAME_F3: String = "F3";
        public static const KEYNAME_F4: String = "F4";
        public static const KEYNAME_F5: String = "F5";
        public static const KEYNAME_F6: String = "F6";
        public static const KEYNAME_F7: String = "F7";
        public static const KEYNAME_F8: String = "F8";
        public static const KEYNAME_F9: String = "F9";
        public static const KEYNAME_F10: String = "F10";
        public static const KEYNAME_F11: String = "F11";
        public static const KEYNAME_F12: String = "F12";
        public static const KEYNAME_F13: String = "F13";
        public static const KEYNAME_F14: String = "F14";
        public static const KEYNAME_F15: String = "F15";
        public static const KEYNAME_F16: String = "F16";
        public static const KEYNAME_F17: String = "F17";
        public static const KEYNAME_F18: String = "F18";
        public static const KEYNAME_F19: String = "F19";
        public static const KEYNAME_F20: String = "F20";
        public static const KEYNAME_F21: String = "F21";
        public static const KEYNAME_F22: String = "F22";
        public static const KEYNAME_F23: String = "F23";
        public static const KEYNAME_F24: String = "F24";
        public static const KEYNAME_F25: String = "F25";
        public static const KEYNAME_F26: String = "F26";
        public static const KEYNAME_F27: String = "F27";
        public static const KEYNAME_F28: String = "F28";
        public static const KEYNAME_F29: String = "F29";
        public static const KEYNAME_F30: String = "F30";
        public static const KEYNAME_F31: String = "F31";
        public static const KEYNAME_F32: String = "F32";
        public static const KEYNAME_F33: String = "F33";
        public static const KEYNAME_F34: String = "F34";
        public static const KEYNAME_F35: String = "F35";
        public static const KEYNAME_INSERT: String = "Insert";
        public static const KEYNAME_DELETE: String = "Delete";
        public static const KEYNAME_HOME: String = "Home";
        public static const KEYNAME_BEGIN: String = "Begin";
        public static const KEYNAME_END: String = "End";
        public static const KEYNAME_PAGEUP: String = "PgUp";
        public static const KEYNAME_PAGEDOWN: String = "PgDn";
        public static const KEYNAME_PRINTSCREEN: String = "PrntScrn";
        public static const KEYNAME_SCROLLLOCK: String = "ScrlLck";
        public static const KEYNAME_PAUSE: String = "Pause";
        public static const KEYNAME_SYSREQ: String = "SysReq";
        public static const KEYNAME_BREAK: String = "Break";
        public static const KEYNAME_RESET: String = "Reset";
        public static const KEYNAME_STOP: String = "Stop";
        public static const KEYNAME_MENU: String = "Menu";
        public static const KEYNAME_USER: String = "User";
        public static const KEYNAME_SYSTEM: String = "Sys";
        public static const KEYNAME_PRINT: String = "Print";
        public static const KEYNAME_CLEARLINE: String = "ClrLn";
        public static const KEYNAME_CLEARDISPLAY: String = "ClrDsp";
        public static const KEYNAME_INSERTLINE: String = "InsLn";
        public static const KEYNAME_DELETELINE: String = "DelLn";
        public static const KEYNAME_INSERTCHAR: String = "InsChr";
        public static const KEYNAME_DELETECHAR: String = "DelChr";
        public static const KEYNAME_PREV: String = "Prev";
        public static const KEYNAME_NEXT: String = "Next";
        public static const KEYNAME_SELECT: String = "Select";
        public static const KEYNAME_EXECUTE: String = "Exec";
        public static const KEYNAME_UNDO: String = "Undo";
        public static const KEYNAME_REDO: String = "Redo";
        public static const KEYNAME_FIND: String = "Find";
        public static const KEYNAME_HELP: String = "Help";
        public static const KEYNAME_MODESWITCH: String = "ModeSw";
        public static const KEYNAME_PLAYPAUSE: String = "PlayPause";

        public static const CharCodeStrings: Array = [
            KEYNAME_UPARROW,
            KEYNAME_DOWNARROW,
            KEYNAME_LEFTARROW,
            KEYNAME_RIGHTARROW,
            KEYNAME_F1,
            KEYNAME_F2,
            KEYNAME_F3,
            KEYNAME_F4,
            KEYNAME_F5,
            KEYNAME_F6,
            KEYNAME_F7,
            KEYNAME_F8,
            KEYNAME_F9,
            KEYNAME_F10,
            KEYNAME_F11,
            KEYNAME_F12,
            KEYNAME_F13,
            KEYNAME_F14,
            KEYNAME_F15,
            KEYNAME_F16,
            KEYNAME_F17,
            KEYNAME_F18,
            KEYNAME_F19,
            KEYNAME_F20,
            KEYNAME_F21,
            KEYNAME_F22,
            KEYNAME_F23,
            KEYNAME_F24,
            KEYNAME_F25,
            KEYNAME_F26,
            KEYNAME_F27,
            KEYNAME_F28,
            KEYNAME_F29,
            KEYNAME_F30,
            KEYNAME_F31,
            KEYNAME_F32,
            KEYNAME_F33,
            KEYNAME_F34,
            KEYNAME_F35,
            KEYNAME_INSERT,
            KEYNAME_DELETE,
            KEYNAME_HOME,
            KEYNAME_BEGIN,
            KEYNAME_END,
            KEYNAME_PAGEUP,
            KEYNAME_PAGEDOWN,
            KEYNAME_PRINTSCREEN,
            KEYNAME_SCROLLLOCK,
            KEYNAME_PAUSE,
            KEYNAME_SYSREQ,
            KEYNAME_BREAK,
            KEYNAME_RESET,
            KEYNAME_STOP,
            KEYNAME_MENU,
            KEYNAME_USER,
            KEYNAME_SYSTEM,
            KEYNAME_PRINT,
            KEYNAME_CLEARLINE,
            KEYNAME_CLEARDISPLAY,
            KEYNAME_INSERTLINE,
            KEYNAME_DELETELINE,
            KEYNAME_INSERTCHAR,
            KEYNAME_DELETECHAR,
            KEYNAME_PREV,
            KEYNAME_NEXT,
            KEYNAME_SELECT,
            KEYNAME_EXECUTE,
            KEYNAME_UNDO,
            KEYNAME_REDO,
            KEYNAME_FIND,
            KEYNAME_HELP,
            KEYNAME_MODESWITCH,
            KEYNAME_PLAYPAUSE,
        ];
    }
}
