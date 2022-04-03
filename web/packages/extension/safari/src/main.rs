#[cfg(target_os = "macos")]
mod macos {
    use objc::class;
    use objc::declare::ClassDecl;
    use objc::runtime::Protocol;

    pub fn extension_class() {
        let mut ruffle = ClassDecl::new("RuffleWebExtension", class!(NSObject)).unwrap();

        ruffle.add_protocol(Protocol::get("NSExtensionRequestHandling").unwrap());

        ruffle.register();
    }

    #[link(name = "Foundation", kind = "framework")]
    extern "C" {
        /// Private function used as the entry point of all app extensions.
        ///
        /// In Obj-C/Swift apps, a linker flag is used to set this as the entry
        /// point, and thus app extensions are mainless. However, we need a main fn
        /// to register our Obj-C classes, so we have to call this after we're
        /// done.
        ///
        /// This is almost certainly a "private API" as per App Store guidelines,
        /// but it's part of Foundation (at least after Sierra) and all App
        /// Extensions reference it directly. So it's probably fine to call this.
        pub fn NSExtensionMain();
    }

    pub fn main() {
        extension_class();

        unsafe { NSExtensionMain() }
    }
}

fn main() {
    #[cfg(target_os = "macos")]
    macos::main();

    #[cfg(not(target_os = "macos"))]
    panic!("Safari stub binary not available outside of macOS");
}
