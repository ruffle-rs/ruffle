use fnv::FnvHashSet;
use std::{
    borrow::Cow,
    cell::RefCell,
    fmt::{Display, Formatter},
};

#[cfg(feature = "known_stubs")]
#[linkme::distributed_slice]
pub static KNOWN_STUBS: [Stub] = [..];

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Clone)]
pub enum Stub {
    Avm1Method {
        class: &'static str,
        method: &'static str,
        specifics: Option<&'static str>,
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
    inner: RefCell<FnvHashSet<Stub>>,
}

impl StubCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encounter(&self, stub: &Stub) {
        let mut inner = self.inner.borrow_mut();
        if !inner.contains(stub) {
            tracing::warn!("Encountered stub: {stub}");
            inner.insert(stub.clone());
        }
    }
}

#[macro_export]
macro_rules! avm2_stub_method {
    ($tracker: expr, $class: literal, $method: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: std::borrow::Cow::Borrowed($class),
            method: std::borrow::Cow::Borrowed($method),
            specifics: None,
        };
        $tracker.encounter(&STUB);
    };
    ($tracker: expr, $class: literal, $method: literal, $specifics: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: std::borrow::Cow::Borrowed($class),
            method: std::borrow::Cow::Borrowed($method),
            specifics: Some(std::borrow::Cow::Borrowed($specifics)),
        };
        $tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_constructor {
    ($tracker: expr, $class: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Constructor {
            class: std::borrow::Cow::Borrowed($class),
            specifics: None,
        };
        $tracker.encounter(&STUB);
    };
    ($tracker: expr, $class: literal, $specifics: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Constructor {
            class: std::borrow::Cow::Borrowed($class),
            specifics: Some(std::borrow::Cow::Borrowed($specifics)),
        };
        $tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_getter {
    ($tracker: expr, $class: literal, $property: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Getter {
            class: std::borrow::Cow::Borrowed($class),
            property: std::borrow::Cow::Borrowed($property),
        };
        $tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_setter {
    ($tracker: expr, $class: literal, $property: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Setter {
            class: std::borrow::Cow::Borrowed($class),
            property: std::borrow::Cow::Borrowed($property),
        };
        $tracker.encounter(&STUB);
    };
}
