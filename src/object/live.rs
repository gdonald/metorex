// The objects a program has made that are still alive, which
// `ObjectSpace.each_object` walks.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::class::Class;

use super::Instance;

/// How many entries the registry holds before its dead ones are first cleared
/// out. Each clearing doubles the count the next one waits for.
const FIRST_CLEARING_AT: usize = 1024;

/// Weak references to what was made, in the order it was made. A reference
/// keeps only its allocation, not the object, so the dead ones are cleared out
/// as the registry grows.
struct Registry<T> {
    held: Vec<Weak<T>>,
    clear_at: usize,
}

impl<T> Registry<T> {
    fn new() -> Self {
        Self {
            held: Vec::new(),
            clear_at: FIRST_CLEARING_AT,
        }
    }

    fn record(&mut self, made: &Rc<T>) {
        if self.held.len() >= self.clear_at {
            self.held.retain(|one| one.strong_count() > 0);
            self.clear_at = (self.held.len() * 2).max(FIRST_CLEARING_AT);
        }
        self.held.push(Rc::downgrade(made));
    }

    fn alive(&self) -> Vec<Rc<T>> {
        self.held.iter().filter_map(Weak::upgrade).collect()
    }
}

thread_local! {
    static INSTANCES: RefCell<Registry<RefCell<Instance>>> = RefCell::new(Registry::new());
    static CLASSES: RefCell<Registry<Class>> = RefCell::new(Registry::new());
}

/// Record an instance the program made.
pub fn record_instance(made: &Rc<RefCell<Instance>>) {
    INSTANCES.with(|registry| registry.borrow_mut().record(made));
}

/// Record a class or module the program made.
pub fn record_class(made: &Rc<Class>) {
    CLASSES.with(|registry| registry.borrow_mut().record(made));
}

/// Every instance still alive, oldest first.
pub fn live_instances() -> Vec<Rc<RefCell<Instance>>> {
    INSTANCES.with(|registry| registry.borrow().alive())
}

/// Every class and module still alive, oldest first.
pub fn live_classes() -> Vec<Rc<Class>> {
    CLASSES.with(|registry| registry.borrow().alive())
}
