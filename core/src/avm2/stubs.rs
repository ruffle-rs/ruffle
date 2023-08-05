#[macro_export]
macro_rules! avm2_stub_method_context {
    ($context: expr, $class: literal, $method: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: std::borrow::Cow::Borrowed($class),
            method: std::borrow::Cow::Borrowed($method),
            specifics: None,
        };
        $context.stub_tracker.encounter(&STUB);
    };
    ($context: expr, $class: literal, $method: literal, $specifics: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: std::borrow::Cow::Borrowed($class),
            method: std::borrow::Cow::Borrowed($method),
            specifics: Some(std::borrow::Cow::Borrowed($specifics)),
        };
        $context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_method {
    ($activation: ident, $class: literal, $method: literal) => {
        $crate::avm2_stub_method_context!($activation.context, $class, $method);
    };
    ($activation: ident, $class: literal, $method: literal, $specifics: literal) => {
        $crate::avm2_stub_method_context!($activation.context, $class, $method, $specifics);
    };
}

#[macro_export]
macro_rules! avm2_stub_constructor {
    ($activation: ident, $class: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Constructor {
            class: std::borrow::Cow::Borrowed($class),
            specifics: None,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
    ($activation: ident, $class: literal, $specifics: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Constructor {
            class: std::borrow::Cow::Borrowed($class),
            specifics: Some(std::borrow::Cow::Borrowed($specifics)),
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_getter {
    ($activation: ident, $class: literal, $property: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Getter {
            class: std::borrow::Cow::Borrowed($class),
            property: std::borrow::Cow::Borrowed($property),
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_setter {
    ($activation: ident, $class: literal, $property: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Setter {
            class: std::borrow::Cow::Borrowed($class),
            property: std::borrow::Cow::Borrowed($property),
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}
