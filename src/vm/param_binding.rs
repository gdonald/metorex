//! Parameter binding utilities for the virtual machine.
//!
//! This module provides functions for binding positional, keyword, variadic,
//! and block parameters when invoking methods and functions.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::VirtualMachine;

/// Count the number of positional arguments, excluding a trailing kwargs dict.
pub(crate) fn positional_arg_count(arguments: &[Object]) -> usize {
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            return arguments.len() - 1;
        }
    }
    arguments.len()
}

/// Bind positional parameters to arguments, handling variadic (splat) parameters.
///
/// When a variadic param is present at index `vi`, parameters before it get one arg each,
/// the variadic param collects remaining args as an Array, and parameters after it
/// get args from the end.
pub(crate) fn bind_params(
    vm: &mut VirtualMachine,
    params: &[String],
    positional: &[Object],
    default_parameters: &[(usize, Expression)],
    variadic_param: &Option<(usize, String)>,
) -> Result<(), MetorexError> {
    if let Some((vi, _)) = variadic_param {
        let vi = *vi;
        let params_after_splat = params.len() - vi - 1;
        let min_positional = vi + params_after_splat;
        let splat_count = positional.len().saturating_sub(min_positional);

        for (i, param) in params.iter().enumerate() {
            let value = if i < vi {
                // Before splat: normal positional
                positional.get(i).cloned().unwrap_or(Object::Nil)
            } else if i == vi {
                // The splat parameter: collect middle args into an array
                let rest: Vec<Object> =
                    positional.get(vi..vi + splat_count).unwrap_or(&[]).to_vec();
                Object::Array(Rc::new(RefCell::new(rest)))
            } else {
                // After splat: take from end of positional
                let offset_from_end = params.len() - i;
                let idx = positional.len().saturating_sub(offset_from_end);
                positional.get(idx).cloned().unwrap_or(Object::Nil)
            };
            vm.environment_mut().define(param.clone(), value);
        }
    } else {
        for (i, param) in params.iter().enumerate() {
            let value = if i < positional.len() {
                positional[i].clone()
            } else if let Some((_, default_expr)) =
                default_parameters.iter().find(|(idx, _)| *idx == i)
            {
                vm.evaluate_expression(default_expr)?
            } else {
                Object::Nil
            };
            vm.environment_mut().define(param.clone(), value);
        }
    }
    Ok(())
}

/// Split a list of evaluated arguments into positional args and keyword args.
/// If the last argument is a Dict with symbol-style keys, it's treated as keyword args.
pub(crate) fn split_keyword_args(
    mut arguments: Vec<Object>,
) -> (Vec<Object>, HashMap<String, Object>) {
    // Only split if the trailing dict carries the parser-emitted kwargs marker.
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            let kwargs: HashMap<String, Object> = dict
                .iter()
                .filter(|(k, _)| k.as_str() != "__MX_KWARGS__")
                .map(|(k, v)| {
                    let name = if let Some(stripped) = k.strip_prefix(':') {
                        stripped.to_string()
                    } else {
                        k.clone()
                    };
                    (name, v.clone())
                })
                .collect();
            drop(dict);
            arguments.pop();
            return (arguments, kwargs);
        }
    }
    (arguments, HashMap::new())
}
