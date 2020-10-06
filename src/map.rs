use super::Symbol;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::borrow::Borrow;
use std::hash::Hash;
use heapsize::HeapSizeOf;
use std::iter::FusedIterator;

const SMALL_MAP_SIZE: usize = 8;

pub struct SymbolMap<V> {
    items: Vec<(Symbol, V)>,
    map: Option<Box<HashMap<Symbol, usize>>>
}

impl<V> SymbolMap<V> {
    pub fn new() -> Self {
        SymbolMap {
            items: Vec::new(),
            map: None,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        SymbolMap {
            items: Vec::with_capacity(capacity),
            map: if capacity > SMALL_MAP_SIZE {
                Some(Box::new(HashMap::with_capacity(capacity)))
            } else {
                None
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    pub fn shrink_to_fit(&mut self) {
        self.items.shrink_to_fit();
        if let Some(m) = &mut self.map {
            m.shrink_to_fit();
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
        if let Some(m) = &mut self.map {
            m.reserve(additional);
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.map = None;
    }

    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
        where Q: AsRef<str> + Hash + Eq, Symbol: Borrow<Q>
    {
        if let Some(s) = Symbol::get(k) {
            match self.map.as_ref() {
                Some(m) => m.contains_key(k),
                None => self.items.iter().find(|&(k, _)| *k == s).is_some(),
            }
        } else {
            false
        }
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
        where Q: AsRef<str> + Hash + Eq
    {
        if let Some(s) = Symbol::get(k) {
            match self.map.as_ref() {
                Some(m) => {
                    match m.get(&s) {
                        Some(&i) => unsafe { Some(&self.items.get_unchecked(i).1) }
                        None => None,
                    }
                },
                None => self.items.iter().find(|&(k, _)| *k == s).map(|e| &e.1),
            }
        } else {
            None
        }
    }

    fn rebuild_map(&mut self) {
        if self.items.len() <= SMALL_MAP_SIZE {
            self.map = None;
        } else {
            if self.map.is_none() {
                self.map = Some(Box::new(HashMap::with_capacity(self.items.capacity())));
            }
            if let Some(m) = self.map.as_mut() {
                m.clear();
                for (i, e) in self.items.iter().enumerate() {
                    m.insert(e.0.clone(), i);
                }
            }
        }
    }

    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
        where Q: AsRef<str> + Hash + Eq
    {
        if let Some(s) = Symbol::get(k) {
            match self.map.as_mut() {
                Some(m) => {
                    match m.get(&s) {
                        Some(&i) => {
                            let e = self.items.remove(i);
                            self.rebuild_map();
                            Some(e.1)
                        }
                        None => None,
                    }
                },
                None => {
                    if let Some(index) = self.items.iter().position(|(k, _)| s == *k) {
                        let e = self.items.remove(index);
                        Some(e.1)
                    } else {
                        None
                    }
                },
            }
        } else {
            None
        }
    }

    pub fn remove_at(&mut self, index: usize) -> Option<V> {
        let old = self.items.remove(index);
        self.rebuild_map();
        Some(old.1)
    }

    pub fn insert(&mut self, k: Symbol, mut v: V) -> Option<V> {
        match self.map.as_mut() {
            Some(m) => {
                match m.entry(k.clone()) {
                    Entry::Vacant(ve) => {
                        let index = self.items.len();
                        self.items.push((k, v));
                        ve.insert(index);
                        None
                    }
                    Entry::Occupied(oe) => {
                        let e = unsafe {
                            self.items.get_unchecked_mut(*oe.get())
                        };
                        std::mem::swap(&mut e.1, &mut v);
                        Some(v)
                    }
                }
            }
            None => {
                for e in self.items.iter_mut() {
                    if e.0 == k {
                        std::mem::swap(&mut e.1, &mut v);
                        return Some(v);
                    }
                }
                self.items.push((k, v));
                self.rebuild_map();
                None
            }
        }
    }

    pub fn insert_at(&mut self, index: usize, k: Symbol, v: V) -> Option<V> {
        let old = self.remove(&k);
        self.items.insert(index, (k, v));
        self.rebuild_map();
        old
    }

    pub fn pop_front(&mut self) -> Option<(Symbol, V)> {
        match self.items.pop() {
            Some(e) => {
                self.rebuild_map();
                Some(e)
            }
            None => None
        }
    }

    pub fn pop_back(&mut self) -> Option<(Symbol, V)> {
        if self.items.is_empty() {
            None
        } else {
            let e = self.items.remove(self.items.len() - 1);
            self.rebuild_map();
            Some(e)
        }
    }

    pub fn iter(&'_ self) -> Iter<'_, V> {
        Iter(self.items.iter())
    }

    pub fn iter_mut(&'_ mut self) -> IterMut<'_, V> {
        IterMut(self.items.iter_mut())
    }

    pub fn keys(&'_ self) -> Keys<'_, V> {
        Keys(self.items.iter())
    }

    pub fn values(&'_ self) -> Values<'_, V> {
        Values(self.items.iter())
    }

    pub fn values_mut(&'_ mut self) -> ValuesMut<'_, V> {
        ValuesMut(self.items.iter_mut())
    }
}

impl<V> Default for SymbolMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: std::fmt::Debug> std::fmt::Debug for SymbolMap<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.items.iter().map(|e| (&e.0, &e.1))).finish()
    }
}

impl<V: HeapSizeOf> HeapSizeOf for SymbolMap<V> {
    fn heap_size_of_children(&self) -> usize {
        self.items.heap_size_of_children() + self.map.heap_size_of_children()
    }
}


pub struct Iter<'a, V: 'a>(std::slice::Iter<'a, (Symbol, V)>);

impl<'a, V: 'a> Iterator for Iter<'a, V> {
    type Item = (&'a Symbol, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|&(ref k , ref v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, V: 'a> ExactSizeIterator for Iter<'a, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, V: 'a> FusedIterator for Iter<'a, V> { }


pub struct IterMut<'a, V: 'a>(std::slice::IterMut<'a, (Symbol, V)>);

impl<'a, V: 'a> Iterator for IterMut<'a, V> {
    type Item = (&'a Symbol, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|&mut (ref k, ref mut v)| (k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, V: 'a> ExactSizeIterator for IterMut<'a, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, V: 'a> FusedIterator for IterMut<'a, V> { }


pub struct Keys<'a, V: 'a>(std::slice::Iter<'a, (Symbol, V)>);

impl<'a, V: 'a> Iterator for Keys<'a, V> {
    type Item = &'a Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|&(ref k , _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, V: 'a> ExactSizeIterator for Keys<'a, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, V: 'a> FusedIterator for Keys<'a, V> { }


pub struct Values<'a, V: 'a>(std::slice::Iter<'a, (Symbol, V)>);

impl<'a, V: 'a> Iterator for Values<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|&(_ , ref v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, V: 'a> ExactSizeIterator for Values<'a, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, V: 'a> FusedIterator for Values<'a, V> { }


pub struct ValuesMut<'a, V: 'a>(std::slice::IterMut<'a, (Symbol, V)>);

impl<'a, V: 'a> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|&mut (_ , ref mut v)| v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, V: 'a> ExactSizeIterator for ValuesMut<'a, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, V: 'a> FusedIterator for ValuesMut<'a, V> { }


#[cfg(test)]
mod tests {
    use crate::*;
    use crate::tests::test_lock;

    #[test]
    fn small_map_smoke_test() {
        let _lock = test_lock();

        let mut m = SymbolMap::new();

        m.insert("key1".into(), "v1");
        m.insert("key2".into(), "v2");
        m.insert("key1".into(), "v3");

        assert_eq!(m.len(), 2);
        assert_eq!(m.get("key1"), Some(&"v3"));
        assert_eq!(m.get("key4"), None);
        assert_eq!(SYMBOLS.lock().len(), 3);
    }
}