#![feature(integer_atomics, allocator_api)]

#[macro_use]
extern crate lazy_static;
extern crate heapsize;
extern crate serde;
extern crate parking_lot;

#[cfg(test)]
extern crate serde_json;

use std::cmp::Ordering;
use std::ops::Deref;
use std::borrow::{Cow, Borrow};
use std::ptr::NonNull;
use std::alloc::{Layout, Alloc, Global, handle_alloc_error};
use std::collections::HashSet;
use std::sync::atomic::AtomicU32;
use std::hash::{Hash, Hasher};
use parking_lot::Mutex;


lazy_static!{
    static ref SYMBOLS: Mutex<HashSet<Symbol>> = Mutex::new(HashSet::new());
}


struct Header {
    ref_count: AtomicU32,
    ptr: NonNull<u8>,
    len: usize,
}

impl AsRef<str> for Header {
    fn as_ref(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.ptr.as_ptr(), self.len))
        }
    }
}


#[inline]
fn layout_offset(len: usize) -> (Layout, usize) {
    unsafe {
        Layout::new::<Header>().extend(Layout::from_size_align_unchecked(len, 1)).unwrap()
    }
}


pub struct Symbol(NonNull<u8>);

impl Symbol {
    pub fn new<S: AsRef<str>>(value: S) -> Symbol {
        let mut symbols = SYMBOLS.lock();
        let value = value.as_ref();
        if let Some(s) = symbols.get(value) {
            s.header().ref_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Symbol(s.0);
        }
        let (layout, offset) = layout_offset(value.len());
        let p = unsafe {
            let data = Global.alloc(layout).unwrap_or_else(|_| handle_alloc_error(layout));
            let str_ptr = data.as_ptr().offset(offset as isize);
            let hdr_ptr = std::mem::transmute::<NonNull<u8>, &mut Header>(data);
            *hdr_ptr = Header {
                ref_count: AtomicU32::new(1),
                ptr: NonNull::new_unchecked(str_ptr),
                len: value.len(),
            };
            std::ptr::copy_nonoverlapping(value.as_ptr(), str_ptr, value.len());
            data
        };
        symbols.insert(Symbol(p));
        Symbol(p)
    }

    #[inline(always)]
    fn header(&self) -> &Header {
        unsafe { std::mem::transmute::<NonNull<u8>, &Header>(self.0) }
    }

    #[cfg(test)]
    fn ref_count(&self) -> u32 {
        self.header().ref_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[cfg(test)]
    pub fn print_all() {
        println!("{:#?}", *SYMBOLS.lock())
    }
}

impl Drop for Symbol {
    #[inline]
    fn drop(&mut self) {
        let mut symbols = SYMBOLS.lock();
        if self.header().ref_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) == 1 {
            let s = symbols.take(self.as_ref()).unwrap();
            std::mem::forget(s);
            let (layout, _) = layout_offset(self.header().len);
            unsafe {
                Global.dealloc(self.0, layout);
            }
        }
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        self.header().ref_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Symbol(self.0)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.header().as_ref()
    }
}

impl Deref for Symbol {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        self.0 == other.0
    }
}

impl Eq for Symbol {}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            Ordering::Equal
        } else {
            self.as_ref().cmp(&other.as_ref())
        }
    }
}

impl PartialEq<str> for Symbol {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<&'a str> for Symbol {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<String> for Symbol {
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl<'a> PartialEq<Cow<'a, str>> for Symbol {
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl PartialOrd<str> for Symbol {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<'a> PartialOrd<&'a str> for Symbol {
    fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
        self.as_ref().partial_cmp(*other)
    }
}

impl PartialOrd<String> for Symbol {
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_str())
    }
}

impl<'a> PartialOrd<Cow<'a, str>> for Symbol {
    fn partial_cmp(&self, other: &Cow<'a, str>) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_ref(), f)
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_ref(), f)
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol::new("")
    }
}

impl<'a> From<&'a Symbol> for Symbol {
    fn from(s: &'a Symbol) -> Self {
        s.clone()
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Symbol::new(s)
    }
}

impl<'a> From<&'a String> for Symbol {
    fn from(s: &'a String) -> Self {
        Symbol::new(s)
    }
}

impl<'a> From<&'a str> for Symbol {
    fn from(s: &'a str) -> Self {
        Symbol::new(s)
    }
}

impl<'a> From<Cow<'a, str>> for Symbol {
    fn from(s: Cow<'a, str>) -> Self {
        Symbol::new(s)
    }
}

impl<'a, 'b> From<&'b Cow<'a, str>> for Symbol {
    fn from(s: &'b Cow<'a, str>) -> Self {
        Symbol::new(s)
    }
}

impl heapsize::HeapSizeOf for Symbol {
    fn heap_size_of_children(&self) -> usize {
        layout_offset(self.header().len).0.size()
    }
}

impl serde::Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.as_ref().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        Ok(Symbol::from(String::deserialize(deserializer)?))
    }
}

unsafe impl Send for Symbol {}

unsafe impl Sync for Symbol {}


#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::{Mutex, MutexGuard};

    // Some tests must be run consecutively (not in parallel), so we need to lock() before each test
    static LOCK: Mutex<()> = Mutex::new(());

    fn lock<'a>() -> MutexGuard<'a, ()> {
        let lock = LOCK.lock();
        debug_assert_eq!(SYMBOLS.lock().len(), 0);
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
    fn symbols_are_dropped() {
        let _lock = lock();

        {
            let _s1 = Symbol::from("aaa");
            let s2 = Symbol::from("aaa");
            let s3 = Symbol::from("aaaa");
            assert_eq!(s2.ref_count(), 2);
            assert_eq!(s3.ref_count(), 1);
            assert_eq!(SYMBOLS.lock().len(), 2);
        }

        assert_eq!(SYMBOLS.lock().len(), 0);
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
        assert_eq!(s.as_ref(), "example");
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
    fn symbol_hash_eq_str_hash() {
        use std::collections::hash_map::DefaultHasher;

        let _lock = lock();

        let s1 = "example string";
        let h1 = {
            let mut hasher = DefaultHasher::new();
            s1.hash(&mut hasher);
            hasher.finish()
        };

        let s2 = Symbol::new(s1);
        let h2 = {
            let mut hasher = DefaultHasher::new();
            s2.hash(&mut hasher);
            hasher.finish()
        };

        assert_eq!(h1, h2);
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



