use std::cell::Cell;
use std::ops::Deref;

use gc_arena::Collect;
use ruffle_wstr::{panic_on_invalid_length, ptr as wptr, wstr_impl_traits, WStr, WString};

use crate::string::avm_string::AvmString;

/// Internal representation of `AvmAtom`s and (owned) `AvmString`.
///
/// Using this type directly is dangerous, as it can be used to violate
/// the interning invariants.
#[derive(Collect)]
#[collect(unsafe_drop)]
pub struct AvmStringRepr<'gc> {
    #[collect(require_static)]
    ptr: *mut (),

    // Length and is_wide bit.
    #[collect(require_static)]
    meta: wptr::WStrMetadata,

    // We abuse WStrMetadata to store capacity and is_interned bit.
    // If a string is Dependent, the capacity should always be 0.
    #[collect(require_static)]
    capacity: Cell<wptr::WStrMetadata>,

    // If a string is Dependent, this should always be 0.
    // If a string is Owned, this indicates used chars, including dependents.
    // Example: assume a string a="abc" has 10 bytes of capacity (chars_used=3).
    // Then, with a+"d", we produce a dependent string and owner's chars_used becomes 4.
    // len <= chars_used <= capacity.
    #[collect(require_static)]
    chars_used: Cell<u32>,

    // If Some, the string is dependent. The owner is assumed to be non-dynamic.
    owner: Option<AvmString<'gc>>,
}

impl<'gc> AvmStringRepr<'gc> {
    pub fn from_raw(s: WString, interned: bool) -> Self {
        let (ptr, meta, cap) = s.into_raw_parts();
        let capacity = Cell::new(wptr::WStrMetadata::new32(cap, interned));
        Self {
            ptr,
            meta,
            capacity,
            chars_used: Cell::new(meta.len32()),
            owner: None,
        }
    }

    pub fn new_dependent(s: AvmString<'gc>, start: usize, end: usize) -> Self {
        let wstr = &s[start..end];
        let wstr_ptr = wstr as *const WStr;

        let meta = unsafe { wptr::WStrMetadata::of(wstr_ptr) };
        // Dependent strings are never interned
        let capacity = Cell::new(wptr::WStrMetadata::new32(0, false));
        let ptr = wstr_ptr as *mut WStr as *mut ();

        let owner = if let Some(owner) = s.owner() {
            owner
        } else {
            s
        };

        Self {
            owner: Some(owner),
            ptr,
            meta,
            chars_used: Cell::new(0),
            capacity,
        }
    }

    unsafe fn new_dependent_raw(
        owner: AvmString<'gc>,
        ptr: *const u8,
        length: u32,
        is_wide: bool,
    ) -> Self {
        let meta = wptr::WStrMetadata::new32(length, is_wide);
        // Dependent strings are never interned
        let capacity = Cell::new(wptr::WStrMetadata::new32(0, false));
        let ptr = ptr as *mut ();

        Self {
            owner: Some(owner),
            ptr,
            meta,
            chars_used: Cell::new(0),
            capacity,
        }
    }

    pub fn try_append_inline(left: AvmString<'gc>, right: &WStr) -> Option<Self> {
        // note: we could also in-place append a byte string to a wide string
        // But it was skipped for now.
        if left.is_wide() != right.is_wide() {
            return None;
        }

        let left_origin_s = left.owner().unwrap_or(left);
        if let (Some(left), Some(left_origin)) = (left.as_managed(), left_origin_s.as_managed()) {
            let char_size = if left.is_wide() { 2 } else { 1 };
            /*
                assumptions:
                - left.len <= left.chars_used <= left.capacity
                - left_ptr is inside left_origin_ptr .. left_origin_ptr + left.chars_used

                note: it's possible that left == left_origin.
            */
            unsafe {
                let left_origin_ptr = left_origin.ptr as *const u8;
                let left_ptr = left.ptr as *const u8;

                /*
                Assume a="abc", b=a+"d", c=a.substr(1), we're running d=c+"e"

                a          ->  abc
                b          ->  abcd
                c          ->   bc        v left_capacity_end
                a's memory ->  abcd_______
                                  ^ first_requested
                                   ^ first_available

                We can only append in-place if first_requested and first_available match
                And we have enough spare capacity.
                */

                let first_available =
                    left_origin_ptr.add(char_size * left_origin.chars_used.get() as usize);
                let first_requested = left_ptr.add(char_size * left.len());

                let mut chars_available = 0;
                if first_available == first_requested {
                    let left_capacity_end =
                        left_origin_ptr.add(char_size * left_origin.capacity.get().len());
                    chars_available =
                        ((left_capacity_end as usize) - (first_available as usize)) / char_size;
                }
                if chars_available >= right.len() {
                    let first_available = first_available as *mut u8;
                    let right_ptr = right as *const WStr as *const () as *const u8;
                    std::ptr::copy_nonoverlapping(
                        right_ptr,
                        first_available,
                        char_size * right.len(),
                    );

                    let new_chars_used: usize = left_origin.chars_used.get() as usize + right.len();
                    if new_chars_used >= u32::MAX as usize {
                        // This isn't really about the string length,
                        // but it's close enough?
                        panic_on_invalid_length(new_chars_used);
                    }
                    left_origin.chars_used.set(new_chars_used as u32);

                    let new_len = left.len() + right.len();
                    if new_len > WStr::MAX_LEN {
                        panic_on_invalid_length(new_len);
                    }

                    let ret = Self::new_dependent_raw(
                        left_origin_s,
                        left_ptr,
                        new_len as u32,
                        left.is_wide(),
                    );
                    return Some(ret);
                }
            }
        }
        None
    }

    #[inline]
    pub fn is_dependent(&self) -> bool {
        self.owner.is_some()
    }

    #[inline]
    pub fn owner(&self) -> Option<AvmString<'gc>> {
        self.owner
    }

    #[inline]
    pub fn as_wstr(&self) -> &WStr {
        // SAFETY: we own a `WString`.
        unsafe { &*wptr::from_raw_parts(self.ptr, self.meta) }
    }

    pub fn is_interned(&self) -> bool {
        self.capacity.get().is_wide()
    }

    pub fn mark_interned(&self) {
        if self.is_dependent() {
            panic!("bug: we interned a dependent string");
        }
        let cap = self.capacity.get();
        let new_cap = wptr::WStrMetadata::new32(cap.len32(), true);
        self.capacity.set(new_cap);
    }
}

impl<'gc> Drop for AvmStringRepr<'gc> {
    fn drop(&mut self) {
        if self.owner.is_none() {
            // SAFETY: we drop the `WString` we logically own.
            unsafe {
                let cap = self.capacity.get().len32();
                let _ = WString::from_raw_parts(self.ptr, self.meta, cap);
            }
        }
    }
}

impl<'gc> Deref for AvmStringRepr<'gc> {
    type Target = WStr;
    #[inline]
    fn deref(&self) -> &WStr {
        self.as_wstr()
    }
}

impl<'gc> Default for AvmStringRepr<'gc> {
    #[inline]
    fn default() -> Self {
        Self::from_raw(WString::new(), false)
    }
}

wstr_impl_traits!(impl['gc] for AvmStringRepr<'gc>);
