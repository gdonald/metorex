//! Native (built-in) method implementations for the virtual machine.
//!
//! This module contains the implementations of all built-in methods for
//! standard classes like Object, String, and Array.

pub(crate) mod array_methods;
pub(crate) mod ast_methods;
pub(crate) mod class_methods;
pub(crate) use class_methods::MODULE_FUNCTION_VISIBILITY;
pub(crate) use class_methods::is_native_kernel_method;
pub(crate) use class_methods::native_module_method_stub;
pub(crate) use hash_methods::remember_key_object;
pub(crate) use method_object_methods::{block_parameter_list, method_parameter_list};
pub(crate) use module_methods::{
    PROCESS_NATIVE_METHODS, REFINEMENT_KEY_PREFIX, REFINEMENT_LABEL_KEY,
};
mod binding_methods;
mod socket_addresses;
pub(crate) use socket_addresses::OpenSockets;
pub(crate) use streams::OpenStreams;
mod complex_methods;
mod constant_visibility;
pub(crate) mod define_method;
mod digest_algorithms;
mod etc_methods;
mod exception_methods;
mod file_methods;
mod float_methods;
pub(crate) mod hash_methods;
mod int_methods;
mod io_methods;
pub(crate) mod kernel_conversion;
pub(crate) mod method_object_methods;
mod module_methods;
pub(crate) mod object_methods;
mod range_methods;
pub(crate) mod rational_methods;
mod syslog_write;
mod zlib_deflate;
mod zlib_streams;
pub(crate) use object_methods::binary_op_for_method_name;
pub(crate) use rational_methods::{complex_parts, rational_parts};
pub(crate) mod regexp_methods;
pub(crate) use regexp_methods::{
    LAST_MATCH, capture_reference, comparable_flags, compile, subject_text,
};
pub(crate) mod euc_jp_table;
pub(crate) mod glob;
pub(crate) mod normalization_table;
mod set_methods;
pub(crate) mod shift_jis_table;
pub(crate) mod string_element_set;
pub(crate) mod string_methods;
pub(crate) mod string_mutation;
mod string_sets;
pub(crate) mod struct_methods;
mod time_methods;
pub(crate) use struct_methods::struct_members;

pub(crate) mod pack_format;
mod streams;
mod visibility;

mod condition_variables;
mod dispatch;
mod enumerators;
mod fibers;
mod mutexes;
mod names;
mod queues;
mod subclasses;
mod threads;
mod warnings;

pub(crate) use crate::class::Class;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::Object;
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use std::rc::Rc;

pub(crate) use enumerators::*;
pub(crate) use names::*;
pub(crate) use subclasses::*;

pub(crate) use int_methods::{RoundingMode, exact_ratio, split_rounding_mode};

/// Multiply by a power of two in steps small enough that each factor is a
/// Float, so an exponent far outside the Float range still scales correctly.
pub(crate) fn scale_by_power_of_two(value: f64, exponent: i64) -> f64 {
    const STEP: i64 = 500;
    let mut value = value;
    let mut remaining = exponent;
    while remaining != 0 {
        let step = remaining.clamp(-STEP, STEP);
        value *= (2f64).powi(step as i32);
        remaining -= step;
        if value == 0.0 || !value.is_finite() {
            break;
        }
    }
    value
}
