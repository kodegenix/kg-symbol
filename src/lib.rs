#![feature(box_syntax, integer_atomics)]

#[macro_use]
extern crate lazy_static;
extern crate heapsize;
extern crate serde;

#[cfg(test)]
extern crate serde_json;

mod atom;
mod symbol;

use atom::*;
pub use symbol::Symbol;

use std::collections::HashSet;
use std::sync::RwLock;


lazy_static!{
    static ref SYMBOLS: RwLock<HashSet<Box<AtomString>>> = {
        let mut set = HashSet::new();
        set.insert(box AtomString::empty());
        RwLock::new(set)
    };

    static ref EMPTY_SYMBOL: Symbol = {
        Symbol::wrap(SYMBOLS.read().unwrap().get("").unwrap())
    };
}

fn get_empty_symbol() -> Symbol {
    EMPTY_SYMBOL.clone()
}

fn get_symbol<S: Into<String> + AsRef<str>>(value: S) -> Symbol {
    {
        let set = SYMBOLS.read().unwrap();
        if let Some(s) = set.get(value.as_ref()) {
            return Symbol::new(s);
        }
    }
    {
        let s = box AtomString::new(value);
        let symbol = Symbol::new(&s);
        let mut set = SYMBOLS.write().unwrap();
        set.insert(s);
        symbol
    }
}

fn remove_entry(e: &AtomString) {
    let mut set = SYMBOLS.write().unwrap();
    set.remove(e);
}


pub fn print_symbols() {
    println!("{:#?}", *SYMBOLS.read().unwrap())
}
