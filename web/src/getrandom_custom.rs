//! wasm64 `getrandom` custom backend — REMOVE WHEN UPSTREAM SHIPS wasm64 SUPPORT.
//!
//! `getrandom` 0.3.4's `wasm_js` backend is cfg-gated to `target_arch = "wasm32"`
//! only. Upstream PR #848 widens it to wasm64, but lives in the 0.4.x series,
//! while `rand 0.9` (pulled by `ruffle_core`) pins getrandom 0.3.x. Until a 0.3.x
//! release ships wasm64, we build with `--cfg getrandom_backend="custom"` and
//! provide the entropy source here.
//!
//! This routes to the Web Crypto API (`crypto.getRandomValues`) — the same
//! source the upstream `wasm_js` backend uses — so it is cryptographically
//! sound, not a placeholder. `crypto` is a global in both window and worker
//! scopes.

use getrandom::Error;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = crypto, catch)]
    fn getRandomValues(array: &Uint8Array) -> Result<(), JsValue>;
}

#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), Error> {
    // Web Crypto rejects buffers larger than 65536 bytes per call.
    const MAX_CHUNK: usize = 65536;
    let mut offset = 0usize;
    while offset < len {
        let chunk = core::cmp::min(len - offset, MAX_CHUNK);
        let buf = Uint8Array::new_with_length(chunk as u32);
        getRandomValues(&buf).map_err(|_| Error::UNEXPECTED)?;
        // SAFETY: the caller guarantees `dest` is valid for `len` bytes, so the
        // `[offset, offset + chunk)` sub-range is in bounds.
        let out = unsafe { core::slice::from_raw_parts_mut(dest.add(offset), chunk) };
        buf.copy_to(out);
        offset += chunk;
    }
    Ok(())
}
