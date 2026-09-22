// What the collector accounts for.

use super::*;

impl VirtualMachine {
    /// How many collections have been asked for. Metorex frees an object when
    /// its last reference goes, so nothing else moves this.
    pub(crate) fn gc_collection_count(&self) -> i64 {
        match self.globals().get("__gc_count") {
            Some(Object::Int(count)) => count,
            _ => 0,
        }
    }

    /// The nanoseconds spent collecting. Each request accounts for the time
    /// its own bookkeeping took, which is what keeps the total climbing.
    pub(crate) fn gc_total_time(&self) -> i64 {
        match self.globals().get("__gc_total_time") {
            Some(Object::Int(nanoseconds)) => nanoseconds,
            _ => 0,
        }
    }

    /// Record that a collection was asked for.
    pub(crate) fn record_gc_run(&mut self) {
        let count = self.gc_collection_count() + 1;
        let spent = self.gc_total_time() + 1;
        self.globals_mut().set("__gc_count", Object::Int(count));
        self.globals_mut()
            .set("__gc_total_time", Object::Int(spent));
    }
}
