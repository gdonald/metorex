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

/// The key the parser adds to a keyword-argument dict so it can be told
/// apart from a Hash the caller passed positionally.
pub(crate) const KWARGS_MARKER: &str = "__MX_KWARGS__";

/// Count the number of positional arguments, excluding a trailing kwargs dict.
pub(crate) fn positional_arg_count(arguments: &[Object]) -> usize {
    positional_arg_count_for(arguments, true)
}

/// The positional count, given whether the callee declares any keyword
/// parameters. A callee that declares none takes a trailing keyword hash as an
/// ordinary positional argument, which is what Ruby counts it as.
pub(crate) fn positional_arg_count_for(arguments: &[Object], takes_keywords: bool) -> usize {
    if takes_keywords && let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key(KWARGS_MARKER) {
            return arguments.len() - 1;
        }
    }
    arguments.len()
}

/// Bind one positional parameter. A `def f((a, b))` group spreads the value
/// it is given across the names in the group, the same way a block's does.
fn define_positional_param(
    vm: &mut VirtualMachine,
    param: &str,
    value: Object,
) -> Result<(), MetorexError> {
    match param.strip_prefix(crate::object::DESTRUCTURED_GROUP_PREFIX) {
        Some(names) => crate::vm::block_execution::bind_group_names(vm, names, value),
        None => {
            vm.environment_mut().define(param.to_string(), value);
            Ok(())
        }
    }
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
    // A default that assigns a local, as `def f(a = b = 1)` does, names that
    // local for the whole body whether or not the default is reached.
    for (_, default_expr) in default_parameters {
        let written = crate::ast::Statement::Expression {
            expression: default_expr.clone(),
            position: default_expr.position(),
        };
        for name in crate::ast::collect_assigned_locals(std::slice::from_ref(&written)) {
            if vm.environment().get(&name).is_none() {
                vm.environment_mut().define(name, Object::Nil);
            }
        }
    }
    if let Some((vi, _)) = variadic_param {
        let vi = *vi;
        let params_after_splat = params.len() - vi - 1;
        let has_a_default =
            |index: usize| default_parameters.iter().any(|(held, _)| *held == index);
        // Every required parameter is filled before any optional one, whether
        // it stands before the splat or after it, so `def f(a, b = 9, *r, q)`
        // called with two values gives `q` the second and leaves `b` at its
        // default.
        let required_before = (0..vi).filter(|index| !has_a_default(*index)).count();
        let mut for_optionals = positional
            .len()
            .saturating_sub(required_before + params_after_splat);
        let mut cursor = 0;

        for (i, param) in params.iter().enumerate() {
            let value = if i < vi {
                // Before the splat, in the order they were written: an
                // optional one takes a value only while any are left over.
                let takes_one = if has_a_default(i) {
                    let spare = for_optionals > 0;
                    for_optionals = for_optionals.saturating_sub(1);
                    spare
                } else {
                    true
                };
                match positional.get(cursor).filter(|_| takes_one) {
                    Some(value) => {
                        cursor += 1;
                        value.clone()
                    }
                    None => match default_parameters.iter().find(|(index, _)| *index == i) {
                        Some((_, default_expr)) => vm.evaluate_expression(default_expr)?,
                        None => Object::Nil,
                    },
                }
            } else if i == vi {
                // The splat parameter: collect middle args into an array
                let upto = positional
                    .len()
                    .saturating_sub(params_after_splat)
                    .max(cursor);
                let rest: Vec<Object> = positional.get(cursor..upto).unwrap_or(&[]).to_vec();
                Object::Array(Rc::new(RefCell::new(rest)))
            } else {
                // After the splat, the last values, and where there are too
                // few they start right after what came before and run out
                // into nil.
                let from = positional
                    .len()
                    .saturating_sub(params_after_splat)
                    .max(cursor);
                positional
                    .get(from + i - vi - 1)
                    .cloned()
                    .unwrap_or(Object::Nil)
            };
            define_positional_param(vm, param, value)?;
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
            define_positional_param(vm, param, value)?;
        }
    } else {
        for (i, param) in params.iter().enumerate() {
            let value = if i < positional.len() {
                positional[i].clone()
            } else {
                Object::Nil
            };
            define_positional_param(vm, param, value)?;
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
    arguments: Vec<Object>,
    has_keyword_params: bool,
) -> (Vec<Object>, IndexMap<String, Object>) {
    split_keyword_args_for(arguments, has_keyword_params, false)
}

/// The same split, told whether the callee was named by `ruby2_keywords`. A
/// method that was keeps the keyword hash marked, so the splat it lands in
/// can pass it on as keywords again.
pub(crate) fn split_keyword_args_for(
    mut arguments: Vec<Object>,
    has_keyword_params: bool,
    keeps_keywords: bool,
) -> (Vec<Object>, IndexMap<String, Object>) {
    // Only split if the trailing dict carries the parser-emitted kwargs marker.
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            if !has_keyword_params {
                // Promote the kwargs dict to a regular Hash positional arg.
                let mut cleaned: IndexMap<String, Object> = dict
                    .iter()
                    .filter(|(k, _)| {
                        !matches!(
                            k.as_str(),
                            "__MX_KWARGS__"
                                | crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                        )
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                if keeps_keywords {
                    cleaned.insert(
                        crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY.to_string(),
                        Object::Bool(true),
                    );
                }
                drop(dict);
                arguments.pop();
                arguments.push(Object::Dict(Rc::new(RefCell::new(cleaned))));
                return (arguments, IndexMap::new());
            }
            // The table of key objects is the hash's own bookkeeping rather
            // than a keyword, so it is left behind with the markers.
            let kwargs: IndexMap<String, Object> = dict
                .iter()
                .filter(|(k, _)| {
                    !matches!(
                        k.as_str(),
                        "__MX_KWARGS__"
                            | crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                            | crate::vm::native_methods::hash_methods::KEY_OBJECTS_KEY
                    )
                })
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
