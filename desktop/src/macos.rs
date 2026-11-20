//! Opening files passed to Ruffle by Finder, the Dock or `open`.
//!
//! macOS delivers these as a `kAEOpenDocuments` Apple Event. AppKit forwards
//! it to `application:openURLs:` of the application delegate, but winit owns
//! the delegate and doesn't implement that method, so instead we replace
//! AppKit's handler for the event with our own.

use crate::custom_event::RuffleEvent;
use crate::player::LaunchOptions;
use crate::preferences::GlobalPreferences;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::NSApplicationWillFinishLaunchingNotification;
use objc2_core_services::{kAEOpenDocuments, kCoreEventClass, keyDirectObject, typeAEList};
use objc2_foundation::{
    NSAppleEventDescriptor, NSAppleEventManager, NSNotification, NSNotificationCenter,
};
use ruffle_frontend_utils::content::ContentDescriptor;
use std::cell::{Cell, RefCell};
use url::Url;
use winit::event_loop::EventLoopProxy;

struct Ivars {
    event_loop: EventLoopProxy<RuffleEvent>,
    preferences: GlobalPreferences,
    /// Whether the main window exists. Until then, the last file received is
    /// kept in `pending`.
    started: Cell<bool>,
    /// The file to open once the main window has been created.
    pending: RefCell<Option<Url>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "RuffleOpenDocumentsHandler"]
    #[ivars = Ivars]
    struct OpenDocumentsHandler;

    impl OpenDocumentsHandler {
        #[unsafe(method(applicationWillFinishLaunching:))]
        fn will_finish_launching(&self, _notification: &NSNotification) {
            // AppKit installs its own handler in `finishLaunching` before posting this
            // notification, and dispatches the event that launched the app right after.
            let manager = NSAppleEventManager::sharedAppleEventManager();

            // SAFETY: The selector matches the signature of `handle_open_documents`,
            // and `Drop for OpenDocuments` removes the handler before `self` is freed.
            unsafe {
                manager.setEventHandler_andSelector_forEventClass_andEventID(
                    self,
                    sel!(handleOpenDocuments:withReplyEvent:),
                    kCoreEventClass,
                    kAEOpenDocuments,
                );
            }
        }

        #[unsafe(method(handleOpenDocuments:withReplyEvent:))]
        fn handle_open_documents(
            &self,
            event: &NSAppleEventDescriptor,
            _reply: Option<&NSAppleEventDescriptor>,
        ) {
            let urls = file_urls(event);

            // Ruffle shows one movie at a time, so the last file wins.
            let Some(url) = urls.last() else {
                tracing::warn!("Received an open documents event without any files");
                return;
            };

            let ivars = self.ivars();

            if ivars.started.get() {
                let options = LaunchOptions::from(&ivars.preferences);
                let descriptor = ContentDescriptor {
                    url,
                    root_content_path: None,
                };
                let _ = ivars
                    .event_loop
                    .send_event(RuffleEvent::Open(descriptor, Box::new(options)));
            } else {
                ivars.pending.replace(Some(url));
            }
        }
    }
);

fn file_urls(event: &NSAppleEventDescriptor) -> impl Iterator<Item = Url> {
    event
        .paramDescriptorForKeyword(keyDirectObject)
        .and_then(|direct| direct.coerceToDescriptorType(typeAEList))
        .into_iter()
        .flat_map(|list| {
            // Descriptor lists are 1-based.
            (1..=list.numberOfItems())
                .filter_map(move |index| list.descriptorAtIndex(index)?.fileURLValue())
        })
        .filter_map(|url| Url::parse(&url.absoluteString()?.to_string()).ok())
}

/// Handles `kAEOpenDocuments` Apple Events for as long as it is alive.
pub struct OpenDocuments(Retained<OpenDocumentsHandler>);

impl OpenDocuments {
    /// Must be called before the event loop starts running, so that the
    /// handler can be installed when the app finishes launching.
    pub fn new(event_loop: EventLoopProxy<RuffleEvent>, preferences: GlobalPreferences) -> Self {
        let mtm = MainThreadMarker::new().expect("Ruffle should run on the main thread");
        let handler = OpenDocumentsHandler::alloc(mtm).set_ivars(Ivars {
            event_loop,
            preferences,
            started: Cell::new(false),
            pending: RefCell::new(None),
        });

        // SAFETY: `NSObject`'s `init` is the designated initializer.
        let handler: Retained<OpenDocumentsHandler> = unsafe { msg_send![super(handler), init] };

        // SAFETY: The selector matches the signature of `will_finish_launching`.
        unsafe {
            NSNotificationCenter::defaultCenter().addObserver_selector_name_object(
                &handler,
                sel!(applicationWillFinishLaunching:),
                Some(NSApplicationWillFinishLaunchingNotification),
                None,
            );
        }

        Self(handler)
    }

    /// Returns the file that launched Ruffle, if any. Files received after
    /// this call are sent to the event loop as [`RuffleEvent::Open`].
    pub fn start(&self) -> Option<Url> {
        let ivars = self.0.ivars();

        ivars.started.set(true);

        ivars.pending.take()
    }
}

impl Drop for OpenDocuments {
    fn drop(&mut self) {
        // SAFETY: `removeObserver` has no documented safety requirements; it's
        // only unsafe because objc2 doesn't know that it's safe to call. `self.0`
        // is the observer added in `new`.
        unsafe { NSNotificationCenter::defaultCenter().removeObserver(&self.0) };

        NSAppleEventManager::sharedAppleEventManager()
            .removeEventHandlerForEventClass_andEventID(kCoreEventClass, kAEOpenDocuments);
    }
}
