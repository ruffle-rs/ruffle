use objc::declare::ClassDecl;
use objc::runtime::Protocol;
use objc::class;

fn extension_class() {
    let mut ruffle = ClassDecl::new("RuffleWebExtension", class!(NSObject)).unwrap();

    ruffle.add_protocol(Protocol::get("NSExtensionRequestHandling").unwrap());

    ruffle.register();
}

#[link(name="Foundation", kind="framework")]
extern {
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
    fn NSExtensionMain();
}

fn main() {
    extension_class();

    unsafe { NSExtensionMain() }
}