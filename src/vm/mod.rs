//! Virtual machine module for the Metorex interpreter.
//!
//! This module contains the core virtual machine implementation and related support structures.

mod begin_rescue;
mod block_execution;
mod call_frame;
mod class_execution;
mod control_flow;
mod control_structures;
pub(crate) mod core;
pub(crate) mod errors;
mod eval;
mod exceptions;
mod expression;
mod global_registry;
mod heap;
mod init;
mod loading;
mod method_execution;
mod method_invocation;
mod method_lookup;
mod native_functions;
mod native_methods;
mod operators;
pub(super) mod param_binding;
mod pattern_matching;
mod program;
pub(crate) mod statement;
pub(crate) mod utils;

pub use call_frame::CallFrame;
pub use core::VirtualMachine;
pub use global_registry::GlobalRegistry;
pub use heap::Heap;

pub(crate) use control_flow::ControlFlow;
