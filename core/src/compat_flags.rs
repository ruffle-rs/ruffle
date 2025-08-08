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
use indexmap::IndexMap;
use std::{collections::HashMap, fmt::Display, sync::LazyLock};

/// The definition of a compatibility flag.
///
/// It's a static definition, it's meant to describe flags statically.
/// See [`define_ruffle_flags!`].
pub struct CompatFlagDefinition {
    /// The flag we're defining.
    pub flag: CompatFlag,

    /// Short identifier of the flag.
    ///
    /// Has to be unique, used for e.g. specifying the flag in config.
    pub id: &'static str,

    /// Whether the flag is enabled by default in Ruffle.
    pub default_value: bool,

    /// Whether Flash Player behaves as if the flag is enabled or disabled.
    pub flash_player_value: bool,
}

impl CompatFlagDefinition {
    pub fn name(&self, language: &LanguageIdentifier) -> String {
        core_text(language, &format!("compat-flag-{}-name", self.id))
    }

    pub fn description(&self, language: &LanguageIdentifier) -> String {
        core_text(language, &format!("compat-flag-{}-description", self.id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatFlags {
    /// Contains all flags, even those that are unknown for
    /// compatibility across versions.
    all_flags: IndexMap<String, bool>,

    parsed_flags: HashMap<CompatFlag, bool>,
}

impl CompatFlags {
    pub fn empty() -> Self {
        Self {
            all_flags: IndexMap::new(),
            parsed_flags: HashMap::new(),
        }
    }

    pub fn parse(value: &str) -> Self {
        if value.is_empty() {
            return CompatFlags::empty();
        }

        let mut all_flags = IndexMap::new();
        let mut parsed_flags = HashMap::new();
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

            all_flags.insert(id.to_string(), value);
            if let Some(flag) = CompatFlag::from_id(id) {
                parsed_flags.insert(flag, value);
            }
        }

        CompatFlags {
            all_flags,
            parsed_flags,
        }
    }

    pub fn all_known_flags() -> &'static [CompatFlag] {
        &RUFFLE_ALL_FLAGS[..]
    }

    /// Check if the given flag is enabled.
    ///
    /// Returns [`Result::Ok`] when the value is overridden, [`Result::Err`]
    /// when the default value is being used.
    pub fn enabled(&self, flag: CompatFlag) -> Result<bool, bool> {
        self.parsed_flags
            .get(&flag)
            .cloned()
            .ok_or_else(|| flag.definition().default_value)
    }

    pub fn set(&mut self, flag: CompatFlag, enabled: bool) {
        let id = flag.definition().id;
        self.all_flags.insert(id.to_string(), enabled);
        self.parsed_flags.insert(flag, enabled);
    }

    pub fn reset(&mut self, flag: CompatFlag) {
        let id = flag.definition().id;
        self.all_flags.shift_remove(id);
        self.parsed_flags.remove(&flag);
    }
}

impl Display for CompatFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (id, &value) in &self.all_flags {
            if !first {
                write!(f, ",")?;
            } else {
                first = false;
            }

            if !value {
                write!(f, "-")?;
            }

            write!(f, "{id}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CompatFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(CompatFlags::parse(&s))
    }
}

macro_rules! count {
    () => (0usize);
    ($x:tt $($xs:tt)*) => (1usize + count!($($xs)*));
}

macro_rules! define_compat_flags {
    ($(flag($flag:ident, $($key:ident: $value:expr),* $(,)?));* $(;)?) => {
        /// The collection of all compatibility flags.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum CompatFlag {
            $($flag),*
        }

        static RUFFLE_FLAGS: LazyLock<[CompatFlagDefinition; count!($($flag),*)]> = LazyLock::new(|| [
            $(CompatFlagDefinition {
                flag: CompatFlag::$flag,
                id: stringify!($flag),
                $($key: $value),*
            }),*
        ]);

        static RUFFLE_FLAG_IDS: LazyLock<HashMap<&'static str, CompatFlag>> = LazyLock::new(|| HashMap::from([
            $((stringify!($flag), CompatFlag::$flag)),*
        ]));

        static RUFFLE_ALL_FLAGS: LazyLock<[CompatFlag; count!($($flag),*)]> = LazyLock::new(|| [
            $(CompatFlag::$flag),*
        ]);

        impl CompatFlag {
            pub fn from_id(id: &str) -> Option<Self> {
                RUFFLE_FLAG_IDS.get(id).cloned()
            }

            pub fn definition(self) -> &'static CompatFlagDefinition {
                &RUFFLE_FLAGS[self as usize]
            }
        }
    };
}

define_compat_flags!(
    flag(
        TabSkip,
        default_value: true,
        flash_player_value: true,
    );
);

#[cfg(test)]
mod tests {
    use crate::compat_flags::{CompatFlag, CompatFlags};

    #[test]
    fn test_parse_empty() {
        let flags = CompatFlags::parse("");
        assert_eq!(flags, CompatFlags::empty());
        assert_eq!(flags.enabled(CompatFlag::TabSkip), Err(true));
    }

    #[test]
    fn test_parse_positive() {
        let flags = CompatFlags::parse("TabSkip");
        assert_eq!(flags.enabled(CompatFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_parse_positive2() {
        let flags = CompatFlags::parse("+TabSkip");
        assert_eq!(flags.enabled(CompatFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_parse_negative() {
        let flags = CompatFlags::parse("-TabSkip");
        assert_eq!(flags.enabled(CompatFlag::TabSkip), Ok(false));
    }

    #[test]
    fn test_parse_space() {
        let flags = CompatFlags::parse("  TabSkip , ");
        assert_eq!(flags.enabled(CompatFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_parse_space2() {
        let flags = CompatFlags::parse(" ,  TabSkip  ");
        assert_eq!(flags.enabled(CompatFlag::TabSkip), Ok(true));
    }

    #[test]
    fn test_to_string1() {
        let flags = CompatFlags::parse("TabSkip");
        assert_eq!(flags.to_string(), "TabSkip");
    }

    #[test]
    fn test_to_string2() {
        let flags = CompatFlags::parse("-TabSkip");
        assert_eq!(flags.to_string(), "-TabSkip");
    }

    #[test]
    fn test_to_string_unknown_flag() {
        let flags = CompatFlags::parse("-A,B");
        assert_eq!(flags.to_string(), "-A,B");
    }

    #[test]
    fn test_to_string_spaces() {
        let flags = CompatFlags::parse(",   -A ,,  B,   ");
        assert_eq!(flags.to_string(), "-A,B");
    }

    #[test]
    fn test_to_string_order() {
        let flags = CompatFlags::parse(" ,K,S,Y,T,R,V,");
        assert_eq!(flags.to_string(), "K,S,Y,T,R,V");
    }

    #[test]
    fn test_to_string_duplicates() {
        let flags = CompatFlags::parse("A,B,A,C,A,D,C,B");
        assert_eq!(flags.to_string(), "A,B,C,D");
    }

    #[test]
    fn test_unknown_flag() {
        let flags = CompatFlags::parse("-UnknownFlag ,, UnknownFlag2,  +TabSkip");
        assert_eq!(flags.to_string(), "-UnknownFlag,UnknownFlag2,TabSkip");
    }

    #[test]
    fn test_eq() {
        let flags1 = CompatFlags::parse("+TabSkip");
        let flags2 = CompatFlags::parse(" TabSkip");
        assert_eq!(flags1, flags2);

        let flags1 = CompatFlags::parse("-TabSkip");
        let flags2 = CompatFlags::parse(" TabSkip");
        assert_ne!(flags1, flags2);

        let flags1 = CompatFlags::parse("");
        let flags2 = CompatFlags::parse(" TabSkip");
        assert_ne!(flags1, flags2);
    }

    #[test]
    fn test_eq_unknown_flags() {
        let flags1 = CompatFlags::parse("-Unknown");
        let flags2 = CompatFlags::parse("-Unknown");
        assert_eq!(flags1, flags2);

        let flags1 = CompatFlags::parse("-Unknown");
        let flags2 = CompatFlags::parse("Unknown");
        assert_ne!(flags1, flags2);

        let flags1 = CompatFlags::parse("-Unknown,TabSkip");
        let flags2 = CompatFlags::parse("Unknown,TabSkip");
        assert_ne!(flags1, flags2);
    }

    #[test]
    fn test_set_positive() {
        let mut flags = CompatFlags::parse("A,B,C");
        flags.set(CompatFlag::TabSkip, true);
        assert_eq!(flags.to_string(), "A,B,C,TabSkip");
    }

    #[test]
    fn test_set_negative() {
        let mut flags = CompatFlags::parse("A,B,C");
        flags.set(CompatFlag::TabSkip, false);
        assert_eq!(flags.to_string(), "A,B,C,-TabSkip");
    }

    #[test]
    fn test_set_update_same() {
        let mut flags = CompatFlags::parse("A,TabSkip,B,C");
        flags.set(CompatFlag::TabSkip, true);
        assert_eq!(flags.to_string(), "A,TabSkip,B,C");
    }

    #[test]
    fn test_set_update_negative() {
        let mut flags = CompatFlags::parse("A,TabSkip,B,C");
        flags.set(CompatFlag::TabSkip, false);
        assert_eq!(flags.to_string(), "A,-TabSkip,B,C");
    }

    #[test]
    fn test_reset_positive() {
        let mut flags = CompatFlags::parse("A,B,TabSkip,C");
        flags.reset(CompatFlag::TabSkip);
        assert_eq!(flags.to_string(), "A,B,C");
    }

    #[test]
    fn test_reset_negative() {
        let mut flags = CompatFlags::parse("A,B,-TabSkip,C");
        flags.reset(CompatFlag::TabSkip);
        assert_eq!(flags.to_string(), "A,B,C");
    }
}
