//! Heap memory management for the Metorex virtual machine.
//!
//! This module provides a lightweight heap structure that tracks allocated objects.
//! Currently a stub implementation; will be expanded for garbage collection integration.

use crate::object::Object;

/// Lightweight heap placeholder that will evolve with the runtime.
#[derive(Debug, Default)]
pub struct Heap {
    /// Tracks allocated objects for future GC integration.
    allocated: Vec<Object>,
}

impl Heap {
    /// Allocate an object on the heap (no-op stub for now).
    pub fn allocate(&mut self, object: Object) {
        self.allocated.push(object);
    }

    /// Returns number of tracked allocations (for testing/introspection).
    pub fn allocation_count(&self) -> usize {
        self.allocated.len()
    }
}
