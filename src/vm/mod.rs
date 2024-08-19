//! Virtual machine module for the Metorex interpreter.
//!
//! This module contains the core virtual machine implementation and related support structures.

mod call_frame;
mod control_flow;
mod core;
mod errors;
mod expression;
mod global_registry;
mod heap;
mod operators;
mod utils;

pub use call_frame::CallFrame;
pub use core::VirtualMachine;
pub use global_registry::GlobalRegistry;
pub use heap::Heap;

pub(crate) use control_flow::ControlFlow;
