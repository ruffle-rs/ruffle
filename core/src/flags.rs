//! Compatibility flags help configure Ruffle by enabling and disabling specific behaviors.
//!
//! They are meant to be used for instance in the following situations.
//!
//! 1. They fix bugs in Flash Player that make some content misbehave.
//!    Note that in general we don't fix bugs in Flash Player -- we are bug compatible.
//!    However, there are examples where a bug is so sneaky, some content could have
//!    been created with an assumption that the bug doesn't exist and the bug affects it.
//!
//! 2. They genuinely improve the experience of using Ruffle at the cost
//!    of lowering compatibility with Flash Player.
//!
//! 3. They improve the "perceived" compatibility at the cost of the "real" compatibility.
//!    For instance, something does not work in Flash Player, and we make it work.

use crate::i18n::core_text;
use fluent_templates::LanguageIdentifier;
use std::{collections::HashMap, fmt::Display, str::FromStr, sync::LazyLock};

/// The definition of a compatibility flag.
///
/// It's a static definition, it's meant to describe flags statically.
/// See [`define_ruffle_flags!`].
pub struct CompatibilityFlagDefinition {
    /// The flag we're defining.
    pub flag: CompatibilityFlag,

    /// Short identifier of the flag.
    ///
    /// Has to be unique, used for e.g. specifying the flag in config.
    pub id: &'static str,

    /// Whether the flag is enabled by default in Ruffle.
    pub default_value: bool,

    /// Whether Flash Player behaves as if the flag is enabled or disabled.
    pub flash_player_value: bool,
}

impl CompatibilityFlagDefinition {
    pub fn name(&self, language: &LanguageIdentifier) -> String {
        core_text(language, &format!("flag-{}-name", self.id))
    }

    pub fn description(&self, language: &LanguageIdentifier) -> String {
        core_text(language, &format!("flag-{}-description", self.id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatibilityFlags(HashMap<CompatibilityFlag, bool>);

impl CompatibilityFlags {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn new(flags: HashMap<CompatibilityFlag, bool>) -> Self {
        Self(flags)
    }

    pub fn parse(value: &str, unknown_flags: &mut Vec<String>) -> Self {
        if value.is_empty() {
            return CompatibilityFlags::empty();
        }

        let mut flags = HashMap::new();
        for flag in value.split(",") {
            let flag = flag.trim();
            if flag.is_empty() {
                continue;
            }
            let (id, value) = if let Some(flag) = flag.strip_prefix('-') {
                (flag, false)
            } else if let Some(flag) = flag.strip_prefix('+') {
                (flag, true)
            } else {
                (flag, true)
            };
            if let Some(flag) = CompatibilityFlag::from_id(id) {
                flags.insert(flag, value);
            } else {
                unknown_flags.push(id.to_string());
            }
        }
        CompatibilityFlags::new(flags)
    }

    pub fn parse_ignore_unknown(value: &str) -> Self {
        let mut unknown_flags = Vec::new();
        Self::parse(value, &mut unknown_flags)
    }

    pub fn all_flags() -> &'static [CompatibilityFlag] {
        &RUFFLE_ALL_FLAGS[..]
    }

    /// Check if the given flag is enabled.
    ///
    /// Returns [`Result::Ok`] when the value is overridden, [`Result::Err`]
    /// when the default value is being used.
    pub fn enabled(&self, flag: CompatibilityFlag) -> Result<bool, bool> {
        self.0
            .get(&flag)
            .cloned()
            .ok_or_else(|| flag.definition().default_value)
    }

    pub fn set(&mut self, flag: CompatibilityFlag, enabled: bool) {
        if enabled == flag.definition().default_value {
            self.0.remove(&flag);
        } else {
            self.0.insert(flag, enabled);
        }
    }

    pub fn override_with(&mut self, other: &CompatibilityFlags) {
        for (flag, value) in &other.0 {
            self.0.insert(*flag, *value);
        }
    }

    pub fn overridden(&self) -> std::collections::hash_map::Keys<'_, CompatibilityFlag, bool> {
        self.0.keys()
    }
}

impl Display for CompatibilityFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (&flag, &value) in &self.0 {
            let def = flag.definition();
            if def.default_value != value {
                if !first {
                    write!(f, ",")?;
                } else {
                    first = false;
                }

                if !value {
                    write!(f, "-")?;
                }

                write!(f, "{}", def.id)?;
            }
        }
        Ok(())
    }
}

impl FromStr for CompatibilityFlags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse(s, &mut unknown_flags);
        if unknown_flags.is_empty() {
            Ok(flags)
        } else {
            Err(format!("Unknown flags: {unknown_flags:?}"))
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CompatibilityFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)*) => (1usize + count!($($xs)*));
}

macro_rules! define_compatibility_flags {
    ($(flag($flag:ident, $($key:ident: $value:expr),* $(,)?));* $(;)?) => {
        /// The collection of all compatibility flags.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum CompatibilityFlag {
            $($flag),*
        }

        static RUFFLE_FLAGS: LazyLock<[CompatibilityFlagDefinition; count!($($flag),*)]> = LazyLock::new(|| [
            $(CompatibilityFlagDefinition {
                flag: CompatibilityFlag::$flag,
                id: stringify!($flag),
                $($key: $value),*
            }),*
        ]);

        static RUFFLE_FLAG_IDS: LazyLock<HashMap<&'static str, CompatibilityFlag>> = LazyLock::new(|| HashMap::from([
            $((stringify!($flag), CompatibilityFlag::$flag)),*
        ]));

        static RUFFLE_ALL_FLAGS: LazyLock<[CompatibilityFlag; count!($($flag),*)]> = LazyLock::new(|| [
            $(CompatibilityFlag::$flag),*
        ]);

        impl CompatibilityFlag {
            pub fn from_id(id: &str) -> Option<Self> {
                RUFFLE_FLAG_IDS.get(id).cloned()
            }

            pub fn definition(self) -> &'static CompatibilityFlagDefinition {
                &RUFFLE_FLAGS[self as usize]
            }
        }
    };
}

define_compatibility_flags!(
    flag(
        TabSkip,
        default_value: true,
        flash_player_value: true,
    );
);

#[cfg(test)]
mod tests {
    use crate::flags::{CompatibilityFlag, CompatibilityFlags};

    #[test]
    fn test_parse_empty() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("", &mut unknown_flags);
        assert_eq!(flags, CompatibilityFlags::empty());
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.enabled(CompatibilityFlag::TabSkip), Err(true));
    }

    #[test]
    fn test_parse_positive() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("TabSkip", &mut unknown_flags);
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.enabled(CompatibilityFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_parse_positive2() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("+TabSkip", &mut unknown_flags);
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.enabled(CompatibilityFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_parse_negative() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("-TabSkip", &mut unknown_flags);
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.enabled(CompatibilityFlag::TabSkip), Ok(false));
    }

    #[test]
    fn test_parse_space() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("  TabSkip , ", &mut unknown_flags);
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.enabled(CompatibilityFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_parse_space2() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse(" ,  TabSkip  ", &mut unknown_flags);
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.enabled(CompatibilityFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_to_string1() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("TabSkip", &mut unknown_flags);
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.to_string(), "");
    }

    #[test]
    fn test_to_string2() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("-TabSkip", &mut unknown_flags);
        assert!(unknown_flags.is_empty());

        assert_eq!(flags.to_string(), "-TabSkip");
    }

    #[test]
    fn test_unknown_flag() {
        let mut unknown_flags = Vec::new();
        let flags = CompatibilityFlags::parse("-UnknownFlag", &mut unknown_flags);
        assert_eq!(unknown_flags, vec!["UnknownFlag"]);

        assert_eq!(flags.to_string(), "");
    }
}
