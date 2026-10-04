// Expression-evaluation dispatch: a thin `match` over `Expression` variants
// that delegates each variant to a helper in the sibling modules.

use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{BinaryOp, Expression};
use crate::error::MetorexError;
use crate::object::{BlockStatement, Object};

use crate::vm::core::VirtualMachine;
use crate::vm::utils::{is_truthy, position_to_location};

impl VirtualMachine {
    /// Dispatch over an expression variant. Each branch is small; large branches
    /// delegate to a helper in `vm/eval/`.
    pub(crate) fn evaluate_expression_inner(
        &mut self,
        expression: &Expression,
    ) -> Result<Object, MetorexError> {
        match expression {
            // ── Literals ────────────────────────────────────────────────────
            Expression::IntLiteral { value, .. } => Ok(Object::Int(*value)),
            Expression::FloatLiteral { value, .. } => Ok(Object::Float(*value)),
            Expression::StringLiteral { value, position } => {
                // A literal is written in the encoding its source is written
                // in, which a magic comment at the top of the file names.
                // A literal written in bytes carries its own encoding, so one
                // reaching here with anything outside ASCII was spelled with a
                // `\u` escape and stands for those codepoints.
                let made = match self.source_literal_encoding() {
                    Some(named) if value.is_ascii() => {
                        crate::object::StringValue::with_encoding(value.clone(), named)
                    }
                    // A file written in an encoding of its own spells a
                    // character with bytes of that encoding, so the literal
                    // carries those bytes rather than the UTF-8 the source was
                    // read into.
                    Some(named)
                        if let Some(bytes) =
                            crate::vm::native_methods::string_methods::spelled_bytes(
                                value, &named,
                            )
                            .or_else(|| crate::file_loader::escaped_source_bytes(value)) =>
                    {
                        let held = crate::object::StringValue::with_encoding(
                            crate::vm::native_methods::string_methods::bytes_as_text(&bytes),
                            named,
                        );
                        held.mark_bytes();
                        held
                    }
                    _ => crate::object::StringValue::new(value.clone()),
                };
                // Ruby 3.4 hands a literal back with notice that a later
                // release will freeze it, so the first change made to it says
                // so. A source that asked for frozen literals produces a
                // frozen one instead and never reaches here.
                // A run told to report where a literal was written names the
                // place instead of pointing at the flag that would name it.
                match self.literal_birthplace(*position) {
                    Some(written_at) => {
                        made.set_created_at(written_at);
                        made.chill(
                            "warning: literal string will be frozen in the future".to_string(),
                        );
                    }
                    None => made.chill(
                        "warning: literal string will be frozen in the future (run with \
--debug-frozen-string-literal for more information)"
                            .to_string(),
                    ),
                }
                let made = Object::String(std::rc::Rc::new(made));
                self.record_allocation(&made, *position);
                Ok(made)
            }
            Expression::Symbol { value, .. } => {
                // A symbol is named in the encoding the source naming it is
                // written in, which is what `Symbol#encoding` reports for one
                // whose name is not all ASCII.
                let named = self
                    .current_source_encoding
                    .clone()
                    .map(|held| {
                        crate::vm::native_methods::string_methods::canonical_encoding_name(&held)
                    })
                    .unwrap_or_else(|| crate::object::string_value::DEFAULT_ENCODING.to_string());
                // A source written in bytes names its symbols in bytes, so
                // each character of the name stands for one of them.
                // A source written in an encoding of its own names its
                // symbols in the bytes that encoding spells them with.
                let plain = named == crate::object::string_value::DEFAULT_ENCODING;
                if plain || value.is_ascii() {
                    let made = crate::object::StringValue::with_encoding(value.clone(), named);
                    return Ok(Object::Symbol(std::rc::Rc::new(made)));
                }
                // A source read as bytes names its symbols in the bytes the
                // file itself holds, which is the text as it was written.
                let spelled = if matches!(named.as_str(), "ASCII-8BIT" | "BINARY") {
                    Some(value.as_bytes().to_vec())
                } else {
                    crate::vm::native_methods::string_methods::spelled_bytes(value, &named)
                };
                let Some(bytes) = spelled else {
                    let made = crate::object::StringValue::with_encoding(value.clone(), named);
                    return Ok(Object::Symbol(std::rc::Rc::new(made)));
                };
                let made = crate::object::StringValue::with_encoding(
                    crate::vm::native_methods::string_methods::bytes_as_text(&bytes),
                    named,
                );
                made.mark_bytes();
                Ok(Object::Symbol(std::rc::Rc::new(made)))
            }
            Expression::RegexLiteral {
                pattern,
                flags,
                position,
            } => {
                // A repetition written on a repetition reads as one, which
                // Ruby says so about where the pattern is read.
                if pattern.contains("}+")
                    && let Ok(read) =
                        crate::vm::native_methods::regexp_methods::read_pattern(pattern, flags)
                {
                    for warning in &read.warnings {
                        self.emit_warning_to_stderr(&format!("warning: {warning}"), *position);
                    }
                }
                Ok(Object::Regex(
                    Rc::new(pattern.clone()),
                    Rc::new(flags.clone()),
                ))
            }
            Expression::BoolLiteral { value, .. } => Ok(Object::Bool(*value)),
            Expression::NilLiteral { .. } => Ok(Object::Nil),
            Expression::InterpolatedString { parts, .. } => {
                self.evaluate_interpolated_object(parts)
            }

            // ── Variables / identifiers ─────────────────────────────────────
            Expression::Identifier { name, position } => {
                let value = self.eval_identifier(name, *position)?;
                if name.starts_with(|first: char| first.is_ascii_uppercase())
                    && self.in_a_non_main_ractor()
                {
                    let (owner, found_where_written) = self.constant_owner_name(name);
                    self.refuse_unshareable_constant(
                        &format!("{owner}::{name}"),
                        found_where_written,
                        &value,
                        *position,
                    )?;
                }
                Ok(value)
            }
            Expression::SelfExpr { position } => self.eval_self(*position),
            Expression::InstanceVariable { name, position } => {
                self.eval_instance_var_read(name, *position)
            }
            Expression::ClassVariable { name, position } => {
                self.eval_class_var_read(name, *position)
            }
            Expression::GlobalVariable { name, position } => {
                self.read_global_variable(name, *position)
            }
            Expression::MagicFile { .. } => {
                // The file the code was written in, which is not the file
                // being run when a required file's method or block is what is
                // executing.
                let path = self
                    .current_source_file
                    .clone()
                    .or_else(|| {
                        self.reported_current_file()
                            .map(|path| path.display().to_string())
                    })
                    .unwrap_or_else(|| "(eval)".to_string());
                Ok(Object::string(path))
            }
            // `value => pattern` refuses a value the pattern does not cover,
            // and `value in pattern` answers whether it does.
            Expression::PatternTest {
                value,
                pattern,
                refuses,
                position,
            } => {
                let held = self.evaluate_expression(value)?;
                let mut bindings = std::collections::HashMap::new();
                self.pattern_failure = None;
                let matched = self.match_pattern(pattern, &held, &mut bindings, *position)?;
                if matched {
                    self.apply_pattern_bindings(&bindings);
                } else if *refuses {
                    return Err(self.detailed_no_matching_pattern(&held, *position));
                }
                Ok(if *refuses {
                    Object::Nil
                } else {
                    Object::Bool(matched)
                })
            }
            Expression::MagicLine { position, .. } => Ok(Object::Int(position.line as i64)),
            Expression::MagicDir { .. } => {
                // `__dir__` is nil where there is no file behind the code, and
                // otherwise the directory holding it, which is "." for a bare
                // filename the way `File.dirname` reports it.
                // The directory is the one holding the file the code was
                // written in, which is what `__FILE__` names, rather than the
                // file being run when a required file's block is executing.
                let written_in = self
                    .current_source_file
                    .clone()
                    .map(std::path::PathBuf::from)
                    .or_else(|| self.get_current_file().cloned());
                let Some(file) = written_in else {
                    return Ok(Object::Nil);
                };
                // Code handed to `eval` with no filename of its own is named
                // for the place the eval was written, which is not a file and
                // so holds no directory.
                if file
                    .to_str()
                    .is_some_and(|named| named.starts_with(crate::vm::EVAL_FILE_PREFIX))
                {
                    return Ok(Object::Nil);
                }
                let file = file.as_path();
                // A relative path expands against the working directory, so
                // running `metorex script.rb` still names the real directory.
                let directory = match std::fs::canonicalize(file) {
                    Ok(resolved) => match resolved.parent() {
                        Some(parent) => parent.display().to_string(),
                        None => ".".to_string(),
                    },
                    Err(_) => match file.parent().map(|d| d.display().to_string()) {
                        Some(directory) if !directory.is_empty() => directory,
                        _ => ".".to_string(),
                    },
                };
                Ok(Object::string(directory))
            }

            // ── Closures, grouping ──────────────────────────────────────────
            Expression::Lambda {
                parameters,
                parameter_defaults,
                body,
                captured_vars,
                outer_locals,
                is_lambda,
                position,
                ..
            } => {
                let mut captured = HashMap::new();
                if let Some(names) = captured_vars {
                    if names.is_empty() {
                        // Empty vec signals automatic capture of all current scope variables.
                        // This is used for true lambdas (lambda do ... end, arrow syntax).
                        captured = self.environment().current_scope_var_refs();
                    } else {
                        // Explicit list of variables to capture
                        for name in names {
                            if let Some(value_ref) = self.environment().get_ref(name) {
                                captured.insert(name.clone(), value_ref);
                            }
                        }
                    }
                }
                // If captured_vars is None, capture all current scope variables
                // so blocks work correctly across method boundaries (which use
                // isolated scopes).
                if captured.is_empty() && captured_vars.is_none() {
                    captured = self.environment().current_scope_var_refs();
                }
                let mut block = BlockStatement::with_def_scope(
                    parameters.clone(),
                    parameter_defaults.clone(),
                    body.clone(),
                    captured,
                    self.def_scope_stack.clone(),
                    self.enclosing_method_names(),
                    *is_lambda,
                );
                block.defining_owner = self.enclosing_method_owner().and_then(|(owner, _)| owner);
                block.outer_locals = Rc::new(outer_locals.clone());
                // The block's body belongs to the file it was written in,
                // wherever it is later called from.
                // A block the core library opens stands in for Ruby's C code,
                // which names no source location at all.
                if !position.prelude {
                    // The main script is named the way it was given, as
                    // `__FILE__` names it.
                    block.source_file = self.current_source_file.clone().or_else(|| {
                        self.reported_current_file()
                            .map(|path| path.display().to_string())
                    });
                    // Where the block was opened, which is the line
                    // `source_location` names however far down the body starts.
                    block.opened_at = Some(position.line);
                    if let Some(file) = block.source_file.clone() {
                        let mut within = self
                            .running_code
                            .last()
                            .map(|running| running.within.as_ref().clone())
                            .unwrap_or_default();
                        within.push((file, position.line));
                        block.written_within = Rc::new(within);
                    }
                }
                // The scopes open here are what the body reads its own
                // lexical nesting as, whatever scope it is later called from.
                block.captured_nesting = match self.method_nesting_stack.last() {
                    Some(captured) => captured.clone(),
                    None => self.snapshot_lexical_nesting(),
                };
                block.home_frame = self.lexical_home_frame.unwrap_or(self.current_method_frame);
                block.written_in = Some(self.enclosing_scope_label());
                block.written_depth = Some(self.block_nesting_depth());
                Ok(Object::Block(Rc::new(block)))
            }
            Expression::Grouped { expression, .. } => self.evaluate_expression(expression),

            // ── Operators ───────────────────────────────────────────────────
            Expression::UnaryOp {
                op,
                operand,
                position,
            } => {
                let value = self.evaluate_expression(operand)?;
                // An object that defines `-@`, `+@`, or `!` names what the
                // operator in front of it does, which is how a number a
                // program writes negates and how a delegator passes `!` on.
                if let Object::Instance(_) = &value
                    && let Some(name) = match op {
                        crate::ast::UnaryOp::Minus => Some("-@"),
                        crate::ast::UnaryOp::Plus => Some("+@"),
                        crate::ast::UnaryOp::Not => Some("!"),
                    }
                    && (self
                        .lookup_method(&value, name)
                        .is_some_and(|(_, method)| !method.is_undefined)
                        || crate::vm::native_methods::rational_parts(&value).is_some()
                        || crate::vm::native_methods::complex_parts(&value).is_some())
                {
                    return self.send_to_object(value, name, vec![], *position);
                }
                self.evaluate_unary_operation(op, value, *position)
            }
            Expression::BinaryOp {
                op,
                left,
                right,
                position,
            } => {
                // Short-circuit evaluation for logical operators and assignment
                match op {
                    BinaryOp::And => {
                        let left_value = self.evaluate_expression(left)?;
                        return if !is_truthy(&left_value) {
                            Ok(left_value)
                        } else {
                            self.evaluate_expression(right)
                        };
                    }
                    BinaryOp::Or => {
                        let left_value = self.evaluate_expression(left)?;
                        return if is_truthy(&left_value) {
                            Ok(left_value)
                        } else {
                            self.evaluate_expression(right)
                        };
                    }
                    BinaryOp::Assign => return self.evaluate_assignment(left, right),
                    _ => {}
                }
                let left_value = self.evaluate_expression(left)?;
                let right_value = self.evaluate_expression(right)?;
                self.operate_on_values(op, left_value, right_value, position)
            }

            // ── Collections ─────────────────────────────────────────────────
            Expression::Array { elements, position } => {
                let made = self.evaluate_array_literal(elements)?;
                self.record_allocation(&made, *position);
                Ok(made)
            }
            Expression::Dictionary { entries, position } => {
                let made = self.evaluate_dictionary_literal(entries)?;
                self.record_allocation(&made, *position);
                Ok(made)
            }
            Expression::Index {
                array,
                index,
                position,
            } => {
                let collection = self.evaluate_expression(array)?;
                // `Held[*values]` spreads the values across the subscript,
                // the same way `Held.[](*values)` does.
                if let Expression::Splat { .. } = index.as_ref() {
                    let spread = self.evaluate_arguments(std::slice::from_ref(index.as_ref()))?;
                    return self.send_to_object(collection, "[]", spread, *position);
                }
                let key = self.evaluate_expression(index)?;
                // Block/Lambda [] call syntax: proc[args]
                if let Object::Block(block) = &collection {
                    return block.call(self, vec![key], *position);
                }
                // `native_fn [args]` — treat as a call with the bracketed array as a single argument.
                // This matches Ruby's `private [:foo, :bar]` which passes an Array to `private`.
                if let Object::NativeFunction(name) = &collection {
                    return self.call_native_function(&name.clone(), vec![key], *position);
                }
                // Check for user-defined [] method on instances
                if let Object::Instance(instance_rc) = &collection {
                    let class = Rc::clone(&instance_rc.borrow().class);
                    if let Some(method) = class.find_method("[]") {
                        return self.invoke_method(
                            class,
                            method,
                            collection.clone(),
                            vec![key],
                            *position,
                        );
                    }
                    // A Thread keeps its locals under `t[:k]`, which the
                    // Thread method table reads. Going through it here keeps
                    // one reading of the key for the subscript and the call.
                    if class.name() == "Thread" {
                        return self
                            .call_thread_method(
                                &collection,
                                "[]",
                                std::slice::from_ref(&key),
                                *position,
                            )?
                            .ok_or_else(|| {
                                MetorexError::runtime_error(
                                    "Thread lookup answered nothing",
                                    position_to_location(*position),
                                )
                            });
                    }
                }
                // A number reads its own bits by subscript, where a pair of
                // arguments arrives as the Array the subscript gathered.
                if matches!(collection, Object::Int(_) | Object::BigInt(_)) {
                    let spread = match &key {
                        Object::Array(gathered) => gathered.borrow().clone(),
                        held => vec![held.clone()],
                    };
                    return self.send_to_object(collection, "[]", spread, *position);
                }
                self.evaluate_index_operation(collection, key, *position)
            }

            // ── Calls ───────────────────────────────────────────────────────
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                trailing_block,
                position,
            } => {
                let attached = self.attached_block_flags.len();
                let result = self.evaluate_method_call(
                    receiver,
                    method,
                    arguments,
                    trailing_block.as_ref().map(|b| b.as_ref()),
                    *position,
                );
                self.release_attached_blocks(attached);
                result
            }
            Expression::Call {
                callee,
                arguments,
                trailing_block,
                position,
            } => {
                let attached = self.attached_block_flags.len();
                let result = self.eval_call(
                    callee,
                    arguments,
                    trailing_block.as_ref().map(|b| b.as_ref()),
                    *position,
                );
                self.release_attached_blocks(attached);
                result
            }
            Expression::Super {
                arguments,
                forward_args,
                trailing_block,
                position,
            } => {
                let attached = self.attached_block_flags.len();
                let calling_frame = self.current_method_frame;
                let result = self.eval_super(
                    arguments,
                    *forward_args,
                    trailing_block.as_ref().map(|block| block.as_ref()),
                    *position,
                );
                self.release_attached_blocks(attached);
                // `break` in the block written on the `super` returns from
                // the `super` call.
                match result {
                    Err(MetorexError::BlockBreak {
                        value, home_frame, ..
                    }) if trailing_block.is_some() && home_frame == calling_frame => Ok(value),
                    other => other,
                }
            }
            Expression::Yield {
                arguments,
                position,
            } => self.eval_yield(arguments, *position),

            // ── Introspection / metaprogramming ─────────────────────────────
            Expression::Defined { expression, .. } => self.eval_defined(expression),

            // ── Other producers ─────────────────────────────────────────────
            Expression::Splat {
                expression,
                position,
            } => {
                // Outside of argument lists, a splat makes a new Array of
                // what it spreads, the way `[*held]` does. Splatting nil
                // names nothing at all.
                let spread = self.evaluate_expression(expression)?;
                Ok(Object::array(self.splat_elements(spread, *position)?))
            }
            // An integer literal past the i64 range, parsed exactly.
            Expression::BigIntLiteral { digits, position } => {
                match num_bigint::BigInt::parse_bytes(digits.as_bytes(), 10) {
                    Some(value) => Ok(Object::integer(value)),
                    None => Err(MetorexError::runtime_error(
                        format!("invalid integer literal '{}'", digits),
                        position_to_location(*position),
                    )),
                }
            }
            // `::Name` reads the top level directly, skipping the lexical
            // chain and any class-local constant of the same name.
            Expression::TopLevelConstant { name, position } => {
                // `::Name` writes Object out as the scope, which a private
                // constant refuses.
                if let Some(Object::Class(object_class)) = self.globals().get("Object")
                    && object_class.is_private_constant(name)
                {
                    return self.private_constant_refused(
                        &object_class,
                        &object_class,
                        name,
                        *position,
                    );
                }
                if let Some(value) = self
                    .globals()
                    .get(name)
                    .or_else(|| self.object_constant(name))
                {
                    self.refuse_unshareable_constant(
                        &format!("Object::{name}"),
                        false,
                        &value,
                        *position,
                    )?;
                    return Ok(value);
                }
                let message = format!("uninitialized constant {}", name);
                Err(MetorexError::UncaughtException {
                    exception: self.constant_name_error(&message, name),
                    location: position_to_location(*position),
                    message,
                })
            }
            Expression::KeywordSplat { expression, .. } => {
                // Outside of an argument list, `**expr` is just `expr`.
                self.evaluate_expression(expression)
            }
            Expression::BlockArg { expression, .. } => {
                // Outside of an argument list, `&expr` is just `expr`.
                self.evaluate_expression(expression)
            }
            Expression::BeginRescue {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                ..
            } => self.evaluate_begin_value(
                body,
                rescue_clauses,
                else_clause.as_deref(),
                ensure_block.as_deref(),
            ),
            Expression::Range {
                start,
                end,
                exclusive,
                ..
            } => {
                let start_value = self.evaluate_expression(start)?;
                let end_value = self.evaluate_expression(end)?;
                self.check_range_ends(&start_value, &end_value, expression.position())?;
                // A range written with literal ends is one object, however
                // many times the line it sits on is run, so the mark it
                // carries is kept against where it was written.
                let written_at = matches!(
                    (start.as_ref(), end.as_ref()),
                    (
                        Expression::IntLiteral { .. }
                            | Expression::FloatLiteral { .. }
                            | Expression::StringLiteral { .. }
                            | Expression::NilLiteral { .. },
                        Expression::IntLiteral { .. }
                            | Expression::FloatLiteral { .. }
                            | Expression::StringLiteral { .. }
                            | Expression::NilLiteral { .. }
                    )
                )
                .then(|| {
                    let at = expression.position();
                    (
                        self.current_source_file.clone().unwrap_or_default(),
                        at.line,
                        at.column,
                    )
                });
                // A range written with literal ends is built once, so the
                // Strings at its ends are frozen with it.
                if written_at.is_some() {
                    for held in [&start_value, &end_value] {
                        if let Object::String(text) = held {
                            text.freeze();
                        }
                    }
                }
                let mark = match written_at {
                    Some(place) => self
                        .written_ranges
                        .entry(place)
                        .or_insert_with(|| std::rc::Rc::new(()))
                        .clone(),
                    None => std::rc::Rc::new(()),
                };
                Ok(Object::Range {
                    start: Box::new(start_value),
                    end: Box::new(end_value),
                    exclusive: *exclusive,
                    mark,
                })
            }
            Expression::Case {
                expression,
                cases,
                else_case,
                position,
            } => self.evaluate_case_expression(expression, cases, else_case.as_deref(), *position),
            Expression::ScopeResolution {
                namespace,
                name,
                position,
            } => {
                let ns_value = self.evaluate_expression(namespace)?;
                let value = self.read_scoped_constant(ns_value.clone(), name, position)?;
                if self.in_a_non_main_ractor() {
                    let owner = match &ns_value {
                        Object::Class(held) | Object::Module(held) => held.ruby_name(),
                        other => format!("{other}"),
                    };
                    self.refuse_unshareable_constant(
                        &format!("{owner}::{name}"),
                        false,
                        &value,
                        *position,
                    )?;
                }
                Ok(value)
            }
            Expression::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                ..
            } => self.evaluate_if_expression(condition, then_branch, elsif_branches, else_branch),
            Expression::Unless {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.evaluate_unless_expression(condition, then_branch, else_branch),
            Expression::SingletonClass {
                target,
                assign_to,
                body,
                position,
            } => self.evaluate_singleton_class_expression(
                target,
                assign_to.as_deref(),
                body,
                *position,
            ),
        }
    }

    /// Apply a binary operator to operands already evaluated, which is how a
    /// compound assignment combines what it read with its right-hand side.
    /// Evaluate the block written after a call, marking the call it is
    /// attached to as running until `release_attached_blocks` clears it.
    pub(crate) fn attach_trailing_block(
        &mut self,
        block_expr: &Expression,
    ) -> Result<Object, MetorexError> {
        let block = self.evaluate_expression(block_expr)?;
        if let Object::Block(held) = &block {
            held.attached_call_running.set(true);
            self.attached_block_flags
                .push(Rc::clone(&held.attached_call_running));
        }
        Ok(block)
    }

    /// The signal a `break` in a block raises: a BlockBreak that returns
    /// from the call the block was attached to, or a LocalJumpError when
    /// that call has already returned.
    pub(crate) fn break_signal(
        &self,
        value: Object,
        location: crate::error::SourceLocation,
    ) -> MetorexError {
        if let Some(Some(running)) = self.running_block_breaks.last()
            && !running.get()
        {
            let message = "break from proc-closure".to_string();
            return MetorexError::UncaughtException {
                exception: Object::exception("LocalJumpError", message.clone()),
                location,
                message,
            };
        }
        MetorexError::BlockBreak {
            value,
            location,
            home_frame: None,
        }
    }

    /// Mark every block attached since the stack stood at `depth` as no
    /// longer running, since the call it was attached to has returned.
    pub(crate) fn release_attached_blocks(&mut self, depth: usize) {
        while self.attached_block_flags.len() > depth {
            if let Some(flag) = self.attached_block_flags.pop() {
                flag.set(false);
            }
        }
    }

    pub(crate) fn operate_on_values(
        &mut self,
        op: &BinaryOp,
        left_value: Object,
        right_value: Object,
        position: &crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        // A reopened core class, or a module prepended to one,
        // holds the operator in a method table rather than in the
        // native one, so `1 + 2` consults it before the built-in
        // arithmetic runs. Only the receiver's own class and its
        // prepended modules are asked: an ancestor's definition, such
        // as the core library's Numeric#%, describes the operator the
        // native path already implements, and answering from there
        // would replace every built-in operator with its Ruby-level
        // spelling.
        if !matches!(left_value, Object::Instance(_))
            && let Some(op_name) = binary_op_method_name(op)
            && let Some((class, method)) = self.lookup_own_operator_method(&left_value, op_name)
        {
            return self.invoke_method(
                class,
                method,
                left_value.clone(),
                vec![right_value],
                *position,
            );
        }
        // Check for user-defined operator methods on instances. Walk
        // via lookup_method so per-instance singleton-class overrides
        // (used by mspec mocks, among other things) win over the
        // underlying class definition.
        if let (Some(op_name), Object::Instance(_)) = (binary_op_method_name(op), &left_value) {
            if let Some((class, method)) = self.lookup_method(&left_value, op_name)
                && !method.is_undefined
            {
                return self.invoke_method(
                    class,
                    method,
                    left_value.clone(),
                    vec![right_value],
                    *position,
                );
            }
            // A generated struct class answers `==` from Struct's
            // native table, which the method map above does not hold.
            let instance_class = match &left_value {
                Object::Instance(instance) => Some(std::rc::Rc::clone(&instance.borrow().class)),
                _ => None,
            };
            if let Some(class) = &instance_class
                && let Some(members) = crate::vm::native_methods::struct_members(class)
                && let Some(result) = self.call_struct_instance_method(
                    class,
                    &members,
                    &left_value,
                    op_name,
                    std::slice::from_ref(&right_value),
                    *position,
                )?
            {
                return Ok(result);
            }
            // Rational arithmetic likewise lives in a native table.
            if let Some(class) = &instance_class
                && class.name() == "Rational"
                && let Some(result) = self.call_rational_method(
                    &left_value,
                    op_name,
                    std::slice::from_ref(&right_value),
                    *position,
                )?
            {
                return Ok(result);
            }
            // Comparable derives `<`, `<=`, `>` and `>=` from `<=>` for a
            // class that includes it; metorex answers them here rather than
            // through methods on the module.
            if matches!(
                op,
                BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual
            ) && (!matches!(left_value, Object::Instance(_)) || self.is_comparable(&left_value))
                && let Some((class, spaceship)) = self.lookup_method(&left_value, "<=>")
            {
                let cmp = self.invoke_method(
                    class,
                    spaceship,
                    left_value.clone(),
                    vec![right_value.clone()],
                    *position,
                )?;
                if let Object::Int(c) = cmp {
                    let result = match op {
                        BinaryOp::Less => c < 0,
                        BinaryOp::LessEqual => c <= 0,
                        BinaryOp::Greater => c > 0,
                        BinaryOp::GreaterEqual => c >= 0,
                        _ => unreachable!(),
                    };
                    return Ok(Object::Bool(result));
                }
            }
        }
        // `1 / 2r` — an Integer or Float on the left of a Rational is
        // promoted so the Rational's own arithmetic runs.
        if let (Some(op_name), Object::Int(_) | Object::Float(_)) =
            (binary_op_method_name(op), &left_value)
            && crate::vm::native_methods::rational_parts(&right_value).is_some()
        {
            let promoted = self.promote_to_rational(&left_value, *position)?;
            if let Some(result) = self.call_rational_method(
                &promoted,
                op_name,
                std::slice::from_ref(&right_value),
                *position,
            )? {
                return Ok(result);
            }
        }
        self.evaluate_binary_operation(op, left_value, right_value, *position)
    }

    /// Read `name` out of a namespace already evaluated, which is how
    /// `Ns::Name` resolves and how `Ns::Name += 1` reads before it writes.
    pub(crate) fn read_scoped_constant(
        &mut self,
        ns_value: Object,
        name: &str,
        position: &crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        match ns_value {
            Object::Class(class_rc) | Object::Module(class_rc) => {
                // Own constants first, then (like Ruby's qualified
                // lookup) the ancestor chain — but not top-level
                // constants, which a qualified reference must not
                // reach. Registered autoloads fire on their owner.
                // Top-level constants are Object's own, so a name
                // written as `Object::Name` reaches one of them before
                // any module mixed into Object.
                let top_level = if class_rc.name() == "Object"
                    && name.starts_with(|held: char| held.is_ascii_uppercase())
                {
                    self.globals()
                        .get(name)
                        .filter(|held| !matches!(held, Object::NativeFunction(_)))
                } else {
                    None
                };
                let entry = match top_level {
                    Some(held) => Some((Rc::clone(&class_rc), Some(held))),
                    None => self.const_entry_on(&class_rc, name, true, false),
                };
                let owner = match &entry {
                    Some((owner, _)) => Rc::clone(owner),
                    None => Rc::clone(&class_rc),
                };
                let value = match entry {
                    Some((_, Some(v))) => Some(v),
                    Some((owner, None)) => self.try_autoload_constant(&owner, name)?,
                    None => self.try_autoload_constant(&class_rc, name)?,
                };
                if let Some(v) = value {
                    // A private constant is read with its scope written out
                    // only from inside the body of the class that owns it.
                    if owner.is_private_constant(name)
                        && !self
                            .def_scope_stack
                            .iter()
                            .any(|open| Rc::ptr_eq(open, &owner))
                    {
                        return self.private_constant_refused(&class_rc, &owner, name, *position);
                    }
                    self.warn_deprecated_constant(&class_rc, name, *position);
                    return Ok(v);
                }
                // Uninitialized constants dispatch const_missing —
                // the default implementation raises NameError.
                // Autoload's "loaded but didn't define" path lands
                // here too.
                self.dispatch_const_missing(&class_rc, name, *position)
            }
            // Only a class or module holds constants, so reading one
            // out of anything else is refused by its type.
            held => Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("{} is not a class/module", held.type_name()),
                *position,
            )),
        }
    }
}

/// Map a binary operator to its operator method name for user-defined dispatch.
pub(crate) fn binary_op_method_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+"),
        BinaryOp::Subtract => Some("-"),
        BinaryOp::Multiply => Some("*"),
        BinaryOp::Divide => Some("/"),
        BinaryOp::Modulo => Some("%"),
        BinaryOp::Power => Some("**"),
        BinaryOp::Equal => Some("=="),
        BinaryOp::CaseEqual => Some("==="),
        BinaryOp::NotEqual => Some("!="),
        BinaryOp::Less => Some("<"),
        BinaryOp::Greater => Some(">"),
        BinaryOp::LessEqual => Some("<="),
        BinaryOp::GreaterEqual => Some(">="),
        BinaryOp::Spaceship => Some("<=>"),
        BinaryOp::BitwiseAnd => Some("&"),
        BinaryOp::BitwiseOr => Some("|"),
        BinaryOp::Xor => Some("^"),
        _ => None,
    }
}

/// Whether `$<digits>` names a capture past the highest one a pattern can
/// number, which Ruby reads as nil and says so about.
fn number_variable_is_too_big(name: &str) -> bool {
    /// The highest group a pattern can number, above which Ruby reads the
    /// variable as nil.
    const HIGHEST_GROUP: u128 = (1 << 30) - 1;
    if name.is_empty() || !name.bytes().all(|held| held.is_ascii_digit()) {
        return false;
    }
    name.parse::<u128>().is_ok_and(|held| held > HIGHEST_GROUP)
}

impl VirtualMachine {
    /// The value the global `$name` holds, read the way `$name` written in
    /// the program reads it.
    pub(crate) fn read_global_variable(
        &mut self,
        name: &str,
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        self.refuse_global_in_ractor(name, position)?;
        // `$?` is the status of the last child waited for, which lives
        // with the process rather than in the global table.
        // A global given a second name reads what the first holds,
        // whatever that first name is: `$MATCH` reads the match `$&`
        // names the same way `$&` does.
        let name = self
            .global_aliases
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string());
        // A global a C extension defined is read through its getter.
        if let Some(hooked) = self.hooked_globals.get(&name).copied() {
            return crate::vm::capi::read_hooked_global(self, &name, hooked, position);
        }
        if name == "?" {
            return Ok(self.process_last_status());
        }
        // `$=` once made matches ignore case, and reading it says it
        // no longer does once the deprecated category is asked for.
        if name == "=" && self.warning_category_enabled("deprecated") {
            let message = format!(
                "{}variable $= is no longer effective\n",
                self.warning_prefix(0, position)
            );
            self.warn_through_warning_module(message, position)?;
        }
        // A number past what a capture can be numbered names no
        // group at all, which Ruby says so about and reads as nil.
        if number_variable_is_too_big(&name) {
            let at = (
                self.current_source_file.clone().unwrap_or_default(),
                position.line,
                position.column,
            );
            if self.reported_big_number_variables.insert(at) {
                let message =
                    format!("warning: '${name}' is too big for a number variable, always nil");
                self.emit_warning_to_stderr(&message, position);
            }
            return Ok(Object::Nil);
        }
        // `$1` through `$9` name the captures of the last match, and
        // `` $` `` and `$'` the text on either side of it.
        if let Some(group) = crate::vm::native_methods::capture_reference(&name) {
            return self.last_match_part(group, position);
        }
        // `$@` is where the exception being handled was raised, which
        // is the backtrace that exception carries.
        if name == "@" {
            let raised = self.globals().get("!").unwrap_or(Object::Nil);
            if matches!(raised, Object::Nil) {
                return Ok(Object::Nil);
            }
            return self.send_to_object(raised, "backtrace", Vec::new(), position);
        }
        match self.globals().get(&name) {
            Some(held) => Ok(held),
            None => {
                // Reading a global nothing has set says so in
                // verbose mode.
                if matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
                    let message = format!(
                        "{}global variable '${name}' not initialized\n",
                        self.warning_prefix(0, position)
                    );
                    self.warn_through_warning_module(message, position)?;
                }
                Ok(Object::Nil)
            }
        }
    }
}
