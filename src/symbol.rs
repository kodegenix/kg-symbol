use super::*;

use std::borrow::{Cow, Borrow};
use std::cmp::Ordering;
use std::ptr::NonNull;


pub struct Symbol(NonNull<AtomString>);

impl Symbol {
    pub (super) fn new(e: &Box<AtomString>) -> Symbol {
        e.inc_ref_count();
        Self::wrap(e)
    }

    pub (super) fn wrap(e: &Box<AtomString>) -> Symbol {
        unsafe {
            Symbol(NonNull::new_unchecked(std::mem::transmute(e.as_ref())))
        }
    }

    fn atom(&self) -> &AtomString {
        unsafe {
            self.0.as_ref()
        }
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        self.atom().dec_ref_count();
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl Eq for Symbol {}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            Ordering::Equal
        } else {
            self.atom().as_ref().cmp(&other.atom().as_ref())
        }
    }
}

impl PartialEq<String> for Symbol {
    fn eq(&self, other: &String) -> bool {
        self.atom().as_ref() == other
    }
}

impl PartialOrd<String> for Symbol {
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        self.atom().as_ref().partial_cmp(other)
    }
}

impl PartialEq<str> for Symbol {
    fn eq(&self, other: &str) -> bool {
        self.atom().as_ref() == other
    }
}

impl PartialOrd<str> for Symbol {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.atom().as_ref().partial_cmp(other)
    }
}

impl<'a> PartialEq<&'a str> for Symbol {
    fn eq(&self, other: &&'a str) -> bool {
        self.atom().as_ref() == *other
    }
}

impl<'a> PartialOrd<&'a str> for Symbol {
    fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
        self.atom().as_ref().partial_cmp(*other)
    }
}


impl<'a> PartialEq<Cow<'a, str>> for Symbol {
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.atom().as_ref() == other
    }
}

impl<'a> PartialOrd<Cow<'a, str>> for Symbol {
    fn partial_cmp(&self, other: &Cow<'a, str>) -> Option<Ordering> {
        self.atom().as_ref().partial_cmp(other)
    }
}


impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.atom().as_ref(), f)
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.atom().as_ref(), f)
    }
}

impl std::hash::Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.atom().hash(state)
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        self.atom().inc_ref_count();
        Symbol(self.0)
    }
}

impl Default for Symbol {
    fn default() -> Self {
        get_empty_symbol()
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        get_symbol(s)
    }
}

impl<'a> From<&'a String> for Symbol {
    fn from(s: &'a String) -> Self {
        get_symbol(s.as_str())
    }
}

impl<'a> From<&'a str> for Symbol {
    fn from(s: &'a str) -> Self {
        get_symbol(s)
    }
}

impl<'a> From<Cow<'a, str>> for Symbol {
    fn from(s: Cow<'a, str>) -> Self {
        get_symbol(s)
    }
}

impl<'a> From<&'a Symbol> for Symbol {
    fn from(s: &'a Symbol) -> Self {
        s.clone()
    }
}

impl std::ops::Deref for Symbol {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.atom().as_ref()
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.atom().as_ref()
    }
}

impl Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        self.atom().borrow()
    }
}

impl heapsize::HeapSizeOf for Symbol {
    fn heap_size_of_children(&self) -> usize {
        self.atom().heap_size_of_children()
    }
}

impl serde::Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.atom().as_ref().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        Ok(Symbol::from(Cow::deserialize(deserializer)?))
    }
}

unsafe impl Send for Symbol {}

unsafe impl Sync for Symbol {}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    // Some tests must be run consecutively (not in parallel), so we need to lock() before each test
    lazy_static! {
        static ref LOCK: Mutex<()> = Mutex::new(());
    }

    fn lock<'a>() -> MutexGuard<'a, ()> {
        let lock = LOCK.lock().unwrap();
        debug_assert_eq!(SYMBOLS.read().unwrap().len(), 1);
        lock
    }

    #[test]
    fn ptr_equality() {
        let _lock = lock();

        let s1 = Symbol::from("aaa");
        let s2 = Symbol::from("aaa");
        let s3 = s1.clone();
        let s4 = Symbol::from("aaaa");

        assert_eq!(s1.0.as_ptr(), s2.0.as_ptr());
        assert_eq!(s1.0.as_ptr(), s3.0.as_ptr());
        assert_ne!(s1.0.as_ptr(), s4.0.as_ptr());
    }

    #[test]
    fn empty_symbol_is_never_dropped() {
        let _lock = lock();

        {
            let _a = Symbol::default();
            let _b = Symbol::from("");
            assert_eq!(EMPTY_SYMBOL.atom().ref_count(), 3);
            assert_eq!(SYMBOLS.read().unwrap().len(), 1);
        }

        assert_eq!(EMPTY_SYMBOL.atom().ref_count(), 1);
        assert_eq!(SYMBOLS.read().unwrap().len(), 1);
    }

    #[test]
    fn non_empty_symbols_are_dropped() {
        let _lock = lock();

        {
            let _s1 = Symbol::from("aaa");
            let s2 = Symbol::from("aaa");
            let s3 = Symbol::from("aaaa");
            assert_eq!(s2.atom().ref_count(), 2);
            assert_eq!(s3.atom().ref_count(), 1);
            assert_eq!(SYMBOLS.read().unwrap().len(), 3);
        }

        assert_eq!(SYMBOLS.read().unwrap().len(), 1);
    }

    #[test]
    fn symbol_keys_in_maps() {
        let _lock = lock();

        use std::collections::HashMap;

        let mut map: HashMap<Symbol, usize> = HashMap::new();
        map.insert("one".into(), 1);
        map.insert("two".into(), 2);
        map.insert("three".into(), 3);

        let three = Symbol::from("three");

        assert_eq!(map.get("one"), Some(&1));
        assert_eq!(map.get("two"), Some(&2));
        assert!(map.contains_key(&three))
    }

    #[test]
    fn serialize() {
        let _lock = lock();

        let s = Symbol::from("example");
        let json = serde_json::to_string_pretty(&s).unwrap();
        assert_eq!("\"example\"", json);
    }

    #[test]
    fn deserialize() {
        let _lock = lock();

        let json = "\"example\"";
        let s: Symbol = serde_json::from_str(json).unwrap();
        assert_eq!(s, "example");
    }

    #[test]
    fn symbol_is_sync() {
        let _lock = lock();

        fn test<T: Sync>(_: T) {}

        test(Symbol::from("example"));
    }

    #[test]
    fn symbol_is_send() {
        let _lock = lock();

        fn test<T: Send>(_: T) {}

        test(Symbol::from("example"));
    }

    #[test]
    fn symbol_sizeof_is_equal_to_pointer() {
        // can be run in parallel
        assert_eq!(std::mem::size_of::<Symbol>(), std::mem::size_of::<*const ()>());
    }

    #[test]
    fn optional_symbol_sizeof_is_equal_to_pointer() {
        // can be run in parallel
        assert_eq!(std::mem::size_of::<Option<Symbol>>(), std::mem::size_of::<*const ()>());
    }
}
