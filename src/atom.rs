use super::*;

use std::sync::atomic::{AtomicU32, Ordering};
use std::hash::{Hash, Hasher};


#[derive(Debug)]
pub (super) struct AtomString {
    value: Box<str>,
    ref_count: AtomicU32,
}

impl AtomString {
    pub (super) fn new<S: Into<String>>(value: S) -> AtomString {
        AtomString {
            value: value.into().into_boxed_str(),
            ref_count: AtomicU32::new(0),
        }
    }

    pub (super) fn empty() -> AtomString {
        AtomString {
            value: String::new().into_boxed_str(),
            ref_count: AtomicU32::new(1),
        }
    }

    pub (super) fn inc_ref_count(&self) {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
    }

    pub (super) fn dec_ref_count(&self) {
        if self.ref_count.fetch_sub(1, Ordering::SeqCst) == 1 {
            remove_entry(self)
        }
    }

    #[cfg(test)]
    pub (super) fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::SeqCst)
    }
}

impl PartialEq for AtomString {
    fn eq(&self, other: &AtomString) -> bool {
        self as *const AtomString == other as *const AtomString
    }
}

impl Eq for AtomString {}

impl Hash for AtomString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
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

impl std::borrow::Borrow<str> for Box<AtomString> {
    fn borrow(&self) -> &str {
        self.as_ref().value.as_ref()
    }
}

impl heapsize::HeapSizeOf for AtomString {
    fn heap_size_of_children(&self) -> usize {
        self.value.heap_size_of_children()
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
