// SPDX-License-Identifier: Apache-2.0

//! Verifies that CBOR deserialization returns an error instead of aborting on OOM.

#![cfg(feature = "std")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use ciborium::{
    de::{from_reader, Error},
    value::Value,
};

struct BoundedAllocator {
    limit: AtomicUsize,
    allocated: AtomicUsize,
}

impl BoundedAllocator {
    const fn new() -> Self {
        Self {
            limit: AtomicUsize::new(usize::MAX),
            allocated: AtomicUsize::new(0),
        }
    }

    fn set_limit(&self, limit: usize) {
        self.allocated.store(0, Ordering::SeqCst);
        self.limit.store(limit, Ordering::SeqCst);
    }

    fn clear_limit(&self) {
        self.limit.store(usize::MAX, Ordering::SeqCst);
    }
}

// SAFETY: Delegates directly to `std::alloc::System` while returning null when the configured
// test allocation budget is exceeded.
unsafe impl GlobalAlloc for BoundedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let limit = self.limit.load(Ordering::SeqCst);
        if limit != usize::MAX {
            let prev = self.allocated.fetch_add(layout.size(), Ordering::SeqCst);
            if prev.saturating_add(layout.size()) > limit {
                self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
                return std::ptr::null_mut();
            }
        }
        // SAFETY: Forwarded directly to `System::alloc` with identical `layout`.
        let ptr = unsafe { System.alloc(layout) };
        if ptr.is_null() && limit != usize::MAX {
            self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.limit.load(Ordering::SeqCst) != usize::MAX {
            let _ = self
                .allocated
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |cur| {
                    Some(cur.saturating_sub(layout.size()))
                });
        }
        // SAFETY: Forwarded directly to `System::dealloc` with identical `ptr` and `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: BoundedAllocator = BoundedAllocator::new();

struct LimitGuard;

impl LimitGuard {
    fn new(limit: usize) -> Self {
        ALLOCATOR.set_limit(limit);
        Self
    }
}

impl Drop for LimitGuard {
    fn drop(&mut self) {
        ALLOCATOR.clear_limit();
    }
}

#[test]
fn deserialization_returns_error_on_oom() {
    // Prepare inputs while allocator is unbounded.
    // 1) 16 KiB byte string (exceeds 4 KiB scratch buffer -> deserialize_byte_buf).
    let mut big_bstr = vec![0x59, 0x40, 0x00];
    big_bstr.resize(3 + 16 * 1024, 0xaa);

    // 2) 16 KiB text string (exceeds 4 KiB scratch buffer -> deserialize_string).
    let mut big_tstr = vec![0x79, 0x40, 0x00];
    big_tstr.resize(3 + 16 * 1024, b'a');

    // 3) 4094-null indefinite array (hits try_reserve in Value::visit_seq).
    let mut wide_array = vec![0x9f];
    wide_array.resize(1 + 4094, 0xf6);
    wide_array.push(0xff);

    // 4) Definite map claiming 1000 pairs (hits try_reserve(hint) in Value::visit_map).
    let wide_map = [0xb9, 0x03, 0xe8];

    // 5) 2 KiB byte string (fits in 4 KiB scratch buffer -> Value::visit_bytes).
    let mut scratch_bstr = vec![0x59, 0x08, 0x00];
    scratch_bstr.resize(3 + 2048, 0xbb);

    let cases: [(&str, &[u8], usize); 5] = [
        ("big_bstr", &big_bstr, 4096),
        ("big_tstr", &big_tstr, 4096),
        ("wide_array", &wide_array, 4096),
        ("wide_map", &wide_map, 4096),
        ("scratch_bstr", &scratch_bstr, 512),
    ];

    for (name, payload, budget) in cases {
        let res = {
            let _guard = LimitGuard::new(budget);
            from_reader::<Value, _>(payload)
        };
        assert!(
            matches!(res, Err(Error::Semantic(..))),
            "expected semantic OOM error for {name}, got: {res:?}"
        );
    }
}
