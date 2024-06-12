use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::runtime::Avm1;
use crate::avm1::{ScriptObject, TObject, Value};
use crate::avm1_stub;
use crate::context::{GcContext, UpdateContext};
use bitflags::bitflags;
use core::fmt;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "exactSettings" => property(get_exact_settings, set_exact_settings);
    "useCodepage" => property(get_use_code_page, set_use_code_page);
    "setClipboard" => method(set_clipboard);
    "showSettings" => method(show_settings);
    // Pretty sure this is a variable
    "onStatus" => method(on_status);
};

/// Available cpu architectures
#[allow(dead_code)]
pub enum CpuArchitecture {
    PowerPc,
    X86,
    Sparc,
    Arm,
}

impl fmt::Display for CpuArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CpuArchitecture::PowerPc => "PowerPC",
            CpuArchitecture::X86 => "x86",
            CpuArchitecture::Sparc => "SPARC",
            CpuArchitecture::Arm => "ARM",
        })
    }
}

/// Available type of sandbox for a given SWF
#[allow(dead_code)]
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
#[allow(dead_code)]
pub enum OperatingSystem {
    WindowsXp,
    Windows2k,
    WindowsNt,
    Windows98,
    Windows95,
    WindowsCe,
    WindowsUnknown,
    Linux,
    MacOs,
}

impl fmt::Display for OperatingSystem {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(match self {
            OperatingSystem::WindowsXp => "Windows XP",
            OperatingSystem::Windows2k => "Windows 2000",
            OperatingSystem::WindowsNt => "Windows NT",
            OperatingSystem::Windows98 => "Windows 98/ME",
            OperatingSystem::Windows95 => "Windows 95",
            OperatingSystem::WindowsCe => "Windows CE",
            OperatingSystem::WindowsUnknown => "Windows",
            OperatingSystem::Linux => "Linux",
            OperatingSystem::MacOs => "MacOS",
        })
    }
}

/// The available player manufacturers
#[allow(dead_code)]
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
            format!("Macromedia {os_part}")
        } else {
            format!("Adobe {os_part}")
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[derive(Debug, Copy, Clone, FromPrimitive)]
enum SettingsPanel {
    Privacy = 0,
    LocalStorage = 1,
    Microphone = 2,
    Camera = 3,
}

impl SettingsPanel {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }
}

bitflags! {
    pub struct SystemCapabilities: u32 {
        const AV_HARDWARE      = 1 << 0;
        const ACCESSIBILITY    = 1 << 1;
        const AUDIO            = 1 << 2;
        const AUDIO_ENCODER    = 1 << 3;
        const EMBEDDED_VIDEO   = 1 << 4;
        const IME              = 1 << 5;
        const MP3              = 1 << 6;
        const PRINTING         = 1 << 7;
        const SCREEN_BROADCAST = 1 << 8;
        const SCREEN_PLAYBACK  = 1 << 9;
        const STREAMING_AUDIO  = 1 << 10;
        const STREAMING_VIDEO  = 1 << 11;
        const VIDEO_ENCODER    = 1 << 12;
        const DEBUGGER         = 1 << 13;
        const LOCAL_FILE_READ  = 1 << 14;
        const PROCESS_64_BIT   = 1 << 15;
        const PROCESS_32_BIT   = 1 << 16;
        const ACROBAT_EMBEDDED = 1 << 17;
        const TLS              = 1 << 18;
        const WINDOW_LESS      = 1 << 19;
    }
}

/// The properties modified by 'System'
pub struct SystemProperties {
    /// If true then settings should be saved and read from the exact same domain of the player
    /// If false then they should be saved to the super domain
    pub exact_settings: bool,
    /// If true, the system codepage should be used for text files
    /// If false, UTF-8 should be used for SWF version >= 6 and ISO Latin-1 for SWF version <= 5
    pub use_codepage: bool,
    /// The capabilities of the player
    pub capabilities: SystemCapabilities,
    /// The type of the player
    pub player_type: PlayerType,
    /// The type of screen available to the player
    pub screen_color: ScreenColor,
    /// The aspect ratio of the screens pixels
    pub pixel_aspect_ratio: f32,
    /// The dpi of the screen
    pub dpi: f32,
    /// The language of the host os
    pub language: Language,
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
    pub fn new(sandbox_type: SandboxType) -> Self {
        SystemProperties {
            //TODO: default to true on fp>=7, false <= 6
            exact_settings: true,
            //TODO: default to false on fp>=7, true <= 6
            use_codepage: false,
            capabilities: SystemCapabilities::empty(),
            player_type: PlayerType::StandAlone,
            screen_color: ScreenColor::Color,
            // TODO: note for fp <7 this should be the locale and the ui lang for >= 7, on windows
            language: Language::English,
            // source: https://web.archive.org/web/20230611050355/https://flylib.com/books/en/4.13.1.272/1/
            pixel_aspect_ratio: 1_f32,
            // source: https://tracker.adobe.com/#/view/FP-3949775
            dpi: 72_f32,
            manufacturer: Manufacturer::Linux,
            os: OperatingSystem::Linux,
            sandbox_type,
            cpu_architecture: CpuArchitecture::X86,
            idc_level: "5.1".into(),
        }
    }
    pub fn get_version_string(&self, avm: &mut Avm1) -> String {
        format!(
            "{} {},0,0,0",
            self.manufacturer.get_platform_name(),
            avm.player_version()
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

    pub fn get_server_string(&self, context: &UpdateContext) -> String {
        let viewport_dimensions = context.renderer.viewport_dimensions();
        url::form_urlencoded::Serializer::new(String::new())
            .append_pair("A", self.encode_capability(SystemCapabilities::AUDIO))
            .append_pair(
                "SA",
                self.encode_capability(SystemCapabilities::STREAMING_AUDIO),
            )
            .append_pair(
                "SV",
                self.encode_capability(SystemCapabilities::STREAMING_VIDEO),
            )
            .append_pair(
                "EV",
                self.encode_capability(SystemCapabilities::EMBEDDED_VIDEO),
            )
            .append_pair("MP3", self.encode_capability(SystemCapabilities::MP3))
            .append_pair(
                "AE",
                self.encode_capability(SystemCapabilities::AUDIO_ENCODER),
            )
            .append_pair(
                "VE",
                self.encode_capability(SystemCapabilities::VIDEO_ENCODER),
            )
            .append_pair(
                "ACC",
                self.encode_not_capability(SystemCapabilities::ACCESSIBILITY),
            )
            .append_pair("PR", self.encode_capability(SystemCapabilities::PRINTING))
            .append_pair(
                "SP",
                self.encode_capability(SystemCapabilities::SCREEN_PLAYBACK),
            )
            .append_pair(
                "SB",
                self.encode_capability(SystemCapabilities::SCREEN_BROADCAST),
            )
            .append_pair("DEB", self.encode_capability(SystemCapabilities::DEBUGGER))
            .append_pair(
                "M",
                &self.encode_string(
                    self.manufacturer
                        .get_manufacturer_string(context.avm1.player_version())
                        .as_str(),
                ),
            )
            .append_pair(
                "R",
                &format!(
                    "{}x{}",
                    viewport_dimensions.width, viewport_dimensions.height
                ),
            )
            .append_pair("COL", &self.screen_color.to_string())
            .append_pair("AR", &self.pixel_aspect_ratio.to_string())
            .append_pair("OS", &self.encode_string(&self.os.to_string()))
            .append_pair(
                "L",
                self.language
                    .get_language_code(context.avm1.player_version()),
            )
            .append_pair("IME", self.encode_capability(SystemCapabilities::IME))
            .append_pair("PT", &self.player_type.to_string())
            .append_pair(
                "AVD",
                self.encode_not_capability(SystemCapabilities::AV_HARDWARE),
            )
            .append_pair(
                "LFD",
                self.encode_not_capability(SystemCapabilities::LOCAL_FILE_READ),
            )
            .append_pair("DP", &self.dpi.to_string())
            .finish()
    }
}

pub fn set_clipboard<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_content = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?
        .to_string();

    activation.context.ui.set_clipboard_content(new_content);

    Ok(Value::Undefined)
}

pub fn show_settings<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    //TODO: should default to the last panel displayed
    let last_panel_pos = 0;

    let panel_pos = args
        .get(0)
        .unwrap_or(&last_panel_pos.into())
        .coerce_to_i32(activation)?;

    let _panel = SettingsPanel::from_u8(panel_pos as u8).unwrap_or(SettingsPanel::Privacy);

    avm1_stub!(activation, "System", "showSettings");
    Ok(Value::Undefined)
}

pub fn set_use_code_page<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(activation.swf_version());

    activation.context.system.use_codepage = value;

    Ok(Value::Undefined)
}

pub fn get_use_code_page<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.use_codepage.into())
}

pub fn set_exact_settings<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_bool(activation.swf_version());

    activation.context.system.exact_settings = value;

    Ok(Value::Undefined)
}

pub fn get_exact_settings<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.system.exact_settings.into())
}

pub fn on_status<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System", "onStatus");
    Ok(Value::Undefined)
}

pub fn create<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    security: Object<'gc>,
    capabilities: Object<'gc>,
    ime: Object<'gc>,
) -> Object<'gc> {
    let gc_context = context.gc_context;
    let system = ScriptObject::new(gc_context, Some(proto));
    define_properties_on(OBJECT_DECLS, context, system, fn_proto);
    system.define_value(gc_context, "IME", ime.into(), Attribute::empty());
    system.define_value(gc_context, "security", security.into(), Attribute::empty());
    system.define_value(
        gc_context,
        "capabilities",
        capabilities.into(),
        Attribute::empty(),
    );

    system.into()
}
