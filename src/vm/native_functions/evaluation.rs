// Reading Ruby source and running it.

use super::*;

impl VirtualMachine {
    pub(crate) fn parse_source(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() != 1 {
            return Err(MetorexError::runtime_error(
                format!("parse() expects 1 argument, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let code = match &arguments[0] {
            Object::String(s) => s.as_str().to_string(),
            other => {
                return Err(MetorexError::runtime_error(
                    format!(
                        "parse() expects a String argument, got {}",
                        other.type_name()
                    ),
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };
        let tokens = crate::lexer::Lexer::new(&code).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .map_err(|errors| {
                MetorexError::runtime_error(
                    format!(
                        "parse: parse error: {}",
                        errors
                            .iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<_>>()
                            .join("; ")
                    ),
                    crate::vm::utils::position_to_location(position),
                )
            })?;
        use crate::vm::native_methods::ast_methods;
        Ok(ast_methods::serialize_statements(&statements))
    }

    pub(crate) fn eval_source(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
        invoked_with: Option<Object>,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() || arguments.len() > 4 {
            return Err(MetorexError::runtime_error(
                format!("eval() expects 1-4 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        // A value of the program's own stands in for source when it
        // says how to read itself as a String.
        let mut arguments = arguments;
        if !matches!(arguments[0], Object::String(_)) && self.responds_to(&arguments[0], "to_str") {
            let spelled = self.send_to_object(arguments[0].clone(), "to_str", vec![], position)?;
            arguments[0] = spelled;
        }
        // Only a Binding says where code runs. A Proc names a scope
        // of its own but is not one.
        if let Some(held) = arguments.get(1)
            && !matches!(held, Object::Binding(_) | Object::Nil)
        {
            // Ruby names a callable by what it is rather than by its
            // class, so a Proc reads as `proc` here.
            let named = match held {
                Object::Block(_) => "proc".to_string(),
                other => self.builtins().class_of(other).name().to_string(),
            };
            let message = format!("wrong argument type {named} (expected binding)");
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("TypeError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        let (code, code_encoding) = match &arguments[0] {
            // Source written in an encoding of its own is read back
            // through that encoding before it is lexed. A magic
            // comment names the encoding the source was written in,
            // whatever the string carrying it is tagged with, so
            // binary source that says UTF-8 is read as UTF-8.
            Object::String(s) => {
                let tagged = s.encoding_name();
                let carried = crate::vm::native_methods::name_text(s);
                match crate::lexer::named_source_encoding(&carried).map(|named| {
                    crate::vm::native_methods::string_methods::canonical_encoding_name(&named)
                }) {
                    Some(named) if named != tagged => (
                        crate::vm::native_methods::text_in_encoding(s, &named),
                        named,
                    ),
                    _ => (carried, tagged),
                }
            }
            other => {
                return Err(MetorexError::runtime_error(
                    format!(
                        "eval() expects a String argument, got {}",
                        other.type_name()
                    ),
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };
        // Code is written in the encoding a magic comment of its own
        // names, and otherwise in the one the string carrying it is
        // tagged with, which is what `__ENCODING__` answers inside.
        let previous_source_encoding = self.current_source_encoding.replace(
            crate::lexer::named_source_encoding(&code).unwrap_or_else(|| code_encoding.clone()),
        );
        // Optional filename (arg 3) and lineno (arg 4) shape the
        // positions recorded for code inside the eval'd string
        // (`__LINE__`, const_source_location, backtraces).
        let filename = match arguments.get(2) {
            Some(Object::String(s)) => Some(s.as_str().to_string()),
            _ => None,
        };
        // The line the code is counted from, which a program may
        // name as anything, negative included: it shapes the numbers
        // a backtrace and a syntax error report.
        let named_lineno = match arguments.get(3) {
            Some(Object::Int(n)) => *n,
            _ => 1,
        };
        let lineno = named_lineno.max(1) as usize;
        let tokens = crate::lexer::Lexer::with_start_line(&code, lineno)
            .with_source_encoding(Some(
                crate::lexer::named_source_encoding(&code).unwrap_or_else(|| code_encoding.clone()),
            ))
            .tokenize();
        // The code is read with the locals of the scope it runs in.
        let outer_locals = match arguments.get(1) {
            Some(Object::Binding(held)) => held
                .keys()
                .into_iter()
                .filter(|name| name != "self")
                .collect(),
            _ => self.visible_local_names(),
        };
        let mut parser = crate::parser::Parser::new(tokens)
            .inside_eval()
            .with_outer_locals(outer_locals);
        let parsed = parser.parse();
        let source_warnings = parser.warnings().to_vec();
        let default_warnings = parser.default_warnings().to_vec();
        // Code the interpreter reads but MRI's parser refuses is refused as
        // MRI refuses it.
        let parsed = match parsed {
            Ok(statements)
                if crate::vm::native_functions::prism_refuses(
                    &code,
                    &crate::vm::native_functions::PrismReading {
                        start_line: named_lineno as i32,
                        command_line: "",
                        main_script: false,
                        partial_script: true,
                        scopes: Some(&parser.inherited_local_names()),
                    },
                ) =>
            {
                let named = filename.clone().unwrap_or_else(|| {
                    format!(
                        "(eval at {}:{})",
                        self.file_for_frames().unwrap_or_default(),
                        position.line
                    )
                });
                match self.prism_syntax_report(&code, &named, named_lineno, &parser, position)? {
                    Some(error) => return Err(error),
                    None => Ok(statements),
                }
            }
            other => other,
        };
        let statements = match parsed {
            Ok(statements) => statements,
            Err(errors) => {
                // MRI words what it refuses as prism does, under the name
                // the code runs as, and only code prism accepts as well
                // keeps the interpreter's own wording.
                let named = filename.clone().unwrap_or_else(|| {
                    format!(
                        "(eval at {}:{})",
                        self.file_for_frames().unwrap_or_default(),
                        position.line
                    )
                });
                if let Some(error) =
                    self.prism_syntax_report(&code, &named, named_lineno, &parser, position)?
                {
                    return Err(error);
                }
                return Err({
                    // Ruby names the file in front of the message, so
                    // code eval'd on behalf of a template points at the
                    // template rather than at the eval.
                    let reported = errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; ");
                    let at = errors
                        .first()
                        .and_then(|held| held.location())
                        .map_or(0, |held| held.line.max(1).saturating_sub(lineno))
                        as i64
                        + named_lineno;
                    let message = match &filename {
                        Some(named) => format!("{named}:{at}: {reported}"),
                        None => format!("eval: parse error: {reported}"),
                    };
                    crate::vm::errors::syntax_error(message, filename.as_deref(), position)
                });
            }
        };
        // What reading the code found to warn about whatever the
        // verbosity, under the name the code runs as.
        if !default_warnings.is_empty() {
            let named = filename.clone().unwrap_or_else(|| {
                format!(
                    "(eval at {}:{})",
                    self.file_for_frames().unwrap_or_default(),
                    position.line
                )
            });
            let counted: Vec<_> = default_warnings
                .iter()
                .map(|(at, warning)| {
                    let mut shifted = *at;
                    shifted.line =
                        ((at.line as i64) - (lineno as i64) + named_lineno).max(0) as usize;
                    (shifted, warning.clone())
                })
                .collect();
            self.report_parse_warnings(&named, &counted);
        }
        // What reading the code found to warn about, which a verbose run
        // reports under the name the code runs as.
        if matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
            let named = filename.clone().unwrap_or_else(|| "(eval)".to_string());
            for (at, warning) in &source_warnings {
                let line = (at.line as i64) - (lineno as i64) + named_lineno;
                self.emit_warning_to_stderr(
                    &format!("{named}:{line}: warning: {warning}"),
                    position,
                );
            }
        }
        // A `frozen_string_literal` comment written after code names
        // nothing, and a verbose run says so.
        if crate::lexer::frozen_string_literal_after_a_token(&code)
            && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true)))
        {
            self.emit_warning_to_stderr(
                "warning: `frozen_string_literal' is ignored after any tokens",
                position,
            );
        }
        // Ruby reports every string it compiles, which is what a
        // trace reads the source back out of.
        self.fire_event(
            "script_compiled",
            position,
            vec![("eval_script", Object::string(code.clone()))],
        )?;
        // A Binding argument re-establishes the frame it captured:
        // its locals (shared cells, so assignment through the eval is
        // visible to a later one) and the `self` in force there.
        let binding = match arguments.get(1) {
            Some(Object::Binding(b)) => Some(std::rc::Rc::clone(b)),
            _ => None,
        };
        // Code run through a binding sees the class the binding was
        // taken in, which is what a class variable written there
        // belongs to.
        let carried_cref = binding.as_ref().and_then(|held| match &held.receiver {
            Some(Object::Class(class) | Object::Module(class)) => Some(Rc::clone(class)),
            Some(Object::Instance(instance)) => Some(Rc::clone(&instance.borrow().class)),
            _ => None,
        });
        let carried_cref_pushed = carried_cref.is_some();
        // A class opened in the eval'd code is nested where the
        // binding was taken, so `class Inside; end` run through a
        // binding taken in a class belongs to that class.
        // Only a binding taken in a class or module body nests what
        // the code opens. One taken in an instance method leaves the
        // eval at the top level, where `main` sits.
        // Code eval'd inside a method body opens what it defines
        // where that method was written, so `eval "class C; end"` in
        // a method of A makes A::C rather than a top-level C.
        let captured_nesting: Option<Vec<Rc<crate::class::Class>>> = match &binding {
            Some(held) if !held.nesting.borrow().is_empty() => {
                Some(held.nesting.borrow().iter().rev().map(Rc::clone).collect())
            }
            // One taken outside every class opens nothing, wherever the
            // code is run from.
            Some(held)
                if held.home_frame.borrow().is_some_and(|home| {
                    home.is_none_or(|frame| frame == crate::vm::core::TOP_LEVEL_FRAME)
                }) =>
            {
                Some(Vec::new())
            }
            Some(_) => None,
            None if !self.def_scope_stack.is_empty() => None,
            None => self
                .method_nesting_stack
                .last()
                .filter(|captured| !captured.is_empty())
                .map(|captured| captured.iter().rev().map(Rc::clone).collect()),
        };
        let saved_def_scope =
            captured_nesting.map(|opened| std::mem::replace(&mut self.def_scope_stack, opened));
        if let Some(cref) = carried_cref {
            self.class_var_cref_stack.push(Some(cref));
        }
        if let Some(b) = &binding {
            self.environment_mut().push_isolated_scope();
            // The binding's own order is what `local_variables` inside
            // the eval reports after the names the eval binds itself.
            for name in b.keys() {
                if let Some(cell) = b.get(&name) {
                    self.environment_mut().define_inherited(name, cell);
                }
            }
            if let Some(receiver) = &b.receiver {
                self.environment_mut()
                    .define("self".to_string(), receiver.clone());
            }
        } else {
            // Code eval'd without a binding runs in a scope of its
            // own that sees the caller's locals. A local the code
            // binds belongs to that scope and is gone once it has
            // run, which is what Ruby does.
            let carried: Vec<(String, std::rc::Rc<std::cell::RefCell<Object>>)> = self
                .environment()
                .binding_variable_names()
                .into_iter()
                .filter_map(|name| self.environment().get_ref(&name).map(|cell| (name, cell)))
                .collect();
            // Code runs against whoever the eval was written with,
            // which for `Kernel.eval` is Kernel itself and otherwise
            // is the self in force where the call was made.
            let here = invoked_with
                .clone()
                .or_else(|| self.environment().get("self"));
            self.environment_mut().push_isolated_scope();
            for (name, cell) in carried {
                self.environment_mut().define_inherited(name, cell);
            }
            if let Some(receiver) = here {
                self.environment_mut().define("self".to_string(), receiver);
            }
        }
        // A name the code assigns to is a local of the scope the eval
        // opens, whether or not the line assigning it runs, which is
        // what leaves `a` behind as nil for a later look.
        let mut hoisted: Vec<String> = Vec::new();
        for name in crate::ast::scope_locals::collect_assigned_locals(&statements) {
            if self.environment().assignment_introduces_a_local(&name) {
                self.environment_mut().hoist(name.clone());
                hoisted.push(name);
            }
        }
        // eval runs at top-level of its string: treat as non-method scope.
        // Refinements activated inside eval are lexical to the eval string.
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        // Code run through a binding sees the refinements in force
        // where the binding was taken; anything else opens a scope of
        // its own, since a refinement used inside an eval is lexical
        // to the string.
        let carried_refinements: Vec<crate::vm::core::RefinementEntry> = binding
            .as_ref()
            .map(|held| {
                held.refinements
                    .borrow()
                    .iter()
                    .map(|(module, classes)| crate::vm::core::RefinementEntry {
                        module: Rc::clone(module),
                        classes: classes.iter().cloned().collect(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.refinement_scopes.push(carried_refinements);
        let prev_file = self.current_file.clone();
        match &filename {
            Some(f) => {
                self.eval_named_files.insert(f.clone());
                self.current_file = Some(std::path::PathBuf::from(f));
            }
            // Ruby names the eval'd code after the place it was
            // written, which is what `__FILE__` reports inside it.
            // Code run through a binding is named for the eval that
            // ran it rather than for where the binding was taken.
            None => {
                let written_in = prev_file
                    .as_ref()
                    .map(|file| self.reported_spelling(file).display().to_string())
                    .unwrap_or_default();
                self.current_file = Some(std::path::PathBuf::from(format!(
                    "{}{}:{})",
                    crate::vm::EVAL_FILE_PREFIX,
                    written_in,
                    position.line
                )));
            }
        }
        // The code was written where the eval stands, so it names
        // that place rather than the file the caller's method came
        // from.
        let prev_source_file = std::mem::replace(
            &mut self.current_source_file,
            self.current_file
                .as_ref()
                .map(|file| file.display().to_string()),
        );
        // The eval'd string runs in the caller's body, so it sees the
        // visibility state in force there. A toggle it sets belongs to
        // the eval and is restored afterwards.
        let enclosing = match self.environment().get("self") {
            Some(Object::Class(class) | Object::Module(class)) => {
                Some((Rc::clone(&class), class.current_visibility()))
            }
            _ => None,
        };
        // Code handed to `eval` is counted into the file it names when
        // the run was started with `eval` coverage on.
        if self.coverage_counts_eval()
            && let Some(named) = self.current_source_file.clone()
        {
            self.coverage_note_eval(&named, &statements);
        }
        // The eval is a place of its own in a backtrace, standing at
        // the line it was written on in the file that wrote it, so
        // code inside it still reports back to the program.
        // The frame reads as the one that ran the eval, which is the
        // name Ruby gives code inside one and what keeps `__method__`
        // there naming the method around it.
        let taken_in = match &binding {
            Some(held) => Some(
                held.frame
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| crate::vm::CallFrame::boundary("<main>")),
            ),
            None => self.call_stack().last().cloned(),
        };
        let written_in = taken_in.unwrap_or_else(|| crate::vm::CallFrame::boundary("<main>"));
        self.call_stack_push(
            written_in
                .with_location(Some(format!("{}:{}", position.line, position.column)))
                .with_source_file(
                    prev_source_file
                        .clone()
                        .or_else(|| prev_file.as_ref().map(|file| file.display().to_string())),
                ),
        );
        // Code run through a binding runs where the binding was
        // taken, so `__method__` names the method it was taken in.
        // A binding that carries its frame already runs in it.
        let named_method = binding
            .as_ref()
            .filter(|held| held.frame.borrow().is_none())
            .and_then(|held| held.method.borrow().clone());
        if let Some((callee, defined)) = &named_method {
            self.call_stack_push(crate::vm::CallFrame::method(
                callee.clone(),
                None,
                callee.clone(),
                defined.clone(),
            ));
        }
        // A block written on the eval belongs to the eval, so a
        // method the code calls is not handed it and a `yield` there
        // has nothing to run.
        let held_block = self.pending_block.take();
        let carried_home = binding.as_ref().and_then(|held| *held.home_frame.borrow());
        let saved_lexical_home = match carried_home {
            Some(home) => Some(self.lexical_home_frame.replace(home)),
            None => None,
        };
        // A `return` written in the code returns from the scope the
        // eval was written in rather than ending the eval, so it
        // carries on out rather than being answered here.
        let result = self.run_eval_statements(&statements);
        // An error the code raised natively is traced here, while the file
        // it was written in is still the one the eval named.
        if let Err(error) = &result {
            self.trace_error_leaving_frame(error);
        }
        if let Some(held) = saved_lexical_home {
            self.lexical_home_frame = held;
        }
        self.pending_block = held_block;
        if named_method.is_some() {
            self.call_stack_pop();
        }
        self.call_stack_pop();
        if carried_cref_pushed {
            self.class_var_cref_stack.pop();
        }
        if let Some(held) = saved_def_scope {
            self.def_scope_stack = held;
        }
        if let Some(held) = &binding {
            // A local the code named that the binding did not have is
            // added to it, which is how `eval("x = 1", b)` leaves `x`
            // behind for the next look through `b`.
            for (name, cell) in self.environment().current_scope_var_refs() {
                if !held.has(&name) {
                    held.set(&name, cell);
                }
            }
            // A name the code only assigns to on a line that never
            // ran is a local of the scope all the same, and the next
            // look through the binding reads it as nil.
            for name in &hoisted {
                if !held.has(name)
                    && let Some(cell) = self.environment().get_ref(name)
                {
                    held.set(name, cell);
                }
            }
        }
        self.environment_mut().pop_scope();
        if let Some((class, visibility)) = enclosing {
            class.set_current_visibility(visibility);
        }
        self.current_file = prev_file;
        self.current_source_file = prev_source_file;
        self.current_source_encoding = previous_source_encoding;
        self.pop_refinement_scope();
        self.user_def_nesting = saved_nesting;
        Ok(result?.unwrap_or(Object::Nil))
    }

    pub(crate) fn load_file(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(MetorexError::runtime_error(
                format!("load() expects 1-2 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        // `load(path, true)` runs the file inside a fresh anonymous
        // module, and `load(path, SomeModule)` inside that one.
        let wrapper: Option<Rc<crate::class::Class>> = match arguments.get(1) {
            Some(Object::Bool(true)) => Some(crate::class::Class::new_module("")),
            Some(Object::Module(module) | Object::Class(module)) => Some(Rc::clone(module)),
            _ => None,
        };
        let wrap = wrapper.is_some();
        self.load_wrap_module = wrapper;
        let path_str = self.coerce_load_path(&arguments[0], position)?;
        // A leading `~` names the home directory, as the shell has it.
        let path_str = self.expand_home_path(&path_str);
        let path = std::path::PathBuf::from(&path_str);
        let path = path.as_path();
        if wrap {
            self.load_wrap_depth += 1;
        }
        // Each load() has its own refinement scope and a fresh
        // user-method-nesting counter for its top-level statements.
        self.push_refinement_scope();
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        // load always executes the file (no deduplication)
        // Try the path directly first, then search $LOAD_PATH
        // A path written relative to the working directory names that
        // file outright and is never searched for in `$LOAD_PATH`.
        let anchored =
            path_str.starts_with("./") || path_str.starts_with("../") || path_str.starts_with('/');
        let result = if path.is_file() {
            self.execute_file_recording(path, false).map_err(|error| {
                // A file that cannot be read is one that cannot be
                // loaded, which Ruby reports as a LoadError.
                if error.message().contains("Failed to read file") {
                    let message = format!("cannot load such file -- {}", path_str);
                    return MetorexError::UncaughtException {
                        exception: Object::exception("LoadError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    };
                }
                crate::vm::errors::keep_exception(error, |message| {
                    MetorexError::runtime_error(
                        format!("load('{}') — {}", path_str, message),
                        crate::vm::utils::position_to_location(position),
                    )
                })
            })
        } else if anchored {
            let message = format!("cannot load such file -- {}", path_str);
            Err(MetorexError::UncaughtException {
                exception: Object::exception("LoadError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            })
        } else {
            // Search $LOAD_PATH
            let search_dirs = self.load_path_directories();
            let mut found = None;
            for dir in &search_dirs {
                let candidate = std::path::PathBuf::from(dir).join(&path_str);
                if candidate.exists() {
                    found = Some(candidate);
                    break;
                }
            }
            match found {
                Some(resolved) => self.execute_file_recording(&resolved, false).map_err(|e| {
                    crate::vm::errors::keep_exception(e, |message| {
                        MetorexError::runtime_error(
                            format!("load('{}') — {}", path_str, message),
                            crate::vm::utils::position_to_location(position),
                        )
                    })
                }),
                None => {
                    let message = format!("cannot load such file -- {}", path_str);
                    Err(MetorexError::UncaughtException {
                        exception: Object::exception("LoadError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    })
                }
            }
        };
        if wrap {
            self.load_wrap_depth -= 1;
        }
        self.pop_refinement_scope();
        self.user_def_nesting = saved_nesting;
        result?;
        Ok(Object::Bool(true))
    }
}

impl VirtualMachine {
    /// The error MRI raises for code `eval` refuses, worded and laid out by
    /// the prelude over prism, or None when prism accepts the code.
    fn prism_syntax_report(
        &mut self,
        code: &str,
        named: &str,
        start_line: i64,
        parser: &crate::parser::Parser,
        position: Position,
    ) -> Result<Option<MetorexError>, MetorexError> {
        let locals = parser
            .inherited_local_names()
            .into_iter()
            .map(Object::string)
            .collect();
        let main = self.globals().get("__main__").unwrap_or(Object::Nil);
        let report = self.send_to_object(
            main,
            "__syntax_error_report__",
            vec![
                Object::string(code.to_string()),
                Object::string(named.to_string()),
                Object::Int(start_line),
                Object::array(locals),
            ],
            position,
        )?;
        let Object::Array(pair) = report else {
            return Ok(None);
        };
        let pair = pair.borrow().clone();
        let [Object::String(class), Object::String(message)] = pair.as_slice() else {
            return Ok(None);
        };
        let message = message.as_str().to_string();
        if &*class.as_str() == "SyntaxError" {
            return Ok(Some(crate::vm::errors::syntax_error(
                message,
                Some(named),
                position,
            )));
        }
        Ok(Some(crate::vm::errors::simple_exception(
            &class.as_str(),
            &message,
            position,
        )))
    }
}
