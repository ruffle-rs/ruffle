//! ruffle_iggy_shim -- C-ABI bridge from LCE's `IggyPlayer*` API to Ruffle.
//!
//! Phase 4.1 of the LCE rebuild (see REBUILD_PLAN_iggy.md). Every public
//! function is `#[unsafe(no_mangle)] extern "C"`; no Rust types cross the
//! FFI boundary. Opaque pointer handles only.
//!
//! Naming convention: `iggy_open_*` for shim-internal Rust functions.
//! The C++ shim in 4.2 will export the actual `IggyInit` / `IggyPlayer*`
//! names that LCE calls and forward to these.
//!
//! Phase 4.1 scope = stubs only. Just enough to:
//!   1. Prove the cargo workspace integration works
//!   2. Prove cdylib emission produces a Windows .dll
//!   3. Prove C-ABI symbols are exported under the expected names
//!
//! 4.3+ wires actual Player lifecycle. 4.5 wires the AS3 name-token
//! dispatch (per `tools/iggy/api_inventory.md` 57-name corpus).

use std::any::Any;
use std::os::raw::c_char;
use std::ffi::{c_void, CStr};
use std::sync::{Arc, Mutex};

use image::RgbaImage;
use ruffle_core::backend::ui::FontDefinition;
use ruffle_core::external::{ExternalInterfaceProvider, Value as ExtValue};
use ruffle_core::context::UpdateContext;
use ruffle_core::font::FontFileData;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Color, Player, PlayerBuilder};
use ruffle_render::backend::ViewportDimensions;
use ruffle_render_wgpu::backend::WgpuRenderBackend;
use ruffle_render_wgpu::wgpu;

/// Global font registry. Populated by `iggy_open_install_truetype_utf8`
/// callbacks from the C++ side; applied to every newly-built Player via
/// `Player::register_device_font` so fonts persist across the many
/// concurrent player instances LCE creates.
struct FontEntry {
    name: String,
    data: Arc<Vec<u8>>,
}
static FONT_REGISTRY: Mutex<Vec<FontEntry>> = Mutex::new(Vec::new());

/// Global library registry. Populated by every successful
/// `iggy_open_library_create_from_memory` (i.e. every C++-side
/// `IggyLibraryCreateFromMemoryUTF16`). At each `iggy_open_player_create_*`
/// we replay these into the new Player's root ApplicationDomain via
/// `Player::inject_secondary_swf_abc`, so that scenes whose root SWF only
/// PlaceByClass-references classes defined elsewhere (LCE's standard
/// `MainMenu.swf` -> `fourj.Buttons.FJ_MenuButton_Normal` lives in
/// `skinHD.swf`) can find them at runtime. Synthesises Iggy's
/// "one Library, many Players" semantic on top of Ruffle's per-Player
/// ApplicationDomain isolation.
static LIBRARY_REGISTRY: Mutex<Vec<Arc<SwfMovie>>> = Mutex::new(Vec::new());

// LCE Phase 4.8: full-inject runs character preload + SymbolClass binding
// per Player. LCE's Iggy lifecycle leaks Players (creates ~90/sec across
// scene transitions, never destroys), so we cap full-inject to the first
// N global calls. After the cap, we fall back to the cheap DoABC-only path
// (inject_secondary_swf_abc) which doesn't grow the per-Player library.
// The early Players are the menu chain that needs button visuals; later
// Players are intro/transition scenes whose missed button bindings are
// imperceptible.
static FULL_INJECT_BUDGET: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(4);

fn _inject_library_abcs(player: &Arc<Mutex<Player>>) {
    let libs: Vec<Arc<SwfMovie>> = match LIBRARY_REGISTRY.lock() {
        Ok(g) => g.clone(),
        Err(_) => { _bc("inject_abcs: registry poisoned"); return; }
    };
    if libs.is_empty() {
        _bc("inject_abcs: registry empty");
        return;
    }
    let use_full = FULL_INJECT_BUDGET
        .fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |b| if b > 0 { Some(b - 1) } else { None },
        )
        .is_ok();
    _bc(&format!(
        "inject_abcs: processing {} libraries (mode={})",
        libs.len(),
        if use_full { "full" } else { "abc-only" }
    ));
    if let Ok(mut p) = player.try_lock() {
        let mut total = 0u32;
        for (i, lib) in libs.iter().enumerate() {
            let n = if use_full {
                p.inject_secondary_swf_full(lib.clone())
            } else {
                p.inject_secondary_swf_abc(lib.clone())
            };
            _bc(&format!("  lib[{}]: {} units processed", i, n));
            total += n;
        }
        _bc(&format!(
            "inject_abcs: {} total units across {} libs",
            total,
            libs.len()
        ));
    } else {
        _bc("inject_abcs: player lock fail");
    }
}

fn _apply_registered_fonts(player: &Arc<Mutex<Player>>) {
    let entries: Vec<FontEntry> = match FONT_REGISTRY.lock() {
        Ok(g) => g.iter().map(|e| FontEntry { name: e.name.clone(), data: e.data.clone() }).collect(),
        Err(_) => return,
    };
    if entries.is_empty() { return; }
    if let Ok(mut p) = player.try_lock() {
        for e in entries {
            let arc_data: Arc<dyn AsRef<[u8]>> = e.data.clone();
            let def = FontDefinition::FontFile {
                name: e.name.clone(),
                is_bold: false,
                is_italic: false,
                data: FontFileData::new_shared(arc_data),
                index: 0,
            };
            p.register_device_font(def);
        }
    }
}

/// Install a TrueType font (raw TTF bytes) into the global font registry.
/// Subsequent player creations will register the font automatically;
/// already-created players need a manual re-apply (Phase-4.6 limitation,
/// LCE installs fonts at boot before any Player exists so this rarely
/// matters).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_install_truetype_utf8(
    name_utf8: *const c_char,
    name_len: i32,
    ttf_data: *const u8,
    ttf_len: usize,
) {
    if name_utf8.is_null() || ttf_data.is_null() || ttf_len == 0 { return; }
    let name = if name_len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(name_utf8 as *const u8, name_len as usize) };
        String::from_utf8_lossy(slice).into_owned()
    } else {
        unsafe { CStr::from_ptr(name_utf8) }.to_string_lossy().into_owned()
    };
    let data: Vec<u8> = unsafe { std::slice::from_raw_parts(ttf_data, ttf_len) }.to_vec();
    if let Ok(mut g) = FONT_REGISTRY.lock() {
        g.push(FontEntry { name: name.clone(), data: Arc::new(data) });
        _bc(&format!("font_registered name='{}' bytes={}", name, ttf_len));
    }
}

// ====================================================================== Phase 4.5
// ExternalInterface bridge: AS3 calls ExternalInterface.call(name, args)
// inside Ruffle; Ruffle invokes ExternalInterfaceProvider::call_method;
// we translate Value -> IggyDataValue and forward to the game's
// registered C callback via iggy_open_dispatch_as3_to_cpp.

#[repr(C)]
#[derive(Clone, Copy)]
struct IggyDataValueRaw {
    typ: i32,
    _pad: i32, // __RAD64__ alignment
    temp_ref: usize,
    union_data: [u8; 16],
}

impl IggyDataValueRaw {
    fn undefined() -> Self {
        Self { typ: 1, _pad: 0, temp_ref: 0, union_data: [0; 16] }
    }
    fn null() -> Self {
        Self { typ: 2, _pad: 0, temp_ref: 0, union_data: [0; 16] }
    }
    fn boolean(b: bool) -> Self {
        let mut d = [0u8; 16];
        d[0..4].copy_from_slice(&(if b { 1i32 } else { 0i32 }).to_le_bytes());
        Self { typ: 3, _pad: 0, temp_ref: 0, union_data: d }
    }
    fn number(n: f64) -> Self {
        let mut d = [0u8; 16];
        d[0..8].copy_from_slice(&n.to_le_bytes());
        Self { typ: 4, _pad: 0, temp_ref: 0, union_data: d }
    }
    /// String pointer lifetime must outlive the FFI call.
    fn string_utf16(ptr: *const u16, len: i32) -> Self {
        let mut d = [0u8; 16];
        d[0..8].copy_from_slice(&(ptr as usize).to_le_bytes());
        d[8..12].copy_from_slice(&len.to_le_bytes());
        Self { typ: 6, _pad: 0, temp_ref: 0, union_data: d }
    }
}

// Function pointer registered by the C++ host (via
// iggy_open_set_as3_dispatch) at init time. We can't link backwards to
// an EXE-defined symbol from the DLL, so the host hands us its dispatch
// function pointer and we invoke through this slot.
type As3DispatchFn = extern "C" fn(
    player: *mut OpaquePlayer,
    func_name_utf16: *const u16,
    func_name_len: i32,
    args: *const IggyDataValueRaw,
    num_args: i32,
) -> i32;

use std::sync::atomic::{AtomicPtr, Ordering};
static AS3_DISPATCH: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

/// Register the C++ host's AS3-to-Cpp dispatch function. Called once
/// from the host's `IggySetAS3ExternalFunctionCallbackUTF16` impl after
/// it captures the game's actual callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_set_as3_dispatch(fn_ptr: Option<As3DispatchFn>) {
    let p = match fn_ptr {
        Some(f) => f as *mut (),
        None => std::ptr::null_mut(),
    };
    AS3_DISPATCH.store(p, Ordering::SeqCst);
}

/// C++ -> AS3: invoke an AS3 method registered via
/// flash.external.ExternalInterface.addCallback. `name_utf8` is NUL-
/// terminated UTF-8. `args` is a flat array of `num_args` IggyDataValueRaw
/// already shaped on the C++ side. Returns 1 on success, 0 on failure.
///
/// This is the back-half of Phase 4.5 IggyPlayerCallMethodRS: LCE
/// scenes register `Init`, `SetSafeZone`, `SetIntroPlatform`, etc. on
/// the AS3 side via `ExternalInterface.addCallback(...)` and the C++
/// scene constructor invokes them by name.
unsafe fn _decode_iggy_args(args: *const IggyDataValueRaw, num_args: i32) -> Vec<ExtValue> {
    let mut out: Vec<ExtValue> = Vec::with_capacity(num_args.max(0) as usize);
    if num_args <= 0 || args.is_null() { return out; }
    for i in 0..num_args as isize {
        let raw = unsafe { &*args.offset(i) };
        out.push(match raw.typ {
            1 => ExtValue::Undefined,
            2 => ExtValue::Null,
            3 => {
                let b = i32::from_le_bytes([raw.union_data[0], raw.union_data[1], raw.union_data[2], raw.union_data[3]]);
                ExtValue::Bool(b != 0)
            }
            4 => {
                let n = f64::from_le_bytes([
                    raw.union_data[0], raw.union_data[1], raw.union_data[2], raw.union_data[3],
                    raw.union_data[4], raw.union_data[5], raw.union_data[6], raw.union_data[7],
                ]);
                ExtValue::Number(n)
            }
            6 => {
                let ptr_bytes = &raw.union_data[0..8];
                let len_bytes = &raw.union_data[8..12];
                let ptr_v = usize::from_le_bytes(ptr_bytes.try_into().unwrap_or([0;8])) as *const u16;
                let len = i32::from_le_bytes(len_bytes.try_into().unwrap_or([0;4]));
                if ptr_v.is_null() || len <= 0 {
                    ExtValue::String(String::new())
                } else {
                    let slice = unsafe { std::slice::from_raw_parts(ptr_v, len as usize) };
                    ExtValue::String(String::from_utf16_lossy(slice))
                }
            }
            _ => ExtValue::Undefined,
        });
    }
    out
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_call_as3(
    p: *mut OpaquePlayer,
    name_utf8: *const c_char,
    args: *const IggyDataValueRaw,
    num_args: i32,
) -> i32 {
    if p.is_null() || name_utf8.is_null() { return 0; }
    let opaque = unsafe { &mut *p };
    let name = unsafe { CStr::from_ptr(name_utf8) }.to_string_lossy().into_owned();
    let ext_args = unsafe { _decode_iggy_args(args, num_args) };
    if let Ok(mut player) = opaque.player.try_lock() {
        let ret = player.call_root_method_avm2(&name, ext_args);
        static CALL_LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = CALL_LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n < 30 {
            _bc(&format!("call_as3 '{}' -> {:?}", name, ret));
        }
        1
    } else {
        0
    }
}

/// Phase 4.8b: path-aware AS3 method dispatch. `path_utf8` is a "."-joined
/// chain of AS3 property names from root (e.g. "Button1" or "myDoc.skin").
/// Empty path or NULL = invoke on root (same as iggy_open_call_as3).
///
/// # Safety
/// `p` must be a valid OpaquePlayer handle. Strings must be NUL-terminated
/// UTF-8 (or NULL for path). `args`/`num_args` follow IggyDataValueRaw
/// conventions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_call_as3_path(
    p: *mut OpaquePlayer,
    path_utf8: *const c_char,
    method_utf8: *const c_char,
    args: *const IggyDataValueRaw,
    num_args: i32,
) -> i32 {
    if p.is_null() || method_utf8.is_null() { return 0; }
    let opaque = unsafe { &mut *p };
    let path = if path_utf8.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(path_utf8) }.to_string_lossy().into_owned()
    };
    let method = unsafe { CStr::from_ptr(method_utf8) }.to_string_lossy().into_owned();
    let ext_args = unsafe { _decode_iggy_args(args, num_args) };
    // LCE Phase 4.8b: path plumbing works (Hud.Description.Init now lands
    // on the right target), but actually invoking the previously-failing
    // child Init() bodies exposes a downstream crash inside the game
    // (~3s in, 0xC0000409 stack-buffer-overrun fastfail, around the
    // first TutorialPopup/Hud Init chain). Until that's diagnosed we
    // default to bypass-child-calls so the boot baseline keeps booting
    // (mirrors pre-plumbing AS3-side behaviour). Set LCE_CHILD_CALLS=1
    // to opt in for repro / continued work on the downstream crash.
    let allow_child = std::env::var_os("LCE_CHILD_CALLS").is_some();
    if !allow_child && !path.is_empty() {
        static SKIP_LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = SKIP_LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n < 30 {
            _bc(&format!("call_as3_path SKIP path='{}' method='{}'", path, method));
        }
        return 1;
    }
    if let Ok(mut player) = opaque.player.try_lock() {
        let ret = player.call_method_at_path_avm2(&path, &method, ext_args);
        static CALL_LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = CALL_LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n < 30 {
            _bc(&format!("call_as3_path path='{}' method='{}' -> {:?}", path, method, ret));
        }
        1
    } else {
        0
    }
}

fn _invoke_as3_dispatch(
    player: *mut OpaquePlayer,
    name: *const u16, name_len: i32,
    args: *const IggyDataValueRaw, num_args: i32,
) -> i32 {
    let p = AS3_DISPATCH.load(Ordering::SeqCst);
    if p.is_null() { return 0; }
    let f: As3DispatchFn = unsafe { std::mem::transmute(p) };
    f(player, name, name_len, args, num_args)
}

struct IggyExternalBridge {
    player_ptr: *mut OpaquePlayer,
}
// The pointer is only ever read from the player's own tick (single-threaded
// in Ruffle's normal model). If we ever move ticks off-thread we'll need to
// revisit -- for now this is sound because the OpaquePlayer outlives every
// call we'll make through this provider.
unsafe impl Send for IggyExternalBridge {}
unsafe impl Sync for IggyExternalBridge {}

impl ExternalInterfaceProvider for IggyExternalBridge {
    fn call_method(&self, _ctx: &mut UpdateContext<'_>, name: &str, args: &[ExtValue]) -> ExtValue {
        _bc(&format!("EI call '{}' nargs={} dispatch_ready={}", name, args.len(),
            !AS3_DISPATCH.load(Ordering::SeqCst).is_null()));
        // Encode function name as UTF-16. The buffer must outlive the
        // C callback; we hold it here on the stack frame.
        let name_utf16: Vec<u16> = name.encode_utf16().collect();
        // Convert args. Strings need backing storage that outlives the
        // FFI call -- park them in a Vec<Vec<u16>>.
        let mut string_holders: Vec<Vec<u16>> = Vec::with_capacity(args.len());
        let mut iggy_args: Vec<IggyDataValueRaw> = Vec::with_capacity(args.len());
        for arg in args {
            let raw = match arg {
                ExtValue::Undefined => IggyDataValueRaw::undefined(),
                ExtValue::Null => IggyDataValueRaw::null(),
                ExtValue::Bool(b) => IggyDataValueRaw::boolean(*b),
                ExtValue::Number(n) => IggyDataValueRaw::number(*n),
                ExtValue::String(s) => {
                    let utf16: Vec<u16> = s.encode_utf16().chain(std::iter::once(0u16)).collect();
                    let ptr = utf16.as_ptr();
                    let len = (utf16.len() - 1) as i32; // exclude NUL
                    string_holders.push(utf16);
                    IggyDataValueRaw::string_utf16(ptr, len)
                }
                // Object/List flatten to undefined for Phase 4.5 minimum.
                ExtValue::Object(_) | ExtValue::List(_) => IggyDataValueRaw::undefined(),
            };
            iggy_args.push(raw);
        }
        _invoke_as3_dispatch(
            self.player_ptr,
            name_utf16.as_ptr(),
            name_utf16.len() as i32,
            iggy_args.as_ptr(),
            iggy_args.len() as i32,
        );
        // string_holders is dropped here -- after the FFI call returns,
        // any string pointers we passed are no longer valid, which is
        // fine because the C callback is synchronous and must finish
        // reading them before returning.
        drop(string_holders);
        ExtValue::Undefined
    }
    fn on_callback_available(&self, _name: &str) {}
    fn get_id(&self) -> Option<String> { None }
}

// DIAGNOSTIC: each export writes a breadcrumb so we can tell exactly
// where the wgpu DLL is crashing. Strip when bug found.
fn _bc(s: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true)
        .open(r"C:\Users\poopo\Desktop\Projects\LCE\LCE_orig\shim_breadcrumbs.log")
    {
        let _ = writeln!(f, "{}", s);
    }
}

/// Opaque handle returned to C callers in place of `IggyLibrary`. Wraps an
/// `Arc<SwfMovie>` so the SWF can be shared across multiple Player instances
/// (Iggy's "load once, instantiate many" pattern).
pub struct OpaqueLibrary {
    pub movie: Arc<SwfMovie>,
}

/// Opaque handle returned to C callers in place of `Iggy *`. Wraps an
/// `Arc<Mutex<Player>>` so the Ruffle runtime can mutate state on tick
/// while the game holds the raw pointer.
///
/// Phase 4.4(a): also caches the most recently captured framebuffer so the
/// `*const u8` returned by `iggy_open_player_render` stays valid until the
/// next render call or until the player is destroyed.
pub struct OpaquePlayer {
    pub player: Arc<Mutex<Player>>,
    pub last_frame: Option<RgbaImage>,
    pub viewport: (u32, u32),
    /// Last time we advanced one frame, for ready_to_tick pacing. The game
    /// calls `while(IggyPlayerReadyToTick) { tick(); }` and relies on us
    /// to gate frame advancement to the SWF's nominal frame rate. Stock
    /// Iggy compares accumulated wall-clock against `1/swf.frame_rate`.
    pub next_frame_due_us: u64,
}

fn _now_us() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64).unwrap_or(0)
}

/// Returns a NUL-terminated UTF-8 version string for the shim. The pointer
/// is stable across the process lifetime (static storage); the caller must
/// NOT free it.
#[unsafe(no_mangle)]
pub extern "C" fn iggy_open_version() -> *const c_char {
    static VERSION: &[u8] = b"ruffle_iggy_shim 0.2.0 (Phase 4.4(a) wgpu+PS_BLIT)\0";
    VERSION.as_ptr() as *const c_char
}

/// Returns a small u32 magic value (0x4F50454E = "OPEN") to confirm the
/// shim is the one driving `RenderManager` -- callers can probe this
/// after the linker swap in 4.2 to verify they're talking to us and not
/// the proprietary `iggy_w64.lib`.
#[unsafe(no_mangle)]
pub extern "C" fn iggy_open_magic() -> u32 {
    0x4F50454E
}

/// One-shot global init. Mirrors `IggyInit(IggyAllocator *)` from iggy.h.
/// `allocator` is currently ignored (Ruffle uses its own allocator); the
/// 4.3+ implementation can route through it for ABI fidelity.
#[unsafe(no_mangle)]
pub extern "C" fn iggy_open_init(_allocator: *const c_void) {
    _bc("iggy_open_init enter");
    // No-op for Phase 4.1. 4.3 wires the real lifecycle.
    _bc("iggy_open_init exit");
}

/// One-shot global shutdown. Mirrors `IggyShutdown()`.
#[unsafe(no_mangle)]
pub extern "C" fn iggy_open_shutdown() {
    // No-op for Phase 4.1.
}

// ====================================================================== Library
//
// Phase 4.3: SWF blob -> ruffle_core::SwfMovie wrapped in Arc<>, then in our
// `OpaqueLibrary`. Caller (the C++ shim's `IggyLibraryCreateFromMemoryUTF16`)
// passes the raw bytes + a UTF-8 filename. Returns a `*mut OpaqueLibrary`
// the caller must NOT dereference; pass back to `iggy_open_library_destroy`
// when done.

/// Construct a library from raw SWF bytes.
///
/// # Safety
/// `data` must point to `len` bytes of readable memory. `name_utf8` must be a
/// NUL-terminated UTF-8 string OR null. Returns null on parse failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_library_create_from_memory(
    data: *const u8,
    len: usize,
    name_utf8: *const c_char,
) -> *mut OpaqueLibrary {
    _bc(&format!("library_create len={}", len));
    if data.is_null() || len == 0 {
        return std::ptr::null_mut();
    }
    let bytes: Vec<u8> = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
    let name: String = if name_utf8.is_null() {
        String::from("unnamed.swf")
    } else {
        unsafe { CStr::from_ptr(name_utf8) }
            .to_string_lossy()
            .into_owned()
    };
    // SwfMovie::from_data takes (data, url, loader_url). loader_url=None means
    // "no parent SWF" -- standalone load.
    let url = format!("file:///lce/{}", name);
    match SwfMovie::from_data(&bytes, url, None) {
        Ok(movie) => {
            let movie_arc = Arc::new(movie);
            if let Ok(mut g) = LIBRARY_REGISTRY.lock() {
                g.push(movie_arc.clone());
            }
            let lib = OpaqueLibrary { movie: movie_arc };
            Box::into_raw(Box::new(lib))
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy a library previously returned by
/// `iggy_open_library_create_from_memory`. Safe to call with null.
///
/// # Safety
/// `lib` must be either null or a pointer returned by
/// `iggy_open_library_create_from_memory` that has not yet been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_library_destroy(lib: *mut OpaqueLibrary) {
    if lib.is_null() {
        return;
    }
    // Remove from LIBRARY_REGISTRY before drop so subsequently created
    // Players don't keep replaying a stale library. Pointer-equality on
    // Arc inner ptr (not value-equality on SwfMovie -- expensive + maybe
    // not implemented).
    {
        let opaque = unsafe { &*lib };
        let target_ptr = Arc::as_ptr(&opaque.movie);
        if let Ok(mut g) = LIBRARY_REGISTRY.lock() {
            g.retain(|m| !std::ptr::eq(Arc::as_ptr(m), target_ptr));
        }
    }
    drop(unsafe { Box::from_raw(lib) });
}

// ======================================================================= Player
//
// Phase 4.4(a): construct a Ruffle Player with a headless wgpu renderer
// (TextureTarget + offscreen-only). On each `iggy_open_player_render` we run
// `Player::render` which submits commands to wgpu, then
// `WgpuRenderBackend::capture_frame` reads the result back to a CPU
// `RgbaImage`. The C++ side blits that into a D3D11 dynamic texture and
// composites with the existing PS_BLIT path in `4J_Render_open`.
//
// We pin the wgpu backend to DX12 + Vulkan only (skip GL / Metal) and ask
// for `LowPower` since menu SWFs are small (<=720p) and the per-frame
// readback stall on integrated is well under a millisecond.

const WGPU_BACKENDS: wgpu::Backends =
    wgpu::Backends::DX12.union(wgpu::Backends::VULKAN);
const WGPU_POWER_PREFERENCE: wgpu::PowerPreference = wgpu::PowerPreference::LowPower;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_player_create_from_memory(
    data: *const u8,
    len: usize,
) -> *mut OpaquePlayer {
    _bc(&format!("iggy_open_player_create_from_memory enter len={}", len));
    if data.is_null() || len == 0 {
        _bc("  early-null");
        return std::ptr::null_mut();
    }
    let bytes: Vec<u8> = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
    _bc("  bytes copied");
    let movie = match SwfMovie::from_data(&bytes, String::from("file:///lce/inline.swf"), None) {
        Ok(m) => m,
        Err(_) => { _bc("  swf parse FAIL"); return std::ptr::null_mut(); }
    };
    _bc("  swf parsed");
    // Initial viewport = the SWF's declared stage size (Twips -> pixels).
    // The caller will overwrite this every frame via
    // `iggy_open_player_set_viewport` matching whatever Iggy stage size
    // the game derived from its UI layout.
    let init_w = movie.width().to_pixels().max(1.0).round() as u32;
    let init_h = movie.height().to_pixels().max(1.0).round() as u32;
    // Headless wgpu renderer. Failure here means no DX12+Vulkan adapter --
    // very unlikely on the LCE target (Win10+, DX12-capable GPUs). Drop
    // the player on failure so the caller's `IggyPlayerCreateFromMemory`
    // gets a NULL and degrades to the dummy-sentinel path (same as a
    // failed SWF parse), keeping the game alive.
    _bc(&format!("  wgpu about to init ({}x{})", init_w, init_h));
    let backend = match WgpuRenderBackend::for_offscreen(
        (init_w, init_h),
        WGPU_BACKENDS,
        WGPU_POWER_PREFERENCE,
    ) {
        Ok(b) => { _bc("  wgpu OK"); b }
        Err(e) => { _bc(&format!("  wgpu ERR: {:?}", e)); return std::ptr::null_mut(); }
    };
    // Phase 4.5: pre-allocate the OpaquePlayer slot so the
    // ExternalInterface bridge can carry its address before the Player
    // exists. The bridge's player_ptr becomes valid once we ptr::write
    // below.
    use std::mem::MaybeUninit;
    let opaque_storage: Box<MaybeUninit<OpaquePlayer>> = Box::new(MaybeUninit::uninit());
    let opaque_ptr: *mut OpaquePlayer = Box::into_raw(opaque_storage) as *mut OpaquePlayer;
    let bridge = IggyExternalBridge { player_ptr: opaque_ptr };
    let player = PlayerBuilder::new()
        .with_boxed_renderer(Box::new(backend))
        .with_movie(movie)
        .with_autoplay(true)
        .with_external_interface(Box::new(bridge))
        .build();
    // Force transparent stage so the SWF's actual content composites
    // Force transparent stage so the navy back-buffer shows through
    // wherever the SWF hasn't painted. Real per-scene placement now
    // drives the dst rect via BlitRuffleFrameRect, so the previous
    // green diagnostic tint isn't needed for visibility.
    {
        if let Ok(mut p) = player.try_lock() {
            p.set_background_color(Some(Color { r: 0, g: 0, b: 0, a: 0 }));
        }
    }
    // Phase 4.6: apply any fonts the C++ side has installed via
    // iggy_open_install_truetype_utf8 so the SWF's text renders.
    _apply_registered_fonts(&player);
    // Phase 4.7: inject AS3 classes from every previously-loaded Library
    // SWF into this new Player's root ApplicationDomain so PlaceByClass
    // refs (e.g. MainMenu's fourj.Buttons.FJ_MenuButton_Normal -> skinHD)
    // resolve.
    _inject_library_abcs(&player);
    _bc("  player built");
    unsafe {
        std::ptr::write(opaque_ptr, OpaquePlayer {
            player,
            last_frame: None,
            viewport: (init_w, init_h),
            next_frame_due_us: _now_us(),
        });
    }
    opaque_ptr
}

/// Destroy a player handle.
///
/// # Safety
/// `p` must be null or a valid handle from
/// `iggy_open_player_create_from_memory` not yet destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_player_destroy(p: *mut OpaquePlayer) {
    if p.is_null() {
        return;
    }
    _bc(&format!("player_destroy p={:p}", p));
    drop(unsafe { Box::from_raw(p) });
    _bc("player_destroy done");
}

/// Run one frame of the player. No-op if handle is null.
///
/// # Safety
/// Same as destroy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_player_tick(p: *mut OpaquePlayer) {
    if p.is_null() {
        return;
    }
    let opaque = unsafe { &mut *p };
    // Compute frame interval from the SWF's nominal frame rate. Fall back
    // to 30 fps if the rate is implausible.
    let fps = {
        if let Ok(player) = opaque.player.try_lock() {
            let f = player.frame_rate();
            if f.is_finite() && f > 1.0 && f < 240.0 { f } else { 30.0 }
        } else { 30.0 }
    };
    let interval_us = (1_000_000.0 / fps) as u64;
    // Advance the "next frame due" gate first so a slow tick still
    // produces stable cadence (rather than drifting forward).
    let now = _now_us();
    opaque.next_frame_due_us = if now > opaque.next_frame_due_us + 4 * interval_us {
        now + interval_us // we fell badly behind; resync
    } else {
        opaque.next_frame_due_us + interval_us
    };
    if let Ok(mut player) = opaque.player.try_lock() {
        player.run_frame();
    }
}

/// Returns 1 if the player has a frame ready to run, 0 otherwise.
/// For Phase 4.4 we always say "ready" -- real frame-time scheduling is
/// 4.4(a) work.
///
/// # Safety
/// Same as destroy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_player_ready_to_tick(p: *mut OpaquePlayer) -> i32 {
    if p.is_null() {
        return 0;
    }
    let opaque = unsafe { &*p };
    if _now_us() >= opaque.next_frame_due_us { 1 } else { 0 }
}

// ================================================================= Render output
//
// Phase 4.4(a) FFI pair. The C++ shim forwards Iggy's per-frame
// `IggyPlayerSetDisplaySize` to `iggy_open_player_set_viewport` and
// `IggyPlayerDraw` to `iggy_open_player_render`. The latter returns a CPU
// pointer to the freshly captured BGRA-on-the-wire-but-RGBA-in-byte-order
// (i.e. `RgbaImage::as_raw()`) framebuffer; the buffer is owned by the
// `OpaquePlayer` and stays valid until the next render call or until the
// player is destroyed.

/// Set the Ruffle player's viewport (and the underlying wgpu
/// TextureTarget's size). Idempotent on repeated calls with the same size.
/// Called every frame from `IggyPlayerSetDisplaySize` so the framebuffer
/// always matches the UI scene's intended draw extent.
///
/// # Safety
/// `p` must be null or a valid handle from
/// `iggy_open_player_create_from_memory` not yet destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_player_set_viewport(
    p: *mut OpaquePlayer,
    w: u32,
    h: u32,
) {
    if p.is_null() || w == 0 || h == 0 {
        return;
    }
    _bc(&format!("set_viewport p={:p} {}x{}", p, w, h));
    let opaque = unsafe { &mut *p };
    if opaque.viewport == (w, h) {
        return;
    }
    if let Ok(mut player) = opaque.player.try_lock() {
        player.set_viewport_dimensions(ViewportDimensions {
            width: w,
            height: h,
            scale_factor: 1.0,
        });
        opaque.viewport = (w, h);
        // The cached frame's size no longer matches the new viewport;
        // drop it so the next render call returns a freshly sized buffer.
        opaque.last_frame = None;
    }
}

/// Render one frame and capture it as a CPU RGBA8 buffer. Returns a
/// pointer to the buffer (owned by the player handle) on success or null
/// on any failure (lock contention, no captured frame). `*out_w` /
/// `*out_h` receive the buffer dimensions when non-null; the row stride
/// is always `w * 4` (contiguous) since `image::RgbaImage` is densely
/// packed.
///
/// Pointer lifetime: valid until the next call to
/// `iggy_open_player_render`, `iggy_open_player_set_viewport`, or
/// `iggy_open_player_destroy` on the same handle.
///
/// # Safety
/// `p` must be a valid `OpaquePlayer` handle. `out_w` / `out_h` may be
/// null or must each point to a writable `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn iggy_open_player_render(
    p: *mut OpaquePlayer,
    out_w: *mut u32,
    out_h: *mut u32,
) -> *const u8 {
    if p.is_null() {
        return std::ptr::null();
    }
    static RENDER_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = RENDER_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if n < 60 {
        _bc(&format!("RENDER p={:p}", p));
    }
    let opaque = unsafe { &mut *p };
    let mut player = match opaque.player.try_lock() {
        Ok(g) => g,
        Err(_) => { _bc("  render lock-fail"); return std::ptr::null(); }
    };
    _bc("  render locked, about to render()");
    // Drive the rasteriser: this calls renderer.submit_frame which fills
    // the TextureTarget's GPU buffer. Side-effects of run_frame already
    // happened during the prior IggyPlayerTickRS call.
    player.render();
    _bc("  render() returned");
    // Cross-version downcast: the renderer trait object is opaque, so we
    // ask for the concrete WgpuRenderBackend<TextureTarget> back. This
    // matches Ruffle's own test-harness pattern in
    // tests/tests/environment.rs (NativeRenderInterface::capture).
    // Recover the concrete WgpuRenderBackend<TextureTarget> from the
    // erased `&mut dyn RenderBackend` via `Any::downcast_mut`. Mirrors
    // the test harness in `tests/tests/environment.rs:121-126`. This
    // works because `RenderBackend: Any` (declared at
    // `render/src/backend.rs:28`).
    let renderer_any: &mut dyn Any = player.renderer_mut() as &mut dyn Any;
    let captured: Option<RgbaImage> = renderer_any
        .downcast_mut::<WgpuRenderBackend<ruffle_render_wgpu::target::TextureTarget>>()
        .and_then(|backend| backend.capture_frame());
    drop(player);
    match captured {
        Some(img) => {
            // DIAGNOSTIC: sample center pixel + count nonzero alpha so we
            // can tell if Ruffle actually rasterised content or just
            // returned a transparent texture.
            use std::sync::atomic::{AtomicU32, Ordering};
            static FRAMES_LOGGED: AtomicU32 = AtomicU32::new(0);
            let n = FRAMES_LOGGED.fetch_add(1, Ordering::Relaxed);
            if n < 10 {
                let raw = img.as_raw();
                let pixels = raw.len() / 4;
                let mut nonzero_alpha = 0usize;
                let mut nonzero_rgb = 0usize;
                for px in raw.chunks_exact(4) {
                    if px[3] != 0 { nonzero_alpha += 1; }
                    if px[0] != 0 || px[1] != 0 || px[2] != 0 { nonzero_rgb += 1; }
                }
                let cx = img.width() as usize / 2;
                let cy = img.height() as usize / 2;
                let off = (cy * img.width() as usize + cx) * 4;
                _bc(&format!(
                    "FRAME{} {}x{} bytes={} nonzero_alpha={}/{} nonzero_rgb={} center=[{},{},{},{}]",
                    n, img.width(), img.height(), raw.len(),
                    nonzero_alpha, pixels, nonzero_rgb,
                    raw[off], raw[off+1], raw[off+2], raw[off+3]
                ));
            }
            if !out_w.is_null() { unsafe { *out_w = img.width(); } }
            if !out_h.is_null() { unsafe { *out_h = img.height(); } }
            let ptr = img.as_raw().as_ptr();
            opaque.last_frame = Some(img);
            ptr
        }
        None => std::ptr::null(),
    }
}
