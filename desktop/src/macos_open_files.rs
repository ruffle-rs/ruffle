use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, Once, OnceLock};

use objc::declare::ClassDecl;
use objc::runtime::{BOOL, Class, NO, Object, Sel, YES};
use objc::{class, msg_send, sel, sel_impl};

type Id = *mut Object;

static PENDING_FILES: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();
static INSTALL_DELEGATE: Once = Once::new();
static mut APP_DELEGATE: Id = std::ptr::null_mut();

fn pending_files() -> &'static Mutex<Vec<PathBuf>> {
    PENDING_FILES.get_or_init(|| Mutex::new(Vec::new()))
}

fn push_path(path: PathBuf) {
    if let Ok(mut pending) = pending_files().lock() {
        pending.push(path);
    }
}

fn nsstring_to_path(value: Id) -> Option<PathBuf> {
    let utf8: *const c_char = unsafe { msg_send![value, UTF8String] };
    if utf8.is_null() {
        return None;
    }

    let path = unsafe { CStr::from_ptr(utf8) }.to_string_lossy();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path.into_owned()))
    }
}

extern "C" fn application_open_file(_: &Object, _: Sel, _: Id, file: Id) -> BOOL {
    if let Some(path) = nsstring_to_path(file) {
        tracing::info!("Received macOS open-file event: {}", path.display());
        push_path(path);
        YES
    } else {
        NO
    }
}

extern "C" fn application_open_files(_: &Object, _: Sel, _: Id, files: Id) {
    let count: usize = unsafe { msg_send![files, count] };
    for index in 0..count {
        let file: Id = unsafe { msg_send![files, objectAtIndex: index] };
        if let Some(path) = nsstring_to_path(file) {
            tracing::info!("Received macOS open-files event: {}", path.display());
            push_path(path);
        }
    }
}

extern "C" fn application_open_urls(_: &Object, _: Sel, _: Id, urls: Id) {
    let count: usize = unsafe { msg_send![urls, count] };
    for index in 0..count {
        let url: Id = unsafe { msg_send![urls, objectAtIndex: index] };
        let path: Id = unsafe { msg_send![url, path] };
        if path.is_null() {
            continue;
        }

        if let Some(path) = nsstring_to_path(path) {
            tracing::info!("Received macOS open-urls event: {}", path.display());
            push_path(path);
        }
    }
}

fn delegate_class() -> *const Class {
    if let Some(class) = Class::get("RuffleFileOpenDelegate") {
        return class;
    }

    let mut decl = ClassDecl::new("RuffleFileOpenDelegate", class!(NSObject))
        .expect("Failed to declare RuffleFileOpenDelegate");
    decl.add_method(
        sel!(application:openFile:),
        application_open_file as extern "C" fn(&Object, Sel, Id, Id) -> BOOL,
    );
    decl.add_method(
        sel!(application:openFiles:),
        application_open_files as extern "C" fn(&Object, Sel, Id, Id),
    );
    decl.add_method(
        sel!(application:openURLs:),
        application_open_urls as extern "C" fn(&Object, Sel, Id, Id),
    );
    decl.register()
}

pub fn install_open_file_handler() {
    INSTALL_DELEGATE.call_once(|| {
        let class = delegate_class();
        let delegate: Id = unsafe { msg_send![class, new] };
        let app: Id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        unsafe {
            let _: () = msg_send![app, setDelegate: delegate];
            APP_DELEGATE = delegate;
        }
    });
}

pub fn take_pending_open_files() -> Vec<PathBuf> {
    if let Ok(mut pending) = pending_files().lock() {
        std::mem::take(&mut *pending)
    } else {
        Vec::new()
    }
}
