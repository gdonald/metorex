//! Every symbol name the program has spelled, whether it has been evaluated
//! yet or not. Ruby interns a symbol when the source naming it is read, and
//! `Symbol.all_symbols` reports what the table holds.

use std::cell::RefCell;
use std::collections::BTreeSet;

thread_local! {
    static NAMES: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
}

/// Record a symbol name, which is what reading one in the source and making
/// one at run time both do.
pub fn record(name: &str) {
    NAMES.with(|held| {
        let mut held = held.borrow_mut();
        if !held.contains(name) {
            held.insert(name.to_string());
        }
    });
}

/// Every name recorded so far, in the order a sorted set holds them.
pub fn all() -> Vec<String> {
    NAMES.with(|held| held.borrow().iter().cloned().collect())
}

/// Whether a symbol of this name has been made, without making one.
pub fn contains(name: &str) -> bool {
    NAMES.with(|held| held.borrow().contains(name))
}
