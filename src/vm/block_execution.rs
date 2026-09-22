//! Block execution for the virtual machine.
//!
//! This module handles the execution of block/lambda/proc objects,
//! including scope capture, control flow, and instance_exec semantics.

use super::errors::*;
use super::utils::*;
use super::{CallFrame, ControlFlow, VirtualMachine};
use crate::ast::{Statement, collect_assigned_locals};
use crate::callable::Callable;
use crate::error::{MetorexError, StackFrame};
use crate::lexer::Position;
use crate::object::{BlockStatement, Object};

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

fn bind_block_params(
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
    if !keyword_params.is_empty() || keyword_rest.is_some() {
        let named = match arguments.last() {
            Some(Object::Dict(entries)) => {
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

impl VirtualMachine {
    /// Execute a block callable within the VM, handling scope capture and return semantics.
    pub(crate) fn execute_block_callable(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // `{ |a,| }` destructures a lone array argument across its parameters,
        // discarding any elements it has no parameter for.
        let mut destructured = false;
        let arguments = match (block.destructures_single_array(), arguments.first()) {
            (true, Some(Object::Array(elements))) if arguments.len() == 1 => {
                destructured = true;
                elements.borrow().clone()
            }
            _ => arguments,
        };

        let parameters = block.binding_parameters();
        let expected = parameters.len();
        // A trailing keyword-argument hash feeds the keyword parameters, so
        // a lambda declaring one does not count it among the positionals.
        let takes_keywords = parameters.iter().any(|name| {
            name.starts_with("**") || name.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
        });
        let found = if takes_keywords && matches!(arguments.last(), Some(Object::Dict(_))) {
            arguments.len() - 1
        } else {
            arguments.len()
        };
        let has_variadic = parameters.iter().any(|p| p.starts_with('*'));
        let has_block_param = parameters.iter().any(|p| p.starts_with('&'));

        // Optional params (`|a, b = 1|`) widen the accepted count: `found`
        // may run from `expected - defaults` up to `expected`.
        let required = expected.saturating_sub(block.parameter_defaults.len());

        // Variadic params accept any number of args; skip strict arity check.
        // Only a lambda checks arity at all: a proc pads missing arguments
        // with nil and drops extras.
        if !destructured {
            let _ = (has_variadic, has_block_param, required, expected);
            // `|**nil|` says the block takes no keyword arguments at all,
            // whether it is a lambda or a proc.
            refuse_keywords_for_none_declared(block, &arguments, position)?;
            strict_arity_check(block, found, position)?;
        }

        let frame_name = block.name().to_string();
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));

        let depth = self.block_nesting_depth();
        let frame = match block.defining_method.clone() {
            Some((callee, defined)) => CallFrame::method(
                frame_name.clone(),
                frame_location_string.clone(),
                callee,
                defined,
            ),
            // A block written outside any method is still entered from
            // somewhere, and that is the call site the frame records.
            None => CallFrame::boundary(frame_name.clone()).with_location(frame_location_string),
        }
        .nested_in_a_block(depth)
        .written_in_scope(block.written_in.clone())
        .with_source_file(self.current_source_file.clone());
        // The body runs in the file the block was written in, which is what a
        // backtrace entry for a call made from here has to name.
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        // A literal in the body is written in the encoding the block's own
        // file names, not the one the file calling it names.
        let saved_source_encoding = std::mem::replace(
            &mut self.current_source_encoding,
            self.current_source_file
                .as_ref()
                .and_then(|named| self.file_encodings.get(named).cloned()),
        );
        let execution_result =
            self.with_call_frame(frame, move |vm| vm.execute_block_body(block, arguments));
        self.current_source_file = saved_source_file;
        self.current_source_encoding = saved_source_encoding;

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        }
    }

    /// Run Ruby source with `self` bound to `receiver`, which is what the
    /// String form of `instance_eval` does. The source sees the receiver's
    /// instance variables and defines methods on its singleton class.
    /// Run Ruby source against `receiver`, counted as written in `named` from
    /// `lineno` on. The code sees the locals of the scope it was written in,
    /// so an assignment there reaches the caller's own name.
    pub(crate) fn evaluate_source_named_with_receiver(
        &mut self,
        source: &str,
        receiver: Object,
        named: Option<String>,
        lineno: i64,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let tokens =
            crate::lexer::Lexer::with_start_line(source, lineno.max(1) as usize).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .map_err(|errors| {
                let reported = errors
                    .iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                let file = named.clone().unwrap_or_default();
                crate::vm::errors::syntax_error(
                    format!("{file}: {reported}"),
                    Some(&file),
                    position,
                )
            })?;
        let singleton = self.singleton_class_of(&receiver);
        // The code runs in a scope of its own that sees the caller's locals,
        // so a name it assigns is the caller's own name.
        let carried: Vec<(String, std::rc::Rc<std::cell::RefCell<Object>>)> = self
            .environment()
            .binding_variable_names()
            .into_iter()
            .filter_map(|name| self.environment().get_ref(&name).map(|cell| (name, cell)))
            .collect();
        let previous_file = self.current_file.clone();
        let previous_source_file = self.current_source_file.clone();
        // Code with no file named for it is counted as written where the
        // call was made, which is what `__FILE__` answers inside it.
        let named = named.or_else(|| {
            let written_in = previous_source_file
                .clone()
                .or_else(|| {
                    previous_file
                        .as_ref()
                        .map(|held| held.display().to_string())
                })
                .unwrap_or_default();
            Some(format!(
                "{}{}:{})",
                crate::vm::EVAL_FILE_PREFIX,
                written_in,
                position.line
            ))
        });
        if let Some(file) = &named {
            self.current_file = Some(std::path::PathBuf::from(file));
            self.current_source_file = Some(file.clone());
        }
        self.environment_mut().push_isolated_scope();
        for (name, cell) in carried {
            self.environment_mut().define_inherited(name, cell);
        }
        self.environment_mut()
            .define("self".to_string(), receiver.clone());
        // A line counted from below one is still counted from there, which a
        // backtrace reports rather than the line the lexer could start at.
        let previous_shift = std::mem::replace(&mut self.source_line_shift, lineno.min(1) - 1);
        // The code reads constants from the receiver's singleton class first
        // and from the receiver's own class next, ahead of the scopes the
        // caller was written in.
        let receiver_class = match &receiver {
            Object::Class(held) | Object::Module(held) => Some(std::rc::Rc::clone(held)),
            Object::Instance(held) => Some(std::rc::Rc::clone(&held.borrow().class)),
            _ => None,
        };
        let mut opened = 1;
        let mut nesting = Vec::new();
        nesting.push(std::rc::Rc::clone(&singleton));
        if let Some(held) = receiver_class {
            nesting.push(std::rc::Rc::clone(&held));
            self.def_scope_stack.push(held);
            opened += 1;
        }
        self.def_scope_stack.push(singleton);
        // The scopes open around the code are the receiver's, ahead of the
        // ones the caller was written in, which is the order a name written
        // there is looked up through.
        if let Some(held) = self.method_nesting_stack.last() {
            nesting.extend(held.iter().map(std::rc::Rc::clone));
        }
        self.method_nesting_stack.push(nesting);
        let mut last = Object::Nil;
        let result = (|| -> Result<(), MetorexError> {
            for statement in &statements {
                if let Statement::Expression { expression, .. } = statement {
                    if !self.tracepoints.is_empty() {
                        self.fire_line_event(statement.position())?;
                    }
                    last = self.evaluate_expression(expression)?;
                    continue;
                }
                match self.execute_statement(statement)? {
                    ControlFlow::Value(value) => last = value,
                    // An exception raised here is the caller's to handle,
                    // rather than something the run swallows.
                    ControlFlow::Exception {
                        exception,
                        position,
                    } => {
                        return Err(MetorexError::UncaughtException {
                            message: crate::vm::utils::format_exception(&exception),
                            exception,
                            location: position_to_location(position),
                        });
                    }
                    _ => {}
                }
            }
            Ok(())
        })();
        for _ in 0..opened {
            self.def_scope_stack.pop();
        }
        self.method_nesting_stack.pop();
        self.environment_mut().pop_scope();
        self.source_line_shift = previous_shift;
        self.current_file = previous_file;
        self.current_source_file = previous_source_file;
        result?;
        Ok(last)
    }

    /// Execute a block with a specific `self` receiver (for instance_exec/instance_eval).
    /// The receiver overrides any captured `self` from the block's closure.
    pub(crate) fn execute_block_with_receiver(
        &mut self,
        block: &BlockStatement,
        receiver: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let frame_name = block.name().to_string();
        let frame_location = position_to_location(position);
        let frame_location_string = Some(format!("{}", frame_location));
        let frame = match block.defining_method.clone() {
            Some((callee, defined)) => {
                CallFrame::method(frame_name.clone(), frame_location_string, callee, defined)
            }
            // A block run against another receiver still sits where the call
            // was made, which is what a backtrace entry for it names.
            None => {
                CallFrame::boundary(frame_name.clone()).with_location(frame_location_string.clone())
            }
        }
        .nested_in_a_block(self.block_nesting_depth())
        .written_in_scope(block.written_in.clone())
        .with_source_file(self.current_source_file.clone());
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        // A literal in the body is written in the encoding the block's own
        // file names, not the one the file calling it names.
        let saved_source_encoding = std::mem::replace(
            &mut self.current_source_encoding,
            self.current_source_file
                .as_ref()
                .and_then(|named| self.file_encodings.get(named).cloned()),
        );
        // A `def` in the body defines a method on the receiver alone, and a
        // class variable written there belongs to the class or module the
        // block was written in.
        // The class or module the block was written in. A block written in a
        // method body belongs to the class holding that method, whatever
        // scope was open where the method was called from.
        let lexical = block
            .captured_nesting
            .first()
            .cloned()
            .or_else(|| self.method_owner_stack.last().cloned().flatten())
            .or_else(|| block.captured_def_scope.last().cloned())
            .or_else(|| {
                self.method_nesting_stack
                    .last()
                    .and_then(|nesting| nesting.last().cloned())
            })
            .or_else(|| self.def_scope_stack.last().cloned())
            .or_else(|| match self.environment().get("self") {
                Some(Object::Class(held) | Object::Module(held)) => Some(held),
                _ => None,
            });

        let carried = lexical.is_some();
        if let Some(home) = lexical {
            self.class_var_home.push(home);
        }
        // A class variable written in the body belongs to the class or
        // module the block was written in, whatever it runs against.
        self.class_var_cref_stack.push(
            block
                .captured_nesting
                .first()
                .cloned()
                .or_else(|| block.captured_def_scope.last().cloned()),
        );
        // The scopes open around the body are the ones the block was written
        // in, whatever the receiver it runs against, which is what
        // `Module.nesting` there reports.
        self.method_nesting_stack
            .push(block.captured_nesting.clone());
        // A `def` or an `alias` written at the top of the body belongs to the
        // receiver alone. Anything the body calls out to keeps its own
        // definee, so this reaches only the statements written here.
        let definee = block
            .body()
            .iter()
            .any(|statement| {
                matches!(
                    statement,
                    Statement::MethodDef { .. }
                        | Statement::FunctionDef { .. }
                        | Statement::Alias { .. }
                )
            })
            .then(|| self.singleton_class_of(&receiver));
        // A value the program cannot hold one copy of has no singleton class
        // to write the method on.
        if definee.is_some()
            && crate::vm::native_methods::object_methods::refuses_a_singleton(&receiver)
        {
            let message = "can't define singleton".to_string();
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: position_to_location(position),
                message,
            });
        }
        let execution_result = self.with_call_frame(frame, move |vm| {
            vm.environment_mut().push_isolated_scope();
            let result = (|| -> Result<Object, MetorexError> {
                for (name, value_ref) in block.captured_vars() {
                    vm.environment_mut()
                        .define_captured(name.clone(), value_ref.clone());
                }
                // Override `self` with the instance_exec receiver
                vm.environment_mut().define("self".to_string(), receiver);

                bind_block_params(
                    vm,
                    block.parameters(),
                    &block.parameter_defaults,
                    arguments,
                    Position::new(0, 0, 0),
                )?;

                // Pre-bind syntactically assigned locals to nil (Ruby's
                // parser-level local hoisting) so an `ensure`/`rescue`
                // clause that reads a variable defined later in the body
                // returns nil instead of NameError when execution
                // short-circuits via raise.
                for name in collect_assigned_locals(block.body()) {
                    if vm.environment().assignment_introduces_a_local(&name) {
                        vm.environment_mut().hoist(name);
                    }
                }

                let mut last_value;
                // `redo` runs the block's body again over the same arguments.
                'again: loop {
                    last_value = Object::Nil;
                    for statement in block.body() {
                        // `def self.name` written here names the receiver the
                        // block runs against, and that receiver's singleton
                        // class is the definee, so the method is written there
                        // under its own name rather than as a class method of
                        // the singleton.
                        let written_plainly = statement_for_its_own_receiver(statement);
                        if let Some(definee) = &definee
                            && let Some(plain) = &written_plainly
                        {
                            let definee = std::rc::Rc::clone(definee);
                            last_value = vm.apply_class_body_statements(
                                &definee,
                                std::slice::from_ref(plain),
                                statement.position(),
                            )?;
                            continue;
                        }
                        if let Some(definee) = &definee
                            && written_plainly.is_none()
                            && matches!(
                                statement,
                                Statement::MethodDef { .. }
                                    | Statement::FunctionDef { .. }
                                    | Statement::Alias { .. }
                            )
                        {
                            let definee = std::rc::Rc::clone(definee);
                            last_value = vm.apply_class_body_statements(
                                &definee,
                                std::slice::from_ref(statement),
                                statement.position(),
                            )?;
                            continue;
                        }
                        if let Statement::Expression { expression, .. } = statement {
                            if !vm.tracepoints.is_empty() {
                                vm.fire_line_event(statement.position())?;
                            }
                            last_value = vm.evaluate_expression(expression)?;
                            continue;
                        }
                        match vm.execute_statement(statement)? {
                            ControlFlow::Next => {}
                            ControlFlow::Value(value) => {
                                last_value = value;
                            }
                            ControlFlow::Return { value, .. } => {
                                last_value = value;
                                break 'again;
                            }
                            ControlFlow::Exception {
                                exception,
                                position,
                            } => {
                                return Err(MetorexError::UncaughtException {
                                    exception: exception.clone(),
                                    location: position_to_location(position),
                                    message: format_exception(&exception),
                                });
                            }
                            ControlFlow::Break { value, position } => {
                                // The break belongs to the call that was
                                // handed this block, which is one made from
                                // the frame the block was written in.
                                return Err(MetorexError::BlockBreak {
                                    value,
                                    location: position_to_location(position),
                                    home_frame: block.home_frame,
                                });
                            }
                            ControlFlow::Redo { .. } | ControlFlow::Retry { .. } => continue 'again,
                            ControlFlow::Continue { position, .. } => {
                                return Err(loop_control_error("continue", position));
                            }
                        }
                    }
                    break;
                }
                Ok(last_value)
            })();
            vm.environment_mut().pop_scope();
            result
        });
        self.method_nesting_stack.pop();
        self.class_var_cref_stack.pop();
        if carried {
            self.class_var_home.pop();
        }
        self.current_source_file = saved_source_file;
        self.current_source_encoding = saved_source_encoding;

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(frame_name, frame_location))),
        }
    }

    /// Execute the statements inside a block object with its captured scope.
    pub(crate) fn execute_block_body(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        // A block body runs in its own scope seeded from `captured_vars`, not
        // chained to whatever method happens to be invoking it. Ruby's block
        // sees the locals of the scope it was written in, and nothing of its
        // caller's.
        self.environment_mut().push_isolated_scope();
        // Restore the lexical class/module nesting from the block's
        // definition site so an uppercase `Foo = ...` inside the body
        // assigns to the same enclosing module the surrounding code would
        // have. Saved/restored in pure stack fashion in case the caller's
        // current def_scope_stack is non-empty (e.g. block invoked from
        // inside a class body).
        let saved_def_scope =
            std::mem::replace(&mut self.def_scope_stack, block.captured_def_scope.clone());
        // A block opened inside this body belongs to the method this one was
        // written in, so a `return` written two blocks deep unwinds to the
        // method holding them both.
        let saved_lexical_home = self.lexical_home_frame.replace(block.home_frame);
        // The body belongs to the file the block was written in, whatever
        // file called it, and a literal in it is written in that file's
        // encoding.
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        let saved_source_encoding = std::mem::replace(
            &mut self.current_source_encoding,
            self.current_source_file
                .as_ref()
                .and_then(|named| self.file_encodings.get(named).cloned()),
        );
        // A class variable written in the body belongs to the class or module
        // the block was written in, which is none at the top level.
        self.class_var_cref_stack.push(
            block
                .captured_nesting
                .first()
                .cloned()
                .or_else(|| block.captured_def_scope.last().cloned()),
        );
        // The scopes open around the block are the ones it was written in,
        // not the ones open in the method that called it, which is what
        // `Module.nesting` in the body reports and where an `eval` written
        // there opens what it defines.
        self.method_nesting_stack
            .push(block.captured_nesting.clone());

        // A trace sees a block body opening and closing, and reads the
        // parameters the block declared off either event.
        let block_position = block
            .body
            .first()
            .map(|held| held.position())
            .unwrap_or_else(|| Position::new(0, 0, 0));
        let declared = crate::vm::native_methods::block_parameter_list(block);
        self.fire_event(
            "b_call",
            block_position,
            vec![("parameters", declared.clone())],
        )?;

        // A lambda is something a `return` carried out of an eval can return
        // from, so its body says while it runs that one is there.
        if block.is_lambda {
            self.lambda_body_depth += 1;
        }
        let result = (|| -> Result<Object, MetorexError> {
            // Define captured variables using shared references
            for (name, value_ref) in block.captured_vars() {
                self.environment_mut()
                    .define_captured(name.clone(), value_ref.clone());
            }

            // A lambda takes its arguments the way a method does, so the
            // count has to match what it declared.
            if block.is_lambda {
                check_lambda_arity(block, &arguments, Position::new(0, 0, 0))?;
            }

            // Define parameters as regular variables (handles *args/&block prefixes)
            let arguments = marked_keyword_tail(block, arguments);
            bind_block_params(
                self,
                &block.binding_parameters(),
                &block.parameter_defaults,
                arguments,
                Position::new(0, 0, 0),
            )?;

            // A name written after the `;` in the parameter list is a local
            // of the block, which starts as nil however the outer scope reads.
            for name in block.block_locals() {
                self.environment_mut().define(name, Object::Nil);
            }

            // Pre-define every local syntactically assigned-to in this block
            // body as `nil`, so a read that runs before its assignment line
            // (e.g. inside an `ensure` clause that fires after an early raise)
            // returns nil rather than raising NameError. Mirrors Ruby's
            // parser-level local-variable hoisting.
            for name in collect_assigned_locals(block.body()) {
                if self.environment().assignment_introduces_a_local(&name) {
                    self.environment_mut().hoist(name);
                }
            }

            let mut last_value;

            // `redo` runs the block's body again over the same arguments.
            'again: loop {
                last_value = Object::Nil;
                for statement in block.body() {
                    if let Statement::Expression { expression, .. } = statement {
                        if !self.tracepoints.is_empty() {
                            self.fire_line_event(statement.position())?;
                        }
                        last_value = self.evaluate_expression(expression)?;
                        continue;
                    }

                    match self.execute_statement(statement)? {
                        ControlFlow::Next => {}
                        ControlFlow::Value(value) => {
                            last_value = value;
                        }
                        // `return` in a lambda returns from the lambda. In a
                        // proc or ordinary block it is a long return: it unwinds
                        // to the method that lexically created the block.
                        ControlFlow::Return { value, position } => {
                            if block.is_lambda {
                                last_value = value;
                                break 'again;
                            }
                            // The invocation the block was written in may have
                            // returned already, and then the return has nowhere
                            // to go.
                            if let Some(home) = block.home_frame
                                && !self.live_frames.contains(&home)
                            {
                                return Err(orphaned_return_error(value, position));
                            }
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: position_to_location(position),
                                home_frame: block.home_frame,
                            });
                        }
                        ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::UncaughtException {
                                exception: exception.clone(),
                                location: position_to_location(position),
                                message: format_exception(&exception),
                            });
                        }
                        ControlFlow::Break { value, position } => {
                            // `break` in a lambda ends the lambda, the way a
                            // `return` written there does.
                            if block.is_lambda {
                                last_value = value;
                                break 'again;
                            }
                            // Ruby: `break <value>` inside a block unwinds to the
                            // method that received the block, returning `value`
                            // from that method call. Uses BlockBreak so the signal
                            // survives `execute_method_body` (which only swallows
                            // NonLocalReturn) and is caught at the invoke boundary.
                            return Err(MetorexError::BlockBreak {
                                value,
                                location: position_to_location(position),
                                home_frame: None,
                            });
                        }
                        ControlFlow::Redo { .. } | ControlFlow::Retry { .. } => continue 'again,
                        // `next <value>` ends this run of the block with that
                        // value, which is what the method holding the block sees.
                        ControlFlow::Continue { value, .. } => {
                            last_value = value;
                            break 'again;
                        }
                    }
                }
                break;
            }

            Ok(last_value)
        })();

        if block.is_lambda {
            self.lambda_body_depth -= 1;
        }
        self.environment_mut().pop_scope();
        self.class_var_cref_stack.pop();
        self.method_nesting_stack.pop();
        self.def_scope_stack = saved_def_scope;
        self.lexical_home_frame = saved_lexical_home;
        // The exception says where it came from while the block's own file is
        // still the one in force, which is what a backtrace entry names.
        if let Err(MetorexError::UncaughtException {
            exception,
            location,
            ..
        }) = &result
        {
            self.note_exception_location(exception, location);
        }
        self.current_source_file = saved_source_file;
        self.current_source_encoding = saved_source_encoding;
        // A `break` leaving this body belongs to the invocation that was
        // handed the block, which is the call made from the frame the block
        // was written in.
        match result {
            // A `return` carried out of an eval stops at the lambda it was
            // written inside, and passing out of any other block leaves it
            // looking for the invocation it belongs to.
            Err(MetorexError::NonLocalReturn {
                value,
                location,
                home_frame,
            }) => {
                if block.is_lambda && home_frame.is_none() {
                    return Ok(value);
                }
                // Nothing is left to return from once the return has passed
                // out of every block, with no lambda around it and no method
                // running, which is what makes it a LocalJumpError.
                let nowhere_to_return_to = match block.home_frame {
                    // A block written outside any method belongs to the unit
                    // itself, which is not something to return from.
                    Some(home) => {
                        home == crate::vm::core::TOP_LEVEL_FRAME
                            || !self.live_frames.contains(&home)
                    }
                    None => true,
                };
                if home_frame.is_none() && self.lambda_body_depth == 0 && nowhere_to_return_to {
                    return Err(escaped_return_error(value, location));
                }
                Err(MetorexError::NonLocalReturn {
                    value,
                    location,
                    home_frame,
                })
            }
            // `next` written inside an expression unwinds to here, and ends
            // this run of the block with the value it carried.
            Err(MetorexError::BlockNext { value, .. }) => Ok(value),
            Err(MetorexError::BlockBreak {
                value,
                location,
                home_frame: None,
            }) => Err(MetorexError::BlockBreak {
                value,
                location,
                home_frame: block.home_frame,
            }),
            other => other,
        }
    }

    /// Execute a block body and return ControlFlow (for use in iterators like .each)
    /// This version propagates Break/Continue instead of converting them to errors
    pub(crate) fn execute_block_with_control_flow(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // `{ |x, y| }` handed a single array spreads it across the parameters,
        // which is how `[[1, 2]].each { |x, y| }` binds x and y.
        let arguments = match (block.destructures_single_array(), arguments.first()) {
            (true, Some(Object::Array(elements))) if arguments.len() == 1 => {
                elements.borrow().clone()
            }
            _ => arguments,
        };
        // A lambda takes its arguments the way a method does, so a yield that
        // does not match its parameters is refused rather than padded.
        strict_arity_check(block, arguments.len(), Position::new(0, 0, 0))?;
        // A block body is a place of its own in a backtrace, named for the
        // scope it was written in and for how many blocks deep it sits.
        let frame_name = block.name().to_string();
        let depth = self.block_nesting_depth();
        // A block reached from a method is entered where that method was
        // called, which is the call site the frame records.
        let called_at = Some(format!("{}", position_to_location(position)));
        let frame = match block.defining_method.clone() {
            Some((callee, defined)) => {
                CallFrame::method(frame_name.clone(), called_at, callee, defined)
            }
            None => CallFrame::boundary(frame_name.clone()).with_location(called_at),
        }
        .nested_in_a_block(depth)
        .written_in_scope(block.written_in.clone())
        .with_source_file(self.current_source_file.clone());
        self.call_stack_push(frame);
        self.environment_mut().push_isolated_scope();
        // The body belongs to the file the block was written in.
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        // A literal in the body is written in the encoding the block's own
        // file names, not the one the file calling it names.
        let saved_source_encoding = std::mem::replace(
            &mut self.current_source_encoding,
            self.current_source_file
                .as_ref()
                .and_then(|named| self.file_encodings.get(named).cloned()),
        );

        let result = (|| -> Result<ControlFlow, MetorexError> {
            // Define captured variables using shared references
            for (name, value_ref) in block.captured_vars() {
                self.environment_mut()
                    .define_captured(name.clone(), value_ref.clone());
            }

            // Define parameters as regular variables (handles *args/&block prefixes)
            bind_block_params(
                self,
                &block.binding_parameters(),
                &block.parameter_defaults,
                arguments,
                Position::new(0, 0, 0),
            )?;

            // Pre-bind syntactically assigned locals to nil — see
            // execute_block_body for the rationale.
            for name in collect_assigned_locals(block.body()) {
                if self.environment().assignment_introduces_a_local(&name) {
                    self.environment_mut().hoist(name);
                }
            }

            // `redo` runs the block's body again over the same arguments,
            // which are already bound in this scope.
            loop {
                let mut again = false;
                for statement in block.body() {
                    match self.execute_statement(statement)? {
                        ControlFlow::Next | ControlFlow::Value(_) => {}
                        ControlFlow::Retry { .. } | ControlFlow::Redo { .. } => {
                            again = true;
                            break;
                        }
                        flow @ (ControlFlow::Return { .. }
                        | ControlFlow::Break { .. }
                        | ControlFlow::Continue { .. }
                        | ControlFlow::Exception { .. }) => {
                            return Ok(flow);
                        }
                    }
                }
                if !again {
                    return Ok(ControlFlow::Next);
                }
            }
        })();

        self.current_source_file = saved_source_file;
        self.current_source_encoding = saved_source_encoding;
        self.environment_mut().pop_scope();
        self.call_stack_pop();
        result
    }
}

/// The same definition written without the `self.` that named the receiver.
/// A block running against a receiver has that receiver's singleton class as
/// its definee, so the method belongs there under its own name.
fn statement_for_its_own_receiver(statement: &Statement) -> Option<Statement> {
    match statement {
        Statement::FunctionDef {
            name,
            parameters,
            body,
            singleton_class: Some(named),
            position,
        } if named == "self" => Some(Statement::FunctionDef {
            name: name.clone(),
            parameters: parameters.clone(),
            body: body.clone(),
            singleton_class: None,
            position: *position,
        }),
        Statement::MethodDef {
            name,
            parameters,
            body,
            is_class_method: true,
            position,
            end_position,
        } => Some(Statement::MethodDef {
            name: name.clone(),
            parameters: parameters.clone(),
            body: body.clone(),
            is_class_method: false,
            position: *position,
            end_position: *end_position,
        }),
        _ => None,
    }
}

/// A `return` that left a block without finding the lambda or the method it
/// belongs to, which Ruby reports the same way as one whose method has
/// already returned.
pub(crate) fn escaped_return_error(
    value: Object,
    location: crate::error::SourceLocation,
) -> MetorexError {
    let message = "unexpected return".to_string();
    let exception = Object::exception("LocalJumpError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details
            .instance_vars
            .insert("@exit_value".to_string(), value);
        details
            .instance_vars
            .insert("@reason".to_string(), Object::symbol("return".to_string()));
    }
    MetorexError::UncaughtException {
        exception,
        location,
        message,
    }
}

/// A `return` from a block whose defining method has already returned. Ruby
/// reports it as a LocalJumpError carrying the value and the reason.
fn orphaned_return_error(value: Object, position: Position) -> MetorexError {
    let message = "unexpected return".to_string();
    let exception = Object::exception("LocalJumpError", message.clone());
    if let Object::Exception(details) = &exception {
        let mut details = details.borrow_mut();
        details
            .instance_vars
            .insert("@exit_value".to_string(), value);
        details
            .instance_vars
            .insert("@reason".to_string(), Object::symbol("return".to_string()));
    }
    MetorexError::UncaughtException {
        exception,
        location: position_to_location(position),
        message,
    }
}

/// `(**nil)` says the callable takes no keyword arguments at all, so a call
/// that hands it one is refused.
fn refuse_keywords_for_none_declared(
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
fn check_lambda_arity(
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
fn strict_arity_check(
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
fn marked_keyword_tail(block: &BlockStatement, mut arguments: Vec<Object>) -> Vec<Object> {
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
