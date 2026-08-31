//! Parameter binding utilities for the virtual machine.
//!
//! This module provides functions for binding positional, keyword, variadic,
//! and block parameters when invoking methods and functions.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::object::Object;
use indexmap::IndexMap;
use std::cell::RefCell;
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
    } else if let (Some(first_optional), Some(last_optional)) = (
        default_parameters.iter().map(|(index, _)| *index).min(),
        default_parameters.iter().map(|(index, _)| *index).max(),
    ) {
        // `def f(a, b = 1, c)` fills the required parameters on either side
        // first, from the front and from the back, and gives what is left to
        // the optional ones in the middle.
        let trailing_required = params.len() - last_optional - 1;
        let for_optionals = positional
            .len()
            .saturating_sub(first_optional + trailing_required);
        for (i, param) in params.iter().enumerate() {
            let value = if i < first_optional {
                positional.get(i).cloned().unwrap_or(Object::Nil)
            } else if i <= last_optional {
                let rank = i - first_optional;
                match positional
                    .get(first_optional + rank)
                    .filter(|_| rank < for_optionals)
                {
                    Some(value) => value.clone(),
                    None => match default_parameters.iter().find(|(index, _)| *index == i) {
                        Some((_, default_expr)) => vm.evaluate_expression(default_expr)?,
                        None => Object::Nil,
                    },
                }
            } else {
                let offset_from_end = params.len() - i;
                let index = positional.len().saturating_sub(offset_from_end);
                positional.get(index).cloned().unwrap_or(Object::Nil)
            };
            vm.environment_mut().define(param.clone(), value);
        }
    } else {
        for (i, param) in params.iter().enumerate() {
            let value = if i < positional.len() {
                positional[i].clone()
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
///
/// `has_keyword_params` indicates whether the callee declared any keyword
/// parameters. When false, the trailing kwargs dict (if any) is kept as a
/// positional argument with its `__MX_KWARGS__` marker stripped — matching
/// Ruby's behavior of folding trailing `key: value` syntax into a Hash that
/// fills the last positional parameter.
pub(crate) fn split_keyword_args(
    mut arguments: Vec<Object>,
    has_keyword_params: bool,
) -> (Vec<Object>, IndexMap<String, Object>) {
    // Only split if the trailing dict carries the parser-emitted kwargs marker.
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            if !has_keyword_params {
                // Promote the kwargs dict to a regular Hash positional arg.
                let cleaned: IndexMap<String, Object> = dict
                    .iter()
                    .filter(|(k, _)| k.as_str() != "__MX_KWARGS__")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                drop(dict);
                arguments.pop();
                arguments.push(Object::Dict(Rc::new(RefCell::new(cleaned))));
                return (arguments, IndexMap::new());
            }
            let kwargs: IndexMap<String, Object> = dict
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
    (arguments, IndexMap::new())
}
