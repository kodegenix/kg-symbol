use super::*;

use std::sync::atomic::{AtomicU32, Ordering};
use std::hash::{Hash, Hasher};
use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::ptr::NonNull;


pub(super) struct AtomString(NonNull<u8>);

impl AtomString {
    pub(super) fn new<S: AsRef<str>>(value: S) -> AtomString {
        let s = value.as_ref();
        let (layout, offset) = unsafe {
            Layout::new::<Header>().extend(Layout::from_size_align_unchecked(s.len(), 1)).unwrap()
        };
        let data = unsafe {
            let data = alloc(layout);
            if data.is_null() {
                handle_alloc_error(layout);
            }
            let data_ptr = data.offset(offset as isize);
            let mut hdr_ptr = std::mem::transmute::<*mut u8, &mut Header>(data);
            *hdr_ptr = Header {
                ref_count: AtomicU32::new(0),
                str_ptr: data_ptr,
                str_len: s.len(),
            };
            std::ptr::copy_nonoverlapping(s.as_ptr(), data_ptr, s.len());
            NonNull::new_unchecked(data)
        };
        AtomString(data)
    }

    pub(super) fn empty() -> AtomString {
        let layout = Layout::new::<Header>();
        let data = unsafe {
            let data = alloc(layout);
            if data.is_null() {
                handle_alloc_error(layout);
            }
            let mut hdr_ptr = std::mem::transmute::<*mut u8, &mut Header>(data);
            *hdr_ptr = Header {
                ref_count: AtomicU32::new(1),
                str_ptr: NonNull::dangling(),
                str_len: 0,
            };
            NonNull::new_unchecked(data)
        };
        AtomString(data)
    }

    fn inc_ref_count(&self) {
        self.header().ref_count.fetch_add(1, Ordering::SeqCst);
    }

    fn dec_ref_count(&self) -> u32 {
        self.header().ref_count.fetch_sub(1, Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(super) fn ref_count(&self) -> u32 {
        self.header().ref_count.load(Ordering::SeqCst)
    }

    #[inline(always)]
    fn header(&self) -> &Header {
        unsafe { std::mem::transmute::<NonNull<u8>, &Header>(self.0) }
    }
}

impl Drop for AtomString {
    fn drop(&mut self) {
        let h = self.header();
        if h.dec_ref_count() <= 1 {
            //remove entry
            let len = self.header().str_len;
            let (layout, _) = unsafe {
                Layout::new::<Header>().extend(Layout::from_size_align_unchecked(len, 1)).unwrap()
            };
            unsafe {
                dealloc(self.0, layout);
            }
        }
    }
}

impl PartialEq for AtomString {
    fn eq(&self, other: &AtomString) -> bool {
        self.0 == other.0
    }
}

impl Eq for AtomString {}

impl Hash for AtomString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl AsRef<str> for AtomString {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl std::borrow::Borrow<str> for AtomString {
    fn borrow(&self) -> &str {
        self.value.as_ref()
    }
}

impl heapsize::HeapSizeOf for AtomString {
    fn heap_size_of_children(&self) -> usize {
        self.value.heap_size_of_children()
    }
}

unsafe impl Send for AtomString {}

unsafe impl Sync for AtomString {}


struct Header {
    ref_count: AtomicU32,
    str_ptr: NonNull<u8>,
    str_len: usize,
}

impl AsRef<str> for Header {
    fn as_ref(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.str_ptr, self.str_len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atom_string_hash_eq_str_hash() {
        use std::collections::hash_map::DefaultHasher;

        let s1 = "example string";
        let h1 = {
            let mut hasher = DefaultHasher::new();
            s1.hash(&mut hasher);
            hasher.finish()
        };

        let s2 = AtomString::new(s1);
        let h2 = {
            let mut hasher = DefaultHasher::new();
            s2.hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(h1, h2);
    }
}
