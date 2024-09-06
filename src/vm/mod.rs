//! Virtual machine module for the Metorex interpreter.
//!
//! This module contains the core virtual machine implementation and related support structures.

mod call_frame;
mod class_execution;
mod control_flow;
mod control_structures;
mod core;
mod errors;
mod exceptions;
mod expression;
mod global_registry;
mod heap;
mod method_invocation;
mod method_lookup;
mod native_methods;
mod operators;
mod pattern_matching;
mod statement;
mod utils;

pub use call_frame::CallFrame;
pub use core::VirtualMachine;
pub use global_registry::GlobalRegistry;
pub use heap::Heap;

pub(crate) use control_flow::ControlFlow;
