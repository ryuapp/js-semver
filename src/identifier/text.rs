#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use core::fmt;
use core::mem::size_of;
use core::ptr;

const INLINE_CAPACITY: usize = size_of::<*mut u8>();
const INLINE_TAG: usize = 1 << (usize::BITS - 1);

// Fix the storage size to one pointer word; this union is not an FFI type.
#[repr(C)]
union Storage {
    inline: [u8; INLINE_CAPACITY],
    heap: *mut u8,
}

// Exactly two words: short strings occupy the pointer word, while long strings
// use it as an owned heap pointer. Heap allocations cannot have the high length
// bit set, so that bit identifies the inline representation.
pub(super) struct IdentifierText {
    storage: Storage,
    len_and_tag: usize,
}

// SAFETY: The heap pointer is exclusively owned, and shared access exposes
// only immutable string contents. This has the same thread-safety as Box<str>.
unsafe impl Send for IdentifierText {}
// SAFETY: IdentifierText has no interior mutation, and as_str returns only
// shared references to valid string bytes.
unsafe impl Sync for IdentifierText {}

impl IdentifierText {
    pub(super) fn new(s: &str) -> Self {
        if s.len() <= INLINE_CAPACITY {
            let mut inline = [0; INLINE_CAPACITY];
            inline[..s.len()].copy_from_slice(s.as_bytes());
            Self {
                storage: Storage { inline },
                len_and_tag: s.len() | INLINE_TAG,
            }
        } else {
            let boxed = Box::<str>::from(s);
            let len = boxed.len();
            let heap = Box::into_raw(boxed).cast::<u8>();
            Self {
                storage: Storage { heap },
                len_and_tag: len,
            }
        }
    }

    fn is_inline(&self) -> bool {
        self.len_and_tag & INLINE_TAG != 0
    }

    pub(super) fn as_str(&self) -> &str {
        let bytes = if self.is_inline() {
            // SAFETY: The tag selects the initialized inline field, whose
            // length is at most INLINE_CAPACITY.
            unsafe { &self.storage.inline[..self.len_and_tag & !INLINE_TAG] }
        } else {
            // SAFETY: The heap field owns a Box<str> allocation of this length.
            let heap = unsafe { self.storage.heap };
            // SAFETY: The allocation is live for the lifetime of self.
            unsafe { core::slice::from_raw_parts(heap, self.len_and_tag) }
        };
        // SAFETY: Both representations contain bytes copied from a valid str.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
}

impl Default for IdentifierText {
    fn default() -> Self {
        Self::new("")
    }
}

impl Clone for IdentifierText {
    fn clone(&self) -> Self {
        Self::new(self.as_str())
    }
}

impl Drop for IdentifierText {
    fn drop(&mut self) {
        if !self.is_inline() {
            // SAFETY: The heap field was produced by Box::into_raw and this
            // instance is its sole owner.
            let heap = unsafe { self.storage.heap };
            let slice = ptr::slice_from_raw_parts_mut(heap, self.len_and_tag);
            // SAFETY: Box<[u8]> has the same allocation layout as Box<str>.
            // Reconstructing it here releases the allocation exactly once.
            unsafe { drop(Box::from_raw(slice)) }
        }
    }
}

impl fmt::Debug for IdentifierText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl PartialEq for IdentifierText {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for IdentifierText {}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::boxed::Box;

    use super::*;

    #[test]
    fn storage_boundary_and_clone() {
        assert_eq!(size_of::<IdentifierText>(), size_of::<Box<str>>());
        for len in [
            1,
            INLINE_CAPACITY - 1,
            INLINE_CAPACITY,
            INLINE_CAPACITY + 1,
            32,
        ] {
            let value = "a".repeat(len);
            let original = IdentifierText::new(&value);
            let cloned = original.clone();
            assert_eq!(original.as_str(), value);
            assert_eq!(original.is_inline(), len <= INLINE_CAPACITY);
            drop(original);
            assert_eq!(cloned.as_str(), value);
        }
    }

    #[test]
    fn storage_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<IdentifierText>();
    }
}
