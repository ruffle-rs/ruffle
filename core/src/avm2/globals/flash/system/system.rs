//! `flash.system.System` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;

/// Implements `flash.system.System.setClipboard` method
pub fn set_clipboard<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // The following restrictions only apply to the plugin.
    // TODO: Check the type of event that triggered the function call.
    #[cfg(target_family = "wasm")]
    if false {
        return Err(crate::avm2::error::make_error_2176(activation));
    }

    let new_content = args.get_string_non_null(activation, 0, "text")?;
    activation
        .context
        .ui
        .set_clipboard_content(new_content.to_string());

    Ok(Value::Undefined)
}

// ---------------------------------------------------------------------------
// Memory introspection helpers (WASM-only, return 0 on native targets).
//
// The three `flash.system.System` memory getters
// (`totalMemoryNumber`, `freeMemory`, `privateMemory`) are derived from
// two primitive measurements of the running WASM module:
//
//   * `wasm_linear_memory_bytes()` — the total size of the module's
//     linear memory at this moment (grows monotonically via
//     `memory.grow`; the WASM MVP has no shrink). This is the entire
//     address space the module has requested from the host browser.
//
//   * `heap_base()` — the link-time offset where the heap region
//     starts, exposed by wasm-ld as the `__heap_base` symbol.
//     Everything below this offset is the fixed boot cost of the
//     module (static data + initial stack). Everything at or above
//     this offset is dynamic heap, served by the tilemalloc pools and
//     the fallback dlmalloc.
//
// From these two values plus the tilemalloc live-byte counters we
// compute the three Adobe-AS3 properties. On non-WASM targets both
// primitives return 0 and the three getters degrade to 0 — the
// tilemalloc isn't the global allocator on desktop builds anyway, so
// the counters wouldn't carry useful information there.

/// WASM linear memory page size as fixed by the WASM specification.
/// Same value on wasm32 and wasm64.
#[cfg(target_family = "wasm")]
const WASM_PAGE_SIZE: usize = 65536;

/// Offset of the start of the heap region within the WASM linear
/// memory. `__heap_base` is a link-time constant emitted by wasm-ld;
/// the bytes in `[0, __heap_base)` are statics + initial stack (fixed
/// at module boot), the bytes in `[__heap_base, linear_memory_size)`
/// are the dynamic heap actually consumed by tilemalloc and dlmalloc.
#[cfg(target_family = "wasm")]
fn heap_base() -> usize {
    unsafe extern "C" {
        static __heap_base: u8;
    }
    // SAFETY: We never dereference the symbol — we only read its
    // address. `__heap_base` is a link-time constant placed by wasm-ld
    // at a valid offset inside the module's linear memory.
    unsafe { &__heap_base as *const u8 as usize }
}

#[cfg(not(target_family = "wasm"))]
fn heap_base() -> usize {
    0
}

/// Total size of the WASM linear memory in bytes at this moment.
/// Monotonically non-decreasing.
//
// `wasm32::memory_size` is stable since Rust 1.33; `wasm64::memory_size`
// lives behind the nightly `simd_wasm64` feature gate (tracking #90599)
// — enabled here for the wasm64 build, which uses `RUSTC_BOOTSTRAP=1`.
#[cfg(target_arch = "wasm32")]
fn wasm_linear_memory_bytes() -> usize {
    core::arch::wasm32::memory_size(0) * WASM_PAGE_SIZE
}

#[cfg(target_arch = "wasm64")]
fn wasm_linear_memory_bytes() -> usize {
    core::arch::wasm64::memory_size(0) * WASM_PAGE_SIZE
}

#[cfg(not(target_family = "wasm"))]
fn wasm_linear_memory_bytes() -> usize {
    0
}

/// Bytes of dynamic heap currently held by the module — the portion
/// of the linear memory that lives at or above `__heap_base`. This is
/// the memory the player has actually requested from the host at
/// runtime (`memory.grow` calls triggered by the tilemalloc or the
/// dlmalloc fallback), after subtracting the fixed boot cost.
fn dynamic_heap_bytes() -> usize {
    wasm_linear_memory_bytes().saturating_sub(heap_base())
}

/// Implements `flash.system.System.totalMemoryNumber` getter.
///
/// Adobe defines `totalMemoryNumber` as "the amount of memory
/// currently in use that has been directly allocated by Flash Player
/// or AIR". On Ruffle WASM this maps to the dynamic heap region of the
/// linear memory: every byte at or above `__heap_base` is either held
/// by a tilemalloc pool extent, by a live dlmalloc allocation, or by
/// dlmalloc free-list headroom — all of which were obtained through
/// `memory.grow` calls driven by the player's allocations.
///
/// Pairs with `freeMemory` (the not-in-use share of this number).
pub fn get_total_memory_number<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Number(dynamic_heap_bytes() as f64))
}

/// Implements `flash.system.System.freeMemory` getter.
///
/// Adobe defines `freeMemory` as "the amount of memory that is
/// allocated to Flash Player and that is not in use. This unused
/// portion of allocated memory fluctuates as garbage collection takes
/// place. Use this property to monitor garbage collection."
///
/// On Ruffle WASM the "in use" share is the sum of live tilemalloc
/// bytes (`total_tile_alive_bytes`) and live fallback dlmalloc bytes
/// (`fallback.bytes_alive`); subtracting from `totalMemoryNumber`
/// gives the bytes inside the dynamic heap that no live allocation
/// currently holds — pool free lists, dlmalloc free lists, and any
/// linear-memory headroom past the high-water mark. After a
/// `System.gc()` call drains the AS3 arena and the backing buffers
/// are released, this value rises accordingly.
pub fn get_free_memory<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let stats = crate::tilemalloc::GLOBAL_TILEMALLOC.snapshot_stats();
    let in_use = stats.total_tile_alive_bytes() + stats.fallback.bytes_alive;
    let free = dynamic_heap_bytes().saturating_sub(in_use);
    Ok(Value::Number(free as f64))
}

/// Implements `flash.system.System.privateMemory` getter.
///
/// Adobe defines `privateMemory` as "the entire amount of memory used
/// by an application. This is the amount of resident private memory
/// for the entire process." On Ruffle WASM the closest measurable
/// analogue is the total size of the module's linear memory: it
/// includes everything `totalMemoryNumber` covers plus the fixed boot
/// cost (statics + initial stack). The JS heap of the wrapping host
/// page is not included here; integrating `performance.memory`
/// (Chrome-only) is a separate iteration.
pub fn get_private_memory<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Number(wasm_linear_memory_bytes() as f64))
}
