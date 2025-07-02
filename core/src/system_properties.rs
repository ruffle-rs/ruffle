use crate::context::UpdateContext;
use bitflags::bitflags;
use core::fmt;
use fluent_templates::{langid, LanguageIdentifier};

/// Available cpu architectures
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

/// The available host operating systems
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

impl From<LanguageIdentifier> for Language {
    fn from(li: LanguageIdentifier) -> Self {
        match li.language.as_str() {
            "da" => Language::Danish,
            "nl" => Language::Dutch,
            "en" => Language::English,
            "fi" => Language::Finnish,
            "fr" => Language::French,
            "de" => Language::German,
            "hu" => Language::Hungarian,
            "it" => Language::Italian,
            "ja" => Language::Japanese,
            "ko" => Language::Korean,
            "no" => Language::Norwegian,
            "und" => Language::Unknown,
            "pl" => Language::Polish,
            "pt" => Language::Portuguese,
            "ru" => Language::Russian,
            "zh" => {
                if li == langid!("zh-TW") {
                    Language::TraditionalChinese
                } else {
                    Language::SimplifiedChinese
                }
            }
            "es" => Language::Spanish,
            "sv" => Language::Swedish,
            "tr" => Language::Turkish,
            // Fallback to English instead of Unknown for better compatibility.
            _ => Language::English,
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
    /// The cpu architecture of the platform
    pub cpu_architecture: CpuArchitecture,
    /// The highest supported h264 decoder level
    pub idc_level: String,
}

impl SystemProperties {
    pub fn new(language: LanguageIdentifier) -> Self {
        SystemProperties {
            //TODO: default to true on fp>=7, false <= 6
            exact_settings: true,
            //TODO: default to false on fp>=7, true <= 6
            use_codepage: false,
            capabilities: SystemCapabilities::empty(),
            player_type: PlayerType::StandAlone,
            screen_color: ScreenColor::Color,
            // TODO: note for fp <7 this should be the locale and the ui lang for >= 7, on windows
            language: language.into(),
            // source: https://web.archive.org/web/20230611050355/https://flylib.com/books/en/4.13.1.272/1/
            pixel_aspect_ratio: 1_f32,
            // source: https://tracker.adobe.com/#/view/FP-3949775
            dpi: 72_f32,
            manufacturer: Manufacturer::Linux,
            os: OperatingSystem::Linux,
            cpu_architecture: CpuArchitecture::X86,
            idc_level: "5.1".into(),
        }
    }

    pub fn get_version_string(&self, player_version: u8) -> String {
        format!(
            "{} {},0,0,0",
            self.manufacturer.get_platform_name(),
            player_version
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
                        .get_manufacturer_string(context.player_version)
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
            .append_pair("L", self.language.get_language_code(context.player_version))
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
