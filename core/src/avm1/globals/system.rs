use crate::avm1::object::Object;
use crate::avm1::property::Attribute::{DontDelete, DontEnum, ReadOnly};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use gc_arena::MutationContext;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use crate::avm1::function::Executable;
use enumset::{EnumSet, EnumSetType};

/// Available type of sandbox for a given SWF
pub enum SandboxType {
    Remote,
    LocalWithFile,
    LocalWithNetwork,
    LocalTrusted
}

impl SandboxType {
    pub fn get_sandbox_name(&self) -> &str {
        match self {
            SandboxType::Remote => "remote",
            SandboxType::LocalWithFile => "localWithFile",
            SandboxType::LocalWithNetwork => "localWithNetwork",
            SandboxType::LocalTrusted => "localTrusted",
        }
    }
}

/// The available host operating systems
pub enum OperatingSystem {
    WindowsXp,
    Windows2k,
    WindowsNt,
    Windows98,
    Windows95,
    WindowsCE,
    Linux,
    MacOS
}

impl OperatingSystem {
    pub fn get_os_name(&self) -> &str {
        match self {
            OperatingSystem::WindowsXp => "Windows XP",
            OperatingSystem::Windows2k => "Windows 2000",
            OperatingSystem::WindowsNt => "Windows NT",
            OperatingSystem::Windows98 => "Windows 98/ME",
            OperatingSystem::Windows95 => "Windows 95",
            OperatingSystem::WindowsCE => "Windows CE",
            OperatingSystem::Linux => "Linux",
            OperatingSystem::MacOS => "MacOS",
        }
    }
}

/// The available player manufacturers
pub enum Manufacturer {
    Windows,
    Macintosh,
    Linux,
    Other(String),
}

impl Manufacturer {
    pub fn get_manufacturer_string(&self) -> String {
        let os_part = match self {
            Manufacturer::Windows => "Windows",
            Manufacturer::Macintosh => "Macintosh",
            Manufacturer::Linux => "Linux",
            Manufacturer::Other(name) => name.as_str(),
        };

        //TODO: this should be adobe in (what version?)
        format!("Macromedia {}", os_part)
    }

    pub fn get_platform_name(&self) -> &str {
        match self {
            Manufacturer::Windows => "WIN",
            Manufacturer::Macintosh => "MAC",
            Manufacturer::Linux => "LNX",
            _ => ""
        }
    }
}

/// The language of the host os
pub enum Language {
    Czech,
    Danish,
    Dutch,
    English,
    Finnish,
    French,
    German,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Norwegian,
    Unknown,
    Polish,
    Portuguese,
    Russian,
    SimplifiedChinese,
    Spanish,
    Swedish,
    TraditionalChinese,
    Turkish
}

impl Language {
    pub fn get_language_code(&self) -> &str {
        return match self {
            Language::Czech => "cs",
            Language::Danish => "da",
            Language::Dutch => "nl",
            Language::English => {
                // TODO: return "en-US" for player_version < 7
                "en"
            }
            Language::Finnish => "fi",
            Language::French => "fr",
            Language::German => "de",
            Language::Hungarian => "hu",
            Language::Italian => "it",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Norwegian => "no",
            Language::Unknown => "xu",
            Language::Polish => "pl",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::SimplifiedChinese => "zh-CN",
            Language::Spanish => "es",
            Language::Swedish => "sv",
            Language::TraditionalChinese => "zh-TW",
            Language::Turkish => "tr"
        }
    }
}

/// The supported colors of the screen
pub enum ScreenColor {
    Color,
    Gray,
    BlackWhite,
}

impl ScreenColor {
    pub fn get_color_code(&self) -> &str {
        return match self {
            ScreenColor::Color => "color",
            ScreenColor::Gray => "gray",
            ScreenColor::BlackWhite => "bw",
        }
    }
}

/// The type of the player
pub enum PlayerType {
    StandAlone,
    External,
    PlugIn,
    ActiveX,
}

impl PlayerType {
    pub fn get_player_name(&self) -> &str {
        return match self {
            PlayerType::StandAlone => "StandAlone",
            PlayerType::External => "External",
            PlayerType::PlugIn => "PlugIn",
            PlayerType::ActiveX => "ActiveX",
        }
    }
}

#[derive(Debug, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
enum SettingsPanel {
    Privacy = 0,
    LocalStorage = 1,
    Microphone = 2,
    Camera = 3,
}

#[derive(EnumSetType, Debug)]
pub enum SystemCapabilities {
    AvHardware,
    Accessibility,
    Audio,
    AudioEncoder,
    EmbeddedVideo,
    IME,
    MP3,
    Printing,
    ScreenBroadcast,
    ScreenPlayback,
    StreamingAudio,
    StreamingVideo,
    VideoEncoder,
    Debugger,
    LocalFileRead,
}

/// The properties modified by 'System'
pub struct SystemProperties {
    /// If true then settings should be saved and read from the exact same domain of the player
    /// If false then they should be saved to the super domain
    pub exact_settings: bool,
    /// If true then the system codepage should be used instead of unicode for text files
    /// If false then unicode should be used
    pub use_codepage: bool,
    /// The capabilities of the player
    pub capabilities: EnumSet<SystemCapabilities>,
    /// The type of the player
    pub player_type: PlayerType,
    /// The type of screen available to the player
    pub screen_color: ScreenColor,
    /// The language of the host os
    pub language: Language,
    /// The resolution of the available screen
    pub screen_resolution: (u32, u32),
    /// The aspect ratio of the screens pixels
    pub aspect_ratio: f32,
    /// The dpi of the screen
    pub dpi: f32,
    /// The manufacturer of the player
    pub manufacturer: Manufacturer,
    /// The os of the host
    pub os: OperatingSystem,
    /// The type of the player sandbox
    pub sandbox_type: SandboxType,
}

impl SystemProperties {
    pub fn has_capability(&self, cap: SystemCapabilities) -> bool {
        self.capabilities.contains(cap)
    }

    pub fn not_has_capability(&self, cap: SystemCapabilities) -> bool {
        !self.capabilities.contains(cap)
    }

    fn encode_bool(&self, b: bool) -> &str {
        match b {
            true => "t",
            false => "f"
        }
    }

    pub fn get_server_string(&self) -> String {
        //TODO: url encode string params
        //TODO: check the order, this should match flash

        format!("AVD={}&ACC={}&A={}&AE={}&EV={}&IME={}&MP3={}&PR={}&SB={}&SP={}&SA={}&SV={}&VE={}&DEB={}&LFD={}&M={}&OS={}&AR={}&PT={}&COL={}&DP={}&R={}x{}",
            self.encode_bool(self.not_has_capability(SystemCapabilities::AvHardware)),
            self.encode_bool(self.has_capability(SystemCapabilities::Accessibility)),
            self.encode_bool(self.has_capability(SystemCapabilities::Audio)),
            self.encode_bool(self.has_capability(SystemCapabilities::AudioEncoder)),
            self.encode_bool(self.has_capability(SystemCapabilities::EmbeddedVideo)),
            self.encode_bool(self.has_capability(SystemCapabilities::IME)),
            self.encode_bool(self.has_capability(SystemCapabilities::MP3)),
            self.encode_bool(self.has_capability(SystemCapabilities::Printing)),
            self.encode_bool(self.has_capability(SystemCapabilities::ScreenBroadcast)),
            self.encode_bool(self.has_capability(SystemCapabilities::ScreenPlayback)),
            self.encode_bool(self.has_capability(SystemCapabilities::StreamingAudio)),
            self.encode_bool(self.has_capability(SystemCapabilities::StreamingVideo)),
            self.encode_bool(self.has_capability(SystemCapabilities::VideoEncoder)),
            self.encode_bool(self.has_capability(SystemCapabilities::Debugger)),
            self.encode_bool(self.not_has_capability(SystemCapabilities::LocalFileRead)),
            self.manufacturer.get_manufacturer_string(),
            self.os.get_os_name(),
            self.aspect_ratio,
            self.player_type.get_player_name(),
            self.screen_color.get_color_code(),
            self.dpi,
            self.screen_resolution.0,
            self.screen_resolution.1
        )
    }
}

impl Default for SystemProperties {
    fn default() -> Self {
        SystemProperties {
            //TODO: default to true on fp>=7, false <= 6
            exact_settings: true,
            //TODO: default to false on fp>=7, true <= 6
            use_codepage: false,
            capabilities: EnumSet::empty(),
            player_type: PlayerType::StandAlone,
            screen_color: ScreenColor::Color,
            // TODO: note for fp <7 this should be the locale and the ui lang for >= 7, on windows
            language: Language::English,
            screen_resolution: (0, 0),
            aspect_ratio: 1 as f32,
            dpi: 1 as f32,
            //TODO: default to current
            manufacturer: Manufacturer::Linux,
            os: OperatingSystem::Linux,
            sandbox_type: SandboxType::LocalTrusted,
        }
    }
}

pub fn set_clipboard<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_content = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_string(avm, action_context)?;

    action_context.input.set_clipboard_content(new_content);

    Ok(Value::Undefined.into())
}

pub fn show_settings<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    //TODO: should default to the last panel displayed
    let last_panel = SettingsPanel::Privacy;

    let panel = args
        .get(0)
        .map(|v| match v {
            Value::Number(x) => SettingsPanel::try_from(*x as u8).unwrap_or(last_panel),
            _ => last_panel,
        })
        .unwrap_or(last_panel);

    log::warn!("System.showSettings({:?}) not not implemented", panel);
    Ok(Value::Undefined.into())
}

pub fn set_use_code_page<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(avm.current_swf_version());

    action_context.system.use_codepage = value;

    Ok(Value::Undefined.into())
}

pub fn get_use_code_page<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(action_context.system.use_codepage.into())
}

pub fn set_exact_settings<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(avm.current_swf_version());

    action_context.system.exact_settings = value;

    Ok(Value::Undefined.into())
}

pub fn get_exact_settings<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(action_context.system.exact_settings.into())
}

pub fn on_status<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("System.onStatus() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut system = ScriptObject::object(gc_context, proto);

    system.add_property(
        gc_context,
        "exactSettings",
        Executable::Native(get_exact_settings),
        Some(Executable::Native(set_exact_settings)),
        DontDelete | DontEnum,
    );


    system.add_property(
        gc_context,
        "useCodepage",
        Executable::Native(get_use_code_page),
        Some(Executable::Native(set_use_code_page)),
        DontDelete | DontEnum,
    );

    system.define_value(
        gc_context,
        "security",
        crate::avm1::globals::system_security::create(gc_context, proto, fn_proto).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    system.define_value(
        gc_context,
        "capabilities",
        crate::avm1::globals::system_capabilities::create(gc_context, proto).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    system.define_value(
        gc_context,
        "IME",
        crate::avm1::globals::system_ime::create(gc_context, proto, fn_proto).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    system.force_set_function(
        "setClipboard",
        set_clipboard,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    system.force_set_function(
        "showSettings",
        show_settings,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    // Pretty sure this is a variable
    system.force_set_function(
        "onStatus",
        on_status,
        gc_context,
        DontDelete | DontEnum,
        fn_proto,
    );

    system.into()
}
