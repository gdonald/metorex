// Binding the parameters a block was written with to the
// arguments it was handed.

use super::*;

/// Bind block parameters to arguments, handling `*args` (variadic) and
/// `&block` (block) prefixes in parameter names. `defaults` carries
/// default-value expressions keyed by index into `params`; they evaluate
/// in the block's fresh scope when the corresponding argument is missing.
/// Bind one block parameter. A `|(a, b)|` group spreads the value it is given
/// across the names in the group, filling nil where the array is shorter.
fn define_block_param(
    vm: &mut VirtualMachine,
    param: &str,
    value: Object,
) -> Result<(), MetorexError> {
    // The parameter a block takes because its body reads a bare `it` binds
    // that name without declaring it: `local_variables` does not report it,
    // and an `it` already in scope keeps its meaning.
    if param == crate::object::IMPLICIT_IT_PARAM {
        if vm.environment().get("it").is_none() || vm.environment().name_is_only_hoisted("it") {
            vm.environment_mut().define_hidden("it".to_string(), value);
        }
        return Ok(());
    }
    // A numbered parameter binds its name without declaring it, the same
    // way the implicit `it` does.
    if crate::parser::names_a_numbered_parameter(param) {
        vm.environment_mut().define_hidden(param.to_string(), value);
        return Ok(());
    }
    let Some(names) = param.strip_prefix(crate::object::DESTRUCTURED_GROUP_PREFIX) else {
        vm.environment_mut().define(param.to_string(), value);
        return Ok(());
    };
    bind_group_names(vm, names, value)
}

/// The names a destructuring group holds, split at the commas that stand
/// outside any nested group.
fn group_parts(names: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut held = String::new();
    let mut depth = 0usize;
    for letter in names.chars() {
        match letter {
            '(' => {
                depth += 1;
                held.push(letter);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                held.push(letter);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut held)),
            _ => held.push(letter),
        }
    }
    if !held.is_empty() {
        parts.push(held);
    }
    parts
}

/// Spread one value across the names of a destructuring group. A group may
/// hold groups of its own, and one name may take whatever the others leave.
pub(crate) fn bind_group_names(
    vm: &mut VirtualMachine,
    names: &str,
    value: Object,
) -> Result<(), MetorexError> {
    // Anything but an Array is asked for `to_ary`, and what that answers
    // spreads across the names. A value that answers none has nothing to
    // spread: the first name that is not a splat takes it, the splats take
    // nothing, and the rest are nil.
    let value = match &value {
        Object::Array(_) => value,
        other if vm.responds_to(other, "to_ary") => {
            let answered =
                vm.send_to_object(other.clone(), "to_ary", Vec::new(), Position::new(0, 0, 0))?;
            match answered {
                Object::Array(_) => answered,
                other_answer => {
                    let named = vm.builtins().class_of(other).ruby_name();
                    let gives = vm.builtins().class_of(&other_answer).ruby_name();
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        &format!(
                            "can't convert {} to Array ({}#to_ary gives {})",
                            named, named, gives
                        ),
                        Position::new(0, 0, 0),
                    ));
                }
            }
        }
        _ => value,
    };
    let (spread, spreads) = match &value {
        Object::Array(elements) => (elements.borrow().clone(), true),
        other => (vec![other.clone()], false),
    };
    let parts = group_parts(names);
    let star_at = parts.iter().position(|part| part.starts_with('*'));
    let first_named = parts.iter().position(|part| !part.starts_with('*'));
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        // Everything the named parts do not take goes to the starred one.
        let taken = if !spreads {
            if Some(index) == first_named {
                value.clone()
            } else if Some(index) == star_at {
                // With no name beside it, the splat is what takes the value.
                if first_named.is_none() {
                    Object::array(vec![value.clone()])
                } else {
                    Object::array(Vec::new())
                }
            } else {
                Object::Nil
            }
        } else {
            match star_at {
                Some(star) if index == star => {
                    let after = parts.len() - index - 1;
                    let upto = spread.len().saturating_sub(after).max(index);
                    Object::array(spread.get(index..upto).unwrap_or(&[]).to_vec())
                }
                Some(star) if index > star => {
                    // The names after the splat take the last values, and
                    // where there are too few they start right after the
                    // names before the splat and run out into nil.
                    let after = parts.len() - star - 1;
                    let from = spread.len().saturating_sub(after).max(star);
                    spread
                        .get(from + index - star - 1)
                        .cloned()
                        .unwrap_or(Object::Nil)
                }
                _ => spread.get(index).cloned().unwrap_or(Object::Nil),
            }
        };
        match part
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
        {
            Some(nested) => bind_group_names(vm, nested, taken)?,
            None => {
                let name = part.strip_prefix('*').unwrap_or(part);
                if name.is_empty() {
                    continue;
                }
                vm.environment_mut().define(name.to_string(), taken);
            }
        }
    }
    Ok(())
}

pub(crate) fn bind_block_params(
    vm: &mut VirtualMachine,
    params: &[String],
    defaults: &[(usize, crate::ast::Expression)],
    arguments: Vec<Object>,
    position: Position,
) -> Result<(), MetorexError> {
    // A keyword declared without a default has to be given, and a call that
    // leaves one out is refused the way a method's would be.
    let mut missing_keywords: Vec<String> = Vec::new();
    // A trailing keyword-argument hash feeds the `name:` parameters, and
    // what is left over is bound by position.
    let keyword_params: Vec<String> = params
        .iter()
        .filter(|param| param.starts_with(crate::object::KEYWORD_PARAM_PREFIX))
        .cloned()
        .collect();
    // `|**rest|` takes whatever keywords the named ones leave.
    let keyword_rest: Option<String> = params
        .iter()
        .find(|param| param.starts_with("**") && *param != crate::object::NO_KEYWORDS_PARAM)
        .map(|param| param.trim_start_matches('*').to_string());
    let mut arguments = arguments;
    // `**empty` passes no keywords at all, so it leaves nothing to bind.
    if matches!(arguments.last(), Some(Object::Dict(entries))
        if passed_as_keywords(entries) && keyword_entries_are_empty(entries))
    {
        arguments.pop();
    }
    if !keyword_params.is_empty() || keyword_rest.is_some() {
        // Only a hash passed as keywords feeds them. One passed by position
        // stays a positional argument.
        let named = match arguments.last() {
            Some(Object::Dict(entries)) if passed_as_keywords(entries) => {
                let taken = entries.borrow().clone();
                arguments.pop();
                taken
            }
            _ => indexmap::IndexMap::new(),
        };
        if let Some(rest_name) = &keyword_rest
            && !rest_name.is_empty()
        {
            let declared: std::collections::HashSet<String> = keyword_params
                .iter()
                .map(|param| {
                    format!(
                        ":{}",
                        param
                            .strip_prefix(crate::object::KEYWORD_PARAM_PREFIX)
                            .unwrap_or(param)
                    )
                })
                .collect();
            let rest: indexmap::IndexMap<String, Object> = named
                .iter()
                .filter(|(name, _)| {
                    !declared.contains(*name)
                        && name.as_str() != crate::vm::param_binding::KWARGS_MARKER
                })
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect();
            vm.environment_mut().define(
                rest_name.clone(),
                Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(rest))),
            );
        }
        for param in &keyword_params {
            let name = param
                .strip_prefix(crate::object::KEYWORD_PARAM_PREFIX)
                .unwrap_or(param)
                .to_string();
            let given = named
                .get(&format!(":{}", name))
                .or_else(|| named.get(&name))
                .cloned();
            let value = match given {
                Some(value) => value,
                // A keyword the call left out takes its default. A lambda
                // declaring none refuses the call, the way a method does.
                None => {
                    let index = params
                        .iter()
                        .position(|declared| declared == param)
                        .unwrap_or(usize::MAX);
                    match defaults.iter().find(|(at, _)| *at == index) {
                        Some((_, default)) => {
                            vm.evaluate_expression(default).unwrap_or(Object::Nil)
                        }
                        // A keyword declared without a default has to be
                        // given, whether the block is a lambda or a proc.
                        None => {
                            missing_keywords.push(name);
                            continue;
                        }
                    }
                }
            };
            vm.environment_mut().define(name, value);
        }
    }
    // A `**kwargs` parameter takes no positional value, so it is left out of
    // the positional binding entirely.
    let params: Vec<String> = params
        .iter()
        .filter(|param| {
            !param.starts_with("**") && !param.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
        })
        .cloned()
        .collect();
    let params = params.as_slice();
    // Find variadic param index (if any)
    let variadic_idx = params.iter().position(|p| p.starts_with('*'));
    let block_idx = params.iter().position(|p| p.starts_with('&'));
    let has_variadic = variadic_idx.is_some();

    if has_variadic {
        let vi = variadic_idx.unwrap();
        // Count non-block positional params
        let positional_params: Vec<&String> =
            params.iter().filter(|p| !p.starts_with('&')).collect();
        let params_after_splat = positional_params.len() - vi - 1;
        // A parameter with a default takes an argument only once the ones
        // that require one, on either side of the splat, have theirs.
        let optional_before: Vec<usize> = (0..vi)
            .filter(|i| {
                params
                    .iter()
                    .position(|declared| Some(declared) == positional_params.get(*i).copied())
                    .is_some_and(|at| defaults.iter().any(|(index, _)| *index == at))
            })
            .collect();
        let required_before = vi - optional_before.len();
        let min_positional = required_before + params_after_splat;
        let mut for_optionals = arguments.len().saturating_sub(min_positional);
        let mut cursor = 0usize;
        let mut taken_before: Vec<Object> = Vec::with_capacity(vi);
        for i in 0..vi {
            let optional = optional_before.contains(&i);
            if optional && for_optionals == 0 {
                let at = params
                    .iter()
                    .position(|declared| Some(declared) == positional_params.get(i).copied())
                    .unwrap_or(usize::MAX);
                let value = match defaults.iter().find(|(index, _)| *index == at) {
                    Some((_, default)) => vm.evaluate_expression(default).unwrap_or(Object::Nil),
                    None => Object::Nil,
                };
                taken_before.push(value);
                continue;
            }
            if optional {
                for_optionals -= 1;
            }
            taken_before.push(arguments.get(cursor).cloned().unwrap_or(Object::Nil));
            cursor += 1;
        }
        let splat_count = arguments.len().saturating_sub(cursor + params_after_splat);

        for (i, param) in positional_params.iter().enumerate() {
            if param.starts_with('*') && param.len() == 1 {
                continue;
            }
            let value = if i < vi {
                taken_before[i].clone()
            } else if i == vi {
                let rest: Vec<Object> = arguments
                    .get(cursor..cursor + splat_count)
                    .unwrap_or(&[])
                    .to_vec();
                Object::Array(std::rc::Rc::new(std::cell::RefCell::new(rest)))
            } else {
                let offset_from_end = positional_params.len() - i;
                let idx = arguments.len().saturating_sub(offset_from_end);
                arguments.get(idx).cloned().unwrap_or(Object::Nil)
            };
            if i == vi {
                let name = param.trim_start_matches('*').to_string();
                if name.is_empty() {
                    continue;
                }
                vm.environment_mut().define(name, value);
            } else {
                // A group written among the parameters spreads its value the
                // same way it does when no splat stands beside it.
                define_block_param(vm, param, value)?;
            }
        }
    } else {
        let positional: Vec<(usize, &String)> = params
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.starts_with('&'))
            .collect();
        for (pos, (orig_idx, param)) in positional.iter().enumerate() {
            let value = match arguments.get(pos) {
                Some(v) => v.clone(),
                None => match defaults.iter().find(|(di, _)| di == orig_idx) {
                    Some((_, default_expr)) => {
                        vm.evaluate_expression(default_expr).unwrap_or(Object::Nil)
                    }
                    None => Object::Nil,
                },
            };
            define_block_param(vm, param, value)?;
        }
    }

    // A `&name` parameter takes the block the call was handed, which is nil
    // when it was handed none.
    if let Some(bi) = block_idx {
        let name = params[bi].trim_start_matches('&').to_string();
        if !name.is_empty() {
            let given = vm.pending_block.take().unwrap_or(Object::Nil);
            vm.environment_mut().define(name, given);
        }
    }
    if !missing_keywords.is_empty() {
        let named = missing_keywords
            .iter()
            .map(|name| format!(":{name}"))
            .collect::<Vec<_>>()
            .join(", ");
        let plural = if missing_keywords.len() == 1 {
            "keyword"
        } else {
            "keywords"
        };
        return Err(crate::vm::errors::simple_exception(
            "ArgumentError",
            &format!("missing {plural}: {named}"),
            position,
        ));
    }
    Ok(())
}

/// `(**nil)` says the callable takes no keyword arguments at all, so a call
/// that hands it one is refused.
pub(crate) fn refuse_keywords_for_none_declared(
    block: &BlockStatement,
    arguments: &[Object],
    position: Position,
) -> Result<(), MetorexError> {
    if !block
        .parameters
        .iter()
        .any(|name| name == crate::object::NO_KEYWORDS_PARAM)
    {
        return Ok(());
    }
    if matches!(arguments.last(), Some(Object::Dict(entries))
        if entries.borrow().contains_key(crate::vm::param_binding::KWARGS_MARKER))
    {
        return Err(crate::vm::errors::simple_exception(
            "ArgumentError",
            "no keywords accepted",
            position,
        ));
    }
    Ok(())
}

/// A lambda refuses a call that gives it the wrong number of arguments, the
/// way a method does. A splat parameter makes the upper bound open, and a
/// parameter with a default makes the lower bound smaller.
pub(crate) fn check_lambda_arity(
    block: &BlockStatement,
    arguments: &[Object],
    position: Position,
) -> Result<(), MetorexError> {
    refuse_keywords_for_none_declared(block, arguments, position)?;
    let names = block.binding_parameters();
    let positional: Vec<&String> = names
        .iter()
        .filter(|name| {
            !name.starts_with('&')
                && !name.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
                && !name.starts_with("**")
        })
        .collect();
    if positional.iter().any(|name| name.starts_with('*')) {
        return Ok(());
    }
    let takes_keywords = names.iter().any(|name| {
        name.starts_with(crate::object::KEYWORD_PARAM_PREFIX) || name.starts_with("**")
    });
    let given = if takes_keywords && matches!(arguments.last(), Some(Object::Dict(_))) {
        arguments.len().saturating_sub(1)
    } else {
        arguments.len()
    };
    let expected = positional.len();
    let optional = block
        .parameter_defaults
        .iter()
        .filter(|(index, _)| {
            names
                .get(*index)
                .is_some_and(|name| !name.starts_with(crate::object::KEYWORD_PARAM_PREFIX))
        })
        .count();
    let required = expected.saturating_sub(optional);
    if given >= required && given <= expected {
        return Ok(());
    }
    let accepted = if required == expected {
        crate::vm::errors::Arity::Exact(expected)
    } else {
        crate::vm::errors::Arity::Range(required, expected)
    };
    Err(crate::vm::errors::argument_count_error(
        accepted, given, position,
    ))
}

/// A lambda counts its arguments the way a method does. A proc pads what is
/// missing with nil and drops what is extra, so nothing is checked for one.
pub(crate) fn strict_arity_check(
    block: &BlockStatement,
    found: usize,
    position: Position,
) -> Result<(), MetorexError> {
    if !block.is_lambda {
        return Ok(());
    }
    let parameters = block.binding_parameters();
    if parameters.iter().any(|name| name.starts_with('&')) {
        return Ok(());
    }
    // A keyword parameter and a `**rest` take their values from the keyword
    // arguments, which the positional count leaves out on both sides.
    let positional: Vec<&String> = parameters
        .iter()
        .filter(|name| {
            !name.starts_with("**") && !name.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
        })
        .collect();
    let defaults = block
        .parameter_defaults
        .iter()
        .filter(|(index, _)| {
            parameters.get(*index).is_some_and(|name| {
                !name.starts_with("**") && !name.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
            })
        })
        .count();
    // A splat takes everything past the named parameters, so only the count
    // below it is settled. The ones before it are still required.
    let splat = positional.iter().any(|name| name.starts_with('*'));
    let expected = positional
        .iter()
        .filter(|name| !name.starts_with('*'))
        .count();
    let required = expected.saturating_sub(defaults);
    if found >= required && (splat || found <= expected) {
        return Ok(());
    }
    if splat {
        return Err(crate::vm::errors::argument_count_error(
            crate::vm::errors::Arity::AtLeast(required),
            found,
            position,
        ));
    }
    let accepted = if required == expected {
        crate::vm::errors::Arity::Exact(expected)
    } else {
        crate::vm::errors::Arity::Range(required, expected)
    };
    Err(crate::vm::errors::argument_count_error(
        accepted, found, position,
    ))
}

/// A proc named by `ruby2_keywords` keeps the keyword hash its splat gathers
/// marked, so the splat can pass it on as keywords again.
pub(crate) fn marked_keyword_tail(
    block: &BlockStatement,
    mut arguments: Vec<Object>,
) -> Vec<Object> {
    if !block.ruby2_keywords.get() {
        return arguments;
    }
    let Some(Object::Dict(entries)) = arguments.last() else {
        return arguments;
    };
    if !entries
        .borrow()
        .contains_key(crate::vm::param_binding::KWARGS_MARKER)
    {
        return arguments;
    }
    let mut marked: indexmap::IndexMap<String, Object> = entries
        .borrow()
        .iter()
        .filter(|(key, _)| key.as_str() != crate::vm::param_binding::KWARGS_MARKER)
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    marked.insert(
        crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY.to_string(),
        Object::Bool(true),
    );
    arguments.pop();
    arguments.push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
        marked,
    ))));
    arguments
}

/// Whether a hash reached the block as keywords: written as keywords at the
/// call, or marked by `ruby2_keywords` to be passed on as them.
fn passed_as_keywords(
    entries: &std::rc::Rc<std::cell::RefCell<indexmap::IndexMap<String, Object>>>,
) -> bool {
    let held = entries.borrow();
    held.contains_key(crate::vm::param_binding::KWARGS_MARKER)
        || held.contains_key(crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY)
}

/// Whether a keyword hash names no keyword, once its own bookkeeping entries
/// are left out.
fn keyword_entries_are_empty(
    entries: &std::rc::Rc<std::cell::RefCell<indexmap::IndexMap<String, Object>>>,
) -> bool {
    entries.borrow().keys().all(|key| {
        matches!(
            key.as_str(),
            crate::vm::param_binding::KWARGS_MARKER
                | crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                | crate::vm::native_methods::hash_methods::KEY_OBJECTS_KEY
        )
    })
}
