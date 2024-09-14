use fnv::FnvHashSet;
use std::borrow::Cow;
use std::collections::hash_set::Iter;
use std::fmt::{Debug, Display, Formatter};

#[cfg(feature = "known_stubs")]
#[linkme::distributed_slice]
pub static KNOWN_STUBS: [Stub] = [..];

#[cfg(feature = "known_stubs")]
mod external {
    include!(concat!(env!("OUT_DIR"), "/actionscript_stubs.rs"));
}

#[cfg(feature = "known_stubs")]
pub fn get_known_stubs() -> FnvHashSet<&'static Stub> {
    let mut result = FnvHashSet::default();
    for stub in KNOWN_STUBS.iter() {
        result.insert(stub);
    }
    for stub in external::AS_DEFINED_STUBS {
        result.insert(stub);
    }
    result
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Clone)]
pub enum Stub {
    Avm1Method {
        class: &'static str,
        method: &'static str,
        specifics: Option<&'static str>,
    },
    Avm1Constructor {
        class: &'static str,
    },
    Avm2Method {
        class: Cow<'static, str>,
        method: Cow<'static, str>,
        specifics: Option<Cow<'static, str>>,
    },
    Avm2Getter {
        class: Cow<'static, str>,
        property: Cow<'static, str>,
    },
    Avm2Setter {
        class: Cow<'static, str>,
        property: Cow<'static, str>,
    },
    Avm2Constructor {
        class: Cow<'static, str>,
        specifics: Option<Cow<'static, str>>,
    },
    Other(Cow<'static, str>),
}

impl Stub {
    pub fn avm2_class(&self) -> Option<Cow<'static, str>> {
        match self {
            Stub::Avm2Method { class, .. } => Some(class.clone()),
            Stub::Avm2Getter { class, .. } => Some(class.clone()),
            Stub::Avm2Setter { class, .. } => Some(class.clone()),
            Stub::Avm2Constructor { class, .. } => Some(class.clone()),
            _ => None,
        }
    }
}

impl Display for Stub {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stub::Avm1Method {
                class,
                method,
                specifics: None,
            } => {
                write!(f, "AVM1 {class}.{method}()")
            }
            Stub::Avm1Constructor { class } => {
                write!(f, "AVM1 {class}() constructor")
            }
            Stub::Avm1Method {
                class,
                method,
                specifics: Some(specifics),
            } => {
                write!(f, "AVM1 {class}.{method}() {specifics}")
            }
            Stub::Avm2Method {
                class,
                method,
                specifics: None,
            } => {
                write!(f, "AVM2 {class}.{method}()")
            }
            Stub::Avm2Method {
                class,
                method,
                specifics: Some(specifics),
            } => {
                write!(f, "AVM2 {class}.{method}() {specifics}")
            }
            Stub::Avm2Getter {
                class,
                property: field,
            } => {
                write!(f, "AVM2 {class}.{field} getter")
            }
            Stub::Avm2Setter {
                class,
                property: field,
            } => {
                write!(f, "AVM2 {class}.{field} setter")
            }
            Stub::Avm2Constructor {
                class,
                specifics: None,
            } => {
                write!(f, "AVM2 {class} constructor")
            }
            Stub::Avm2Constructor {
                class,
                specifics: Some(specifics),
            } => {
                write!(f, "AVM2 {class} constructor {specifics}")
            }
            Stub::Other(text) => write!(f, "{text}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct StubCollection {
    inner: FnvHashSet<Stub>,
}

impl StubCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encounter(&mut self, stub: &Stub) {
        if !self.inner.contains(stub) {
            tracing::warn!("Encountered stub: {stub}");
            self.inner.insert(stub.clone());
        }
    }

    pub fn iter(&self) -> Iter<Stub> {
        self.inner.iter()
    }
}

#[macro_export]
macro_rules! context_stub {
    ($context: ident, $message: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub =
            $crate::stub::Stub::Other(std::borrow::Cow::Borrowed($message));
        $context.stub_tracker.encounter(&STUB);
    };
}
