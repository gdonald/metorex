// Kernel#warn.
//
// The message is assembled here, then handed to `Warning.warn`, which the
// prelude defines in Ruby so a program can replace it. Whether the category
// travels along as a keyword depends on the arity of the `Warning.warn` in
// force, matching MRI.

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

/// The keyword names `warn` accepts. A trailing Hash counts as keywords only
/// when every key is one of these, so `warn({a: 1})` still prints the Hash.
const WARN_KEYWORDS: &[&str] = &["uplevel", "category"];

impl VirtualMachine {
    /// `Kernel#warn(*messages, uplevel: nil, category: nil)`.
    pub(crate) fn kernel_warn(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.kernel_warn_for(None, arguments, position)
    }

    /// The same, told which object the call was made on. Kernel#warn on the
    /// Warning module writes the message out rather than handing it back to
    /// `Warning.warn`, which is what lets a `Warning.warn` of a program's own
    /// call `super` without reaching itself.
    pub(crate) fn kernel_warn_for(
        &mut self,
        receiver: Option<&Object>,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let on_warning_itself = match (receiver, self.globals().get("Warning")) {
            (Some(Object::Module(given)), Some(Object::Module(named))) => Rc::ptr_eq(given, &named),
            _ => false,
        };
        let (messages, keywords) = split_warn_keywords(arguments);

        let category = match keyword_value(&keywords, "category") {
            Some(Object::Nil) | None => Object::Nil,
            Some(value) => self.coerce_warn_category(&value, position)?,
        };
        let prefix = match keyword_value(&keywords, "uplevel") {
            Some(Object::Nil) | None => String::new(),
            Some(value) => {
                let level = self.coerce_uplevel(&value, position)?;
                self.warning_prefix(level, position)
            }
        };

        // `$VERBOSE` set to nil silences the warning before anything is built.
        if messages.is_empty() || matches!(self.globals().get("VERBOSE"), None | Some(Object::Nil))
        {
            return Ok(Object::Nil);
        }
        // A warning in a category that is switched off never reaches
        // `Warning.warn` at all.
        if let Object::Symbol(name) = &category
            && !self.warning_category_enabled(&name.as_str())
        {
            return Ok(Object::Nil);
        }

        let mut text = String::new();
        for message in &messages {
            for line in flatten_warn_message(message) {
                let rendered = self.string_for_warning(&line, position)?;
                text.push_str(&rendered);
                if !rendered.ends_with('\n') {
                    text.push('\n');
                }
            }
        }
        let text = format!("{}{}", prefix, text);

        if on_warning_itself {
            self.write_to_stderr(&text, position)?;
            return Ok(Object::Nil);
        }
        self.dispatch_to_warning_module(text, category, position)?;
        Ok(Object::Nil)
    }

    /// `category:` is either nil or something that answers `to_sym`.
    fn coerce_warn_category(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Object::Symbol(name) = value {
            return Ok(Object::Symbol(Rc::clone(name)));
        }
        if let Object::String(text) = value {
            return Ok(Object::symbol(text.as_str().to_string()));
        }
        if self.responds_to(value, "to_sym") {
            let converted = self.send_to_object(value.clone(), "to_sym", vec![], position)?;
            if let Object::Symbol(_) = converted {
                return Ok(converted);
            }
        }
        Err(type_error(
            format!(
                "no implicit conversion of {} into Symbol",
                value.type_name()
            ),
            position,
        ))
    }

    /// `uplevel:` counts stack frames, so it has to be a non-negative Integer
    /// or something that converts to one.
    fn coerce_uplevel(&mut self, value: &Object, position: Position) -> Result<i64, MetorexError> {
        let level = match value {
            Object::Int(number) => *number,
            Object::Float(number) => *number as i64,
            // These answer `to_i` but not `to_int`, so Ruby refuses them.
            Object::String(_) | Object::Array(_) | Object::Dict(_) | Object::Nil => {
                return Err(type_error(
                    format!(
                        "no implicit conversion of {} into Integer",
                        value.type_name()
                    ),
                    position,
                ));
            }
            other => match self.integer_conversion_of(other, position)? {
                Some(number) => number,
                None => {
                    return Err(type_error(
                        format!(
                            "no implicit conversion of {} into Integer",
                            other.type_name()
                        ),
                        position,
                    ));
                }
            },
        };
        if level < 0 {
            let message = format!("negative level ({})", level);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("ArgumentError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        Ok(level)
    }

    /// `to_int`, then `to_i`, answering None when the object has neither.
    /// Both are tried by sending rather than by asking `respond_to?`, which
    /// does not report the conversions a class implements natively.
    fn integer_conversion_of(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<i64>, MetorexError> {
        for name in ["to_int", "to_i"] {
            match self.send_to_object(value.clone(), name, vec![], position) {
                Ok(Object::Int(number)) => return Ok(Some(number)),
                Ok(Object::Float(number)) => return Ok(Some(number as i64)),
                Ok(_) => continue,
                Err(MetorexError::UncaughtException { exception, .. })
                    if is_no_method_error(&exception) =>
                {
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    /// The `path:line: warning: ` a message with `uplevel:` carries. Level 0
    /// names the line the `warn` call itself sits on, level 1 the line that
    /// called that method, and so on outward. A level past the top of the
    /// stack leaves only the `warning: ` part.
    pub(crate) fn warning_prefix(&self, level: i64, position: Position) -> String {
        let stack = self.call_stack();
        // Where each frame was called from, with the core library's own
        // source passed over rather than counted as a level of the program's.
        let counted: Vec<(Option<usize>, Option<String>)> = stack
            .iter()
            .map(|frame| {
                let line = frame.location().and_then(|location| {
                    location
                        .rsplit(':')
                        .nth(1)
                        .and_then(|held| held.parse::<usize>().ok())
                });
                (line, frame.source_file().map(|file| file.to_string()))
            })
            .filter(|(_, path)| {
                !path
                    .as_deref()
                    .is_some_and(|file| file.starts_with(crate::vm::INTERNAL_FILE_PREFIX))
            })
            .collect();
        let (line, path) = if level == 0 {
            (Some(position.line), self.current_source_file.clone())
        } else {
            counted
                .len()
                .checked_sub(level as usize)
                .map(|index| counted[index].clone())
                .unwrap_or_default()
        };
        let path = path.or_else(|| {
            self.current_file
                .as_ref()
                .map(|file| file.display().to_string())
        });
        match (line, path) {
            (Some(line), Some(path)) => format!("{}:{}: warning: ", path, line),
            (Some(line), None) => format!("{}: warning: ", line),
            (None, _) => "warning: ".to_string(),
        }
    }

    /// Emit an interpreter warning the way `Kernel#warn` does, through the
    /// `Warning.warn` in force so a program that replaced it sees this one too.
    pub(crate) fn warn_through_warning_module(
        &mut self,
        text: String,
        position: Position,
    ) -> Result<(), MetorexError> {
        if matches!(self.globals().get("VERBOSE"), None | Some(Object::Nil)) {
            return Ok(());
        }
        self.dispatch_to_warning_module(text, Object::Nil, position)?;
        Ok(())
    }

    /// Report that a program replaced a method the interpreter otherwise
    /// answers without a dispatch. `Warning[:performance]` asks for these,
    /// and they are silent by default.
    pub(crate) fn warn_redefined_optimized_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if !disables_optimization(class_name, method_name)
            || !self.warning_category_enabled("performance")
        {
            return Ok(());
        }
        let text = format!(
            "{}Redefining '{}#{}' disables interpreter and JIT optimizations\n",
            self.warning_prefix(0, position),
            class_name,
            method_name,
        );
        self.dispatch_to_warning_module(text, Object::symbol("performance".to_string()), position)?;
        Ok(())
    }

    /// Hand the assembled text to `Warning.warn`. MRI passes `category:` only
    /// when the method in force takes more than the one message argument, so
    /// a replacement written as `def warn(message)` still works.
    fn dispatch_to_warning_module(
        &mut self,
        text: String,
        category: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(warning) = self.globals().get("Warning") else {
            self.write_to_stderr(&text, position)?;
            return Ok(Object::Nil);
        };
        let mut arguments = vec![Object::string(text.clone())];
        if warning_warn_takes_keywords(&warning) {
            let mut keywords = indexmap::IndexMap::new();
            keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
            keywords.insert(":category".to_string(), category);
            arguments.push(Object::Dict(Rc::new(RefCell::new(keywords))));
        }
        self.send_to_object(warning, "warn", arguments, position)
    }

    /// `to_s` for one warning line, honoring a user-defined `to_s`.
    fn string_for_warning(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        match value {
            Object::Nil => Ok(String::new()),
            other => {
                let rendered = self.send_to_object(other.clone(), "to_s", vec![], position)?;
                match rendered {
                    Object::String(text) => Ok(text.as_str().to_string()),
                    other => Ok(other.to_string()),
                }
            }
        }
    }
}

/// Whether the `Warning.warn` in force accepts anything past the message. A
/// method taking exactly one required positional argument does not.
fn warning_warn_takes_keywords(warning: &Object) -> bool {
    let (Object::Module(class) | Object::Class(class)) = warning else {
        return false;
    };
    let Some(method) = class
        .singleton_class_slot()
        .as_ref()
        .and_then(|singleton| singleton.find_method("warn"))
        .or_else(|| class.find_method("__class__warn"))
    else {
        return true;
    };
    let only_the_message = method.parameters.len() == 1
        && method.variadic_param.is_none()
        && method.keyword_parameters.is_empty()
        && method.keyword_rest_parameter.is_none();
    !only_the_message
}

/// Peel a trailing keyword Hash off the argument list.
fn split_warn_keywords(mut arguments: Vec<Object>) -> (Vec<Object>, Option<Object>) {
    let is_keywords = matches!(arguments.last(), Some(Object::Dict(entries))
    if !entries.borrow().is_empty()
        && entries
            .borrow()
            .keys()
            .filter(|key| key.as_str() != "__MX_KWARGS__")
            .all(|key| match key.strip_prefix(':') {
                Some(name) => WARN_KEYWORDS.contains(&name),
                None => false,
            }));
    if is_keywords {
        let keywords = arguments.pop();
        (arguments, keywords)
    } else {
        (arguments, None)
    }
}

/// One keyword's value, when the call supplied it.
fn keyword_value(keywords: &Option<Object>, name: &str) -> Option<Object> {
    let Some(Object::Dict(entries)) = keywords else {
        return None;
    };
    entries.borrow().get(&format!(":{}", name)).cloned()
}

/// An Array argument warns with each element on its own line.
fn flatten_warn_message(message: &Object) -> Vec<Object> {
    match message {
        Object::Array(elements) => elements.borrow().clone(),
        other => vec![other.clone()],
    }
}

/// A TypeError carrying `message`.
fn type_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("TypeError", message.clone()),
        location: crate::vm::utils::position_to_location(position),
        message,
    }
}

/// Whether an exception object is a NoMethodError or NameError.
fn is_no_method_error(exception: &Object) -> bool {
    match exception {
        Object::Exception(details) => {
            matches!(
                details.borrow().exception_type.as_str(),
                "NoMethodError" | "NameError"
            )
        }
        _ => false,
    }
}

/// The core classes whose methods the interpreter answers without a dispatch,
/// with the method names it does that for.
fn disables_optimization(class_name: &str, method_name: &str) -> bool {
    let optimized: &[&str] = match class_name {
        "Integer" | "Float" => &[
            "+", "-", "*", "/", "%", "==", "<", "<=", ">", ">=", "<=>", "succ",
        ],
        "String" => &[
            "+", "*", "%", "==", "<=>", "[]", "[]=", "<<", "length", "size", "empty?", "succ",
            "include?",
        ],
        "Array" => &[
            "+", "-", "*", "==", "[]", "[]=", "<<", "length", "size", "empty?", "min", "max",
            "include?", "pack",
        ],
        "Hash" => &["==", "[]", "[]=", "length", "size", "empty?", "default"],
        "Symbol" => &["==", "<=>", "succ", "length", "size", "empty?"],
        "NilClass" | "TrueClass" | "FalseClass" => &["==", "&", "|", "nil?"],
        "Regexp" => &["==", "==="],
        "Proc" => &["call"],
        _ => return false,
    };
    optimized.contains(&method_name)
}

impl VirtualMachine {
    /// Warn that a block written at the call is passed to a method whose body
    /// never uses it: no `yield`, no block parameter, no `super` to hand it
    /// on. A verbose run warns, unless `Warning[:strict_unused_block]` was
    /// set false, and setting it true warns in any run. Each method warns
    /// once, and defining it again makes a new one.
    pub(crate) fn warn_block_the_method_ignores(
        &mut self,
        method: &crate::object::Method,
        owner: &Rc<crate::class::Class>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if !matches!(self.pending_block, Some(Object::Block(_)))
            || self.pending_block_from_ampersand
        {
            return Ok(());
        }
        let wanted = match self.warning_category_setting("strict_unused_block") {
            Some(Object::Bool(set)) => set,
            _ => matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))),
        };
        if !wanted
            || method.name == "initialize"
            || method.block_parameter.is_some()
            || method.captured_vars.is_some()
            || body_takes_a_block(&method.body)
        {
            return Ok(());
        }
        let called_in = self
            .reported_current_file()
            .map(|file| file.display().to_string())
            .unwrap_or_default();
        let written_at = method.source_location.as_ref().map(|location| {
            (
                location.filename.clone().unwrap_or_default(),
                location.line,
                location.column,
            )
        });
        if !self
            .unused_block_methods
            .insert((written_at, method.name.clone()))
        {
            return Ok(());
        }
        // The core library's own methods are written in Ruby, and Ruby's
        // are not, so they never warn.
        let written_by_metorex = method.source_location.as_ref().is_some_and(|location| {
            location
                .filename
                .as_deref()
                .is_some_and(|file| file.starts_with("<internal:") || file.starts_with("<metorex>"))
        });
        if written_by_metorex {
            return Ok(());
        }
        let (defined_in, defined_on) = match &method.source_location {
            Some(location) => (
                location
                    .filename
                    .clone()
                    .unwrap_or_else(|| called_in.clone()),
                location.line,
            ),
            None => (called_in.clone(), 0),
        };
        let holder = method
            .owner_class
            .clone()
            .or_else(|| method.definee.clone())
            .unwrap_or_else(|| Rc::clone(owner));
        // A `def self.name` is kept on the class under a prefix of its own,
        // and is named the way a singleton method is.
        let on_the_class = holder
            .find_own_method(&format!("__class__{}", method.name))
            .is_some_and(|found| std::ptr::eq(&*found, method));
        let named = if on_the_class && !holder.ruby_name().is_empty() {
            format!("{}.{}", holder.ruby_name(), method.name)
        } else {
            qualified_method_name(&holder, &method.name)
        };
        // Writing the warning calls methods of its own, which the block
        // waiting for this call must not reach.
        let waiting = self.pending_block.take();
        let from_ampersand = self.pending_block_from_ampersand;
        self.emit_warning_to_stderr(
            &format!(
                "{called_in}:{}: warning: the block passed to '{named}' defined at {defined_in}:{defined_on} may be ignored",
                position.line
            ),
            position,
        );
        self.pending_block = waiting;
        self.pending_block_from_ampersand = from_ampersand;
        Ok(())
    }
}

/// Whether a method body reaches the block it was given, through `yield` or
/// through `super`, which hands the block on. `block_given?` alone asks about
/// the block without using it.
fn body_takes_a_block(body: &[crate::ast::Statement]) -> bool {
    let written = format!("{:?}", body);
    written.contains("Yield { arguments") || written.contains("Super { arguments")
}

/// How a warning names a method: with the class it belongs to when that has
/// a name, `Class#name` for an instance method and `Class.name` for one on a
/// class itself, and alone otherwise.
fn qualified_method_name(owner: &Rc<crate::class::Class>, name: &str) -> String {
    if owner.is_singleton_class() {
        return match owner.get_class_var("__attached__") {
            Some(Object::Class(attached) | Object::Module(attached))
                if !attached.ruby_name().is_empty() && !attached.ruby_name().starts_with("#<") =>
            {
                format!("{}.{}", attached.ruby_name(), name)
            }
            _ => name.to_string(),
        };
    }
    let owner_name = owner.ruby_name();
    if owner_name.is_empty() || owner_name.starts_with("#<") {
        name.to_string()
    } else {
        format!("{owner_name}#{name}")
    }
}
