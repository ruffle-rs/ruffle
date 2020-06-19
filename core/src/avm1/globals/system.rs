use crate::avm1::function::Executable;
use crate::avm1::object::Object;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use core::fmt;
use enumset::{EnumSet, EnumSetType};
use gc_arena::MutationContext;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

/// Available cpu architectures
pub enum CpuArchitecture {
    PowerPC,
    X86,
    SPARC,
    ARM,
}

impl fmt::Display for CpuArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CpuArchitecture::PowerPC => "PowerPC",
            CpuArchitecture::X86 => "x86",
            CpuArchitecture::SPARC => "SPARC",
            CpuArchitecture::ARM => "ARM",
        })
    }
}

/// Available type of sandbox for a given SWF
pub enum SandboxType {
    Remote,
    LocalWithFile,
    LocalWithNetwork,
    LocalTrusted,
}

impl fmt::Display for SandboxType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self {
            SandboxType::Remote => "remote",
            SandboxType::LocalWithFile => "localWithFile",
            SandboxType::LocalWithNetwork => "localWithNetwork",
            SandboxType::LocalTrusted => "localTrusted",
        })
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
    WindowsUnknown,
    Linux,
    MacOS,
}

impl fmt::Display for OperatingSystem {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self {
            OperatingSystem::WindowsXp => "Windows XP",
            OperatingSystem::Windows2k => "Windows 2000",
            OperatingSystem::WindowsNt => "Windows NT",
            OperatingSystem::Windows98 => "Windows 98/ME",
            OperatingSystem::Windows95 => "Windows 95",
            OperatingSystem::WindowsCE => "Windows CE",
            OperatingSystem::WindowsUnknown => "Windows",
            OperatingSystem::Linux => "Linux",
            OperatingSystem::MacOS => "MacOS",
        })
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
    pub fn get_manufacturer_string(&self, version: u8) -> String {
        let os_part = match self {
            Manufacturer::Windows => "Windows",
            Manufacturer::Macintosh => "Macintosh",
            Manufacturer::Linux => "Linux",
            Manufacturer::Other(name) => name.as_str(),
        };

        if version <= 8 {
            format!("Macromedia {}", os_part)
        } else {
            format!("Adobe {}", os_part)
        }
    }

    pub fn get_platform_name(&self) -> &str {
        match self {
            Manufacturer::Windows => "WIN",
            Manufacturer::Macintosh => "MAC",
            Manufacturer::Linux => "LNX",
            _ => "",
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
    Turkish,
}

impl Language {
    pub fn get_language_code(&self, player_version: u8) -> &str {
        match self {
            Language::Czech => "cs",
            Language::Danish => "da",
            Language::Dutch => "nl",
            Language::English => {
                if player_version < 7 {
                    "en-US"
                } else {
                    "en"
                }
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
            Language::Turkish => "tr",
        }
    }
}

/// The supported colors of the screen
pub enum ScreenColor {
    Color,
    Gray,
    BlackWhite,
}

impl fmt::Display for ScreenColor {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self {
            ScreenColor::Color => "color",
            ScreenColor::Gray => "gray",
            ScreenColor::BlackWhite => "bw",
        })
    }
}
/// The type of the player
pub enum PlayerType {
    StandAlone,
    External,
    PlugIn,
    ActiveX,
}

impl fmt::Display for PlayerType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self {
            PlayerType::StandAlone => "StandAlone",
            PlayerType::External => "External",
            PlayerType::PlugIn => "PlugIn",
            PlayerType::ActiveX => "ActiveX",
        })
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
    Process64Bit,
    Process32Bit,
    AcrobatEmbedded,
    TLS,
    WindowLess,
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
    /// The cpu architecture of the platform
    pub cpu_architecture: CpuArchitecture,
    /// The highest supported h264 decoder level
    pub idc_level: String,
}

impl SystemProperties {
    pub fn get_version_string(&self, avm: &Avm1) -> String {
        format!(
            "{} {},0,0,0",
            self.manufacturer.get_platform_name(),
            avm.player_version
        )
    }

    pub fn has_capability(&self, cap: SystemCapabilities) -> bool {
        self.capabilities.contains(cap)
    }

    fn encode_capability(&self, cap: SystemCapabilities) -> &str {
        if self.has_capability(cap) {
            "t"
        } else {
            "f"
        }
    }

    fn encode_not_capability(&self, cap: SystemCapabilities) -> &str {
        if self.has_capability(cap) {
            "f"
        } else {
            "t"
        }
    }

    fn encode_string(&self, s: &str) -> String {
        percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
    }

    pub fn get_server_string(&self, avm: &Avm1) -> String {
        url::form_urlencoded::Serializer::new(String::new())
            .append_pair("A", self.encode_capability(SystemCapabilities::Audio))
            .append_pair(
                "SA",
                self.encode_capability(SystemCapabilities::StreamingAudio),
            )
            .append_pair(
                "SV",
                self.encode_capability(SystemCapabilities::StreamingVideo),
            )
            .append_pair(
                "EV",
                self.encode_capability(SystemCapabilities::EmbeddedVideo),
            )
            .append_pair("MP3", self.encode_capability(SystemCapabilities::MP3))
            .append_pair(
                "AE",
                self.encode_capability(SystemCapabilities::AudioEncoder),
            )
            .append_pair(
                "VE",
                self.encode_capability(SystemCapabilities::VideoEncoder),
            )
            .append_pair(
                "ACC",
                self.encode_not_capability(SystemCapabilities::Accessibility),
            )
            .append_pair("PR", self.encode_capability(SystemCapabilities::Printing))
            .append_pair(
                "SP",
                self.encode_capability(SystemCapabilities::ScreenPlayback),
            )
            .append_pair(
                "SB",
                self.encode_capability(SystemCapabilities::ScreenBroadcast),
            )
            .append_pair("DEB", self.encode_capability(SystemCapabilities::Debugger))
            .append_pair(
                "M",
                &self.encode_string(
                    self.manufacturer
                        .get_manufacturer_string(avm.player_version)
                        .as_str(),
                ),
            )
            .append_pair(
                "R",
                &format!("{}x{}", self.screen_resolution.0, self.screen_resolution.1),
            )
            .append_pair("COL", &self.screen_color.to_string())
            .append_pair("AR", &self.aspect_ratio.to_string())
            .append_pair("OS", &self.encode_string(&self.os.to_string()))
            .append_pair("L", self.language.get_language_code(avm.player_version))
            .append_pair("IME", self.encode_capability(SystemCapabilities::IME))
            .append_pair("PT", &self.player_type.to_string())
            .append_pair(
                "AVD",
                self.encode_not_capability(SystemCapabilities::AvHardware),
            )
            .append_pair(
                "LFD",
                self.encode_not_capability(SystemCapabilities::LocalFileRead),
            )
            .append_pair("DP", &self.dpi.to_string())
            .finish()
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
            aspect_ratio: 1_f32,
            dpi: 1_f32,
            manufacturer: Manufacturer::Linux,
            os: OperatingSystem::Linux,
            sandbox_type: SandboxType::LocalTrusted,
            cpu_architecture: CpuArchitecture::X86,
            idc_level: "5.1".into(),
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
        .coerce_to_string(avm, action_context)?
        .to_string();

    action_context.input.set_clipboard_content(new_content);

    Ok(Value::Undefined.into())
}

pub fn show_settings<'gc>(
    avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    //TODO: should default to the last panel displayed
    let last_panel_pos = 0;

    let panel_pos = args
        .get(0)
        .unwrap_or(&Value::Number(last_panel_pos as f64))
        .coerce_to_i32(avm, action_context)?;

    let panel = SettingsPanel::try_from(panel_pos as u8).unwrap_or(SettingsPanel::Privacy);

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
    security: Object<'gc>,
    capabilities: Object<'gc>,
    ime: Object<'gc>,
) -> Object<'gc> {
    let mut system = ScriptObject::object(gc_context, proto);

    system.add_property(
        gc_context,
        "exactSettings",
        Executable::Native(get_exact_settings),
        Some(Executable::Native(set_exact_settings)),
        EnumSet::empty(),
    );

    system.add_property(
        gc_context,
        "useCodepage",
        Executable::Native(get_use_code_page),
        Some(Executable::Native(set_use_code_page)),
        EnumSet::empty(),
    );

    system.define_value(gc_context, "security", security.into(), EnumSet::empty());

    system.define_value(
        gc_context,
        "capabilities",
        capabilities.into(),
        EnumSet::empty(),
    );

    system.define_value(gc_context, "IME", ime.into(), EnumSet::empty());

    system.force_set_function(
        "setClipboard",
        set_clipboard,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    system.force_set_function(
        "showSettings",
        show_settings,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    // Pretty sure this is a variable
    system.force_set_function(
        "onStatus",
        on_status,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    system.into()
}
