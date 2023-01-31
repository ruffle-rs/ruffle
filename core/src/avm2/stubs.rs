#[macro_export]
macro_rules! avm2_stub_method {
    ($activation: ident, $class: literal, $method: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: $class,
            method: $method,
            specifics: None,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
    ($activation: ident, $class: literal, $method: literal, $specifics: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: $class,
            method: $method,
            specifics: Some($specifics),
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_getter {
    ($activation: ident, $class: literal, $method: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Getter {
            class: $class,
            field: $method,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_setter {
    ($activation: ident, $class: literal, $method: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Setter {
            class: $class,
            field: $method,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}
