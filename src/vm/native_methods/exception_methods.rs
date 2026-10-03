//! Native method implementations for Exception objects

use super::super::VirtualMachine;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::utils::*;
use std::cell::RefCell;
use std::rc::Rc;

impl VirtualMachine {
    /// What Ruby prints for an exception nothing rescued: the place it was
    /// raised, its message and class, and the frames under it.
    pub fn uncaught_report(&mut self, exception: &Object) -> Option<String> {
        if let Object::Exception(held) = exception
            && held.borrow().exception_type == "SystemExit"
        {
            // `exit` ends the program rather than failing it, so there is
            // nothing to report.
            return Some(String::new());
        }
        let mut keywords = indexmap::IndexMap::new();
        keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
        keywords.insert(":highlight".to_string(), Object::Bool(false));
        let arguments = vec![Object::Dict(Rc::new(RefCell::new(keywords)))];
        match self.send_to_object(
            exception.clone(),
            "full_message",
            arguments,
            Position::default(),
        ) {
            Ok(Object::String(text)) => Some(text.as_str().to_string()),
            _ => None,
        }
    }

    /// Call a native method on an Exception object
    pub(crate) fn call_exception_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let exception = match receiver {
            Object::Exception(exc) => exc,
            // The Exception class object itself is not an exception, so its
            // methods are looked up elsewhere.
            Object::Class(_) | Object::Module(_) => return Ok(None),
            _ => {
                return Err(MetorexError::runtime_error(
                    format!("Expected Exception, got {:?}", receiver),
                    position_to_location(position),
                ));
            }
        };

        match method_name {
            // What Marshal writes for the message: the one given, or nil when
            // the exception reports its class name for lack of one.
            "__given_message__" => {
                let details = exception.borrow();
                if let Some(given) = details.instance_vars.get(crate::vm::MESSAGE_STRING_KEY) {
                    return Ok(Some(given.clone()));
                }
                Ok(Some(
                    if details.message_given || !details.message.is_empty() {
                        Object::string(details.message.clone())
                    } else {
                        Object::Nil
                    },
                ))
            }
            // What Marshal reads back for the message and the cause.
            "__restore_message__" => {
                let mut details = exception.borrow_mut();
                match arguments.first() {
                    Some(given @ Object::String(text)) => {
                        details.message = text.as_str().to_string();
                        details.message_given = true;
                        details
                            .instance_vars
                            .insert(crate::vm::MESSAGE_STRING_KEY.to_string(), given.clone());
                    }
                    _ => {
                        details.message = String::new();
                        details.message_given = false;
                    }
                }
                Ok(Some(Object::Nil))
            }
            // A NameError's name and a NoMethodError's arguments, which
            // Marshal reads back under names that are not instance variables.
            "__restore_attribute__" => {
                let key = match arguments.first() {
                    Some(Object::Symbol(named)) if &*named.as_str() == "args" => {
                        crate::vm::NO_METHOD_ARGS_KEY
                    }
                    _ => crate::vm::NAME_ERROR_NAME_KEY,
                };
                let value = arguments.get(1).cloned().unwrap_or(Object::Nil);
                exception
                    .borrow_mut()
                    .instance_vars
                    .insert(key.to_string(), value);
                Ok(Some(Object::Nil))
            }
            "__restore_cause__" => {
                let mut details = exception.borrow_mut();
                details.cause = match arguments.first() {
                    None | Some(Object::Nil) => None,
                    Some(held) => Some(Box::new(held.clone())),
                };
                details.cause_settled = true;
                Ok(Some(Object::Nil))
            }
            // Ruby's `#message` is `to_s`, which answers the class name when
            // there is no message of its own. The call is dispatched, so a
            // subclass that redefines `to_s` decides what its message is.
            "message" => self
                .send_to_object(receiver.clone(), "to_s", vec![], position)
                .map(Some),
            // `#exception` answers self with no argument or when handed self,
            // and otherwise a copy carrying the new message. The copy is not
            // re-initialized, so a subclass keeps the state it set.
            "exception" => {
                match arguments.first() {
                    None => return Ok(Some(receiver.clone())),
                    Some(argument) if argument.equals(receiver) => {
                        return Ok(Some(receiver.clone()));
                    }
                    _ => {}
                }
                if arguments.len() > 1 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Range(0, 1),
                        arguments.len(),
                        position,
                    ));
                }
                let message = self.coerce_name_argument(&arguments[0], position)?;
                let copy = exception.borrow().clone();
                let copy = std::rc::Rc::new(RefCell::new(copy));
                {
                    let mut copy = copy.borrow_mut();
                    copy.message = message;
                    copy.message_given = true;
                }
                Ok(Some(Object::Exception(copy)))
            }
            // SignalException#signo / #signm — the number the signal carries
            // and the name it goes by. Absent on an exception no signal
            // raised, where the lookup carries on to a NoMethodError.
            "signo" | "signm" => {
                let number = exception
                    .borrow()
                    .instance_vars
                    .get(crate::vm::signals::SIGNO_KEY)
                    .cloned();
                match number {
                    None => Ok(None),
                    Some(number) if method_name == "signo" => Ok(Some(number)),
                    Some(_) => self.call_exception_method(receiver, "to_s", &[], position),
                }
            }
            // NoMethodError#args — the arguments the failed call was made
            // with, nil on one constructed without them.
            "args" => {
                let details = exception.borrow();
                if let Some(args) = details.instance_vars.get(crate::vm::NO_METHOD_ARGS_KEY) {
                    return Ok(Some(args.clone()));
                }
                let is_no_method_error = match &details.class {
                    Some(class) => {
                        crate::vm::method_invocation::descends_from(class, "NoMethodError")
                    }
                    None => details.exception_type == "NoMethodError",
                };
                Ok(is_no_method_error.then_some(Object::Nil))
            }
            // LoadError#path — the feature that could not be loaded.
            // SyntaxError#path — the file the unparsable code came from. Both
            // answer nil on an exception raised by nothing in particular.
            "path" => {
                let details = exception.borrow();
                if let Some(path) = details.instance_vars.get(crate::vm::EXCEPTION_PATH_KEY) {
                    return Ok(Some(path.clone()));
                }
                let has_path = match &details.class {
                    Some(class) => ["LoadError", "SyntaxError"]
                        .iter()
                        .any(|name| crate::vm::method_invocation::descends_from(class, name)),
                    None => matches!(details.exception_type.as_str(), "LoadError" | "SyntaxError"),
                };
                Ok(has_path.then_some(Object::Nil))
            }
            // UncaughtThrowError#tag / #value — the tag `throw` was called
            // with and the value it carried, both nil on one raised by no
            // throw at all.
            "tag" | "value" => {
                let details = exception.borrow();
                let key = match method_name {
                    "tag" => crate::vm::THROW_TAG_KEY,
                    _ => crate::vm::THROW_VALUE_KEY,
                };
                if let Some(thrown) = details.instance_vars.get(key) {
                    return Ok(Some(thrown.clone()));
                }
                let is_throw_error = match &details.class {
                    Some(class) => {
                        crate::vm::method_invocation::descends_from(class, "UncaughtThrowError")
                    }
                    None => details.exception_type == "UncaughtThrowError",
                };
                Ok(is_throw_error.then_some(Object::Nil))
            }
            // LocalJumpError#exit_value and #reason — the value a stray
            // `return` carried and what kind of jump it was.
            "exit_value" | "reason" => {
                let details = exception.borrow();
                let key = format!("@{}", method_name);
                Ok(details.instance_vars.get(&key).cloned())
            }
            // NoMatchingPatternKeyError#matchee — the hash the missing key was
            // looked for in.
            "matchee" => {
                let details = exception.borrow();
                Ok(details.instance_vars.get(crate::vm::MATCHEE_KEY).cloned())
            }
            // KeyError#key — the lookup that missed, absent on an exception
            // no failed lookup raised.
            "key" => {
                let details = exception.borrow();
                if let Some(key) = details.instance_vars.get(crate::vm::KEY_ERROR_KEY) {
                    return Ok(Some(key.clone()));
                }
                let is_key_error = match &details.class {
                    Some(class) => crate::vm::method_invocation::descends_from(class, "KeyError"),
                    None => details.exception_type == "KeyError",
                };
                drop(details);
                if !is_key_error {
                    return Ok(None);
                }
                Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "no key is available",
                    position,
                ))
            }
            // NameError#receiver / NoMethodError#receiver — the object the
            // call was made on, nil when unset.
            "receiver" => {
                let details = exception.borrow();
                if let Some(value) = details.receiver.clone() {
                    return Ok(Some(*value));
                }
                let carries_receiver = match &details.class {
                    Some(class) => ["NameError", "FrozenError", "KeyError"]
                        .iter()
                        .any(|name| crate::vm::method_invocation::descends_from(class, name)),
                    None => matches!(
                        details.exception_type.as_str(),
                        "NameError" | "NoMethodError" | "FrozenError" | "KeyError"
                    ),
                };
                drop(details);
                if !carries_receiver {
                    return Ok(None);
                }
                // Ruby refuses to invent one, so an exception raised with no
                // receiver has none to report.
                Err(crate::vm::errors::simple_exception(
                    "ArgumentError",
                    "no receiver is available",
                    position,
                ))
            }
            "name" => {
                // NameError#name / NoMethodError#name — the offending
                // constant or method name as a Symbol, nil when unset. A
                // name handed in by the caller comes back as that object.
                if let Some(value) = exception
                    .borrow()
                    .instance_vars
                    .get(crate::vm::NAME_ERROR_NAME_KEY)
                {
                    return Ok(Some(value.clone()));
                }
                let name = exception.borrow().name.clone();
                Ok(Some(match name {
                    Some(n) => Object::symbol(n),
                    None => Object::Nil,
                }))
            }
            // SystemExit#status — the exit status the exception carries.
            "status" if exception.borrow().is_system_exit() => {
                let status = exception.borrow().status.unwrap_or(0);
                Ok(Some(Object::Int(status)))
            }
            // SystemExit#success? — true when the status means a clean exit.
            "success?" if exception.borrow().is_system_exit() => {
                let status = exception.borrow().status.unwrap_or(0);
                Ok(Some(Object::Bool(status == 0)))
            }
            "type" | "exception_type" => {
                // Return the exception type as a String
                let exception_type = exception.borrow().exception_type.clone();
                Ok(Some(Object::string(exception_type)))
            }
            // `set_backtrace` accepts nil, a String, or an Array of Strings,
            // and refuses anything else.
            "set_backtrace" => {
                if arguments.len() != 1 {
                    return Err(crate::vm::errors::argument_count_error(
                        crate::vm::errors::Arity::Exact(1),
                        arguments.len(),
                        position,
                    ));
                }
                let (trace, kept) = match &arguments[0] {
                    Object::Nil => (None, None),
                    Object::String(line) => (Some(vec![line.as_str().to_string()]), None),
                    Object::Array(entries) => {
                        let mut lines = Vec::with_capacity(entries.borrow().len());
                        let mut sites = Vec::with_capacity(entries.borrow().len());
                        for entry in entries.borrow().iter() {
                            match entry {
                                Object::String(line) => lines.push(line.as_str().to_string()),
                                // Ruby 3.4 also accepts Location objects, and
                                // reports them through both accessors.
                                Object::Instance(instance)
                                    if instance.borrow().class.name()
                                        == "Thread::Backtrace::Location" =>
                                {
                                    let borrowed = instance.borrow();
                                    let path = match borrowed.get_var("path") {
                                        Some(Object::String(path)) => path.as_str().to_string(),
                                        _ => String::new(),
                                    };
                                    let line = match borrowed.get_var("lineno") {
                                        Some(Object::Int(line)) => *line as usize,
                                        _ => 0,
                                    };
                                    let label = match borrowed.get_var("label") {
                                        Some(Object::String(label)) => label.as_str().to_string(),
                                        _ => String::new(),
                                    };
                                    lines.push(if label.is_empty() {
                                        format!("{}:{}", path, line)
                                    } else {
                                        format!("{}:{}:in '{}'", path, line, label)
                                    });
                                    sites.push((path, line, label));
                                }
                                other => return Err(backtrace_type_error(other, position)),
                            }
                        }
                        let given_locations = !sites.is_empty();
                        if given_locations {
                            let mut details = exception.borrow_mut();
                            details.backtrace_sites = Some(sites);
                            details.backtrace_locations_array = Some(arguments[0].clone());
                        }
                        // Ruby keeps the very Array of Strings it was handed,
                        // so a later `backtrace` answers the identical object.
                        // An Array of Locations is a different matter: those
                        // are what `backtrace_locations` reports, and
                        // `backtrace` renders them as Strings.
                        let kept = if given_locations {
                            None
                        } else {
                            Some(arguments[0].clone())
                        };
                        (Some(lines), kept)
                    }
                    other => return Err(backtrace_type_error(other, position)),
                };
                let mut details = exception.borrow_mut();
                details.backtrace = trace;
                details.backtrace_array = kept;
                Ok(Some(arguments[0].clone()))
            }
            // Ruby renders an exception as `#<ClassName: message>`, using
            // whatever `to_s` answers, and as the class name alone when that
            // is empty.
            "inspect" => {
                let rendered =
                    match self.send_to_object(receiver.clone(), "to_s", vec![], position)? {
                        Object::String(text) => text.as_str().to_string(),
                        other => other.to_string(),
                    };
                let class_name = {
                    let details = exception.borrow();
                    match &details.class {
                        // An anonymous class shows its generated label.
                        Some(class) => class.inspect_name(),
                        None => details.exception_type.clone(),
                    }
                };
                Ok(Some(Object::string(if rendered.is_empty() {
                    class_name
                } else {
                    format!("#<{}: {}>", class_name, rendered)
                })))
            }
            // Ruby's `#full_message` is the rendering an uncaught exception
            // gets on stderr: the backtrace plus the detailed message, with
            // `order:` deciding which end the message sits at.
            "full_message" => {
                // Ruby paints the report only when it is going to a
                // terminal, which is what `Exception.to_tty?` reports.
                let highlight = match keyword_argument(arguments, "highlight") {
                    Some(Object::Bool(asked)) => asked,
                    Some(Object::Nil) | None => unsafe { libc::isatty(2) == 1 },
                    Some(_) => true,
                };
                let bottom_first = matches!(
                    keyword_argument(arguments, "order"),
                    Some(Object::Symbol(order)) if *order.as_str() == *"bottom"
                );
                // Through `send` so a class that overrides `detailed_message`
                // decides how its own message reads. Every keyword this call
                // was given is handed on, `order:` aside, with `highlight:`
                // settled first.
                let mut keywords = indexmap::IndexMap::new();
                keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
                if let Some(Object::Dict(given)) = arguments.last() {
                    for (name, value) in given.borrow().iter() {
                        if name == "__MX_KWARGS__" || name == ":order" || name == ":highlight" {
                            continue;
                        }
                        keywords.insert(name.clone(), value.clone());
                    }
                }
                keywords.insert(":highlight".to_string(), Object::Bool(highlight));
                let detail_arguments = vec![Object::Dict(Rc::new(RefCell::new(keywords)))];
                // An exception that answers no `detailed_message`, or answers
                // nil, is named by its class the way Ruby names it.
                let stand_in = self.exception_class_label(exception, highlight);
                // The exception answers `detailed_message` unless a program
                // took it away, which `undef` does by leaving a tombstone.
                let answers = !self
                    .lookup_method(receiver, "detailed_message")
                    .is_some_and(|(_, held)| held.is_undefined);
                let detail = if answers {
                    match self.send_to_object(
                        receiver.clone(),
                        "detailed_message",
                        detail_arguments,
                        position,
                    )? {
                        Object::String(text) => text.as_str().to_string(),
                        Object::Nil => stand_in,
                        other => self.message_text_of(other, position)?,
                    }
                } else {
                    stand_in
                };
                let trace = exception.borrow().backtrace.clone().unwrap_or_default();
                // An exception carrying no backtrace is reported against the
                // line that asked for the report.
                let origin = match trace.first() {
                    Some(entry) => entry.clone(),
                    None => format!(
                        "{}:{}:in '{}'",
                        self.current_source_file.clone().unwrap_or_default(),
                        position.line,
                        self.get_current_method_name().unwrap_or("<main>")
                    ),
                };
                let mut rest: Vec<String> = trace.iter().skip(1).cloned().collect();
                // `--backtrace-limit` says how many frames under the top one
                // are written out, with a count standing for the rest.
                let mut left_out = 0;
                if self.backtrace_limit >= 0 {
                    let kept = self.backtrace_limit as usize;
                    if rest.len() > kept {
                        left_out = rest.len() - kept;
                        rest.truncate(kept);
                    }
                }
                let mut lines = Vec::new();
                if bottom_first {
                    lines.push(if highlight {
                        "\u{1b}[1mTraceback\u{1b}[m (most recent call last):".to_string()
                    } else {
                        "Traceback (most recent call last):".to_string()
                    });
                    for entry in rest.iter().rev() {
                        lines.push(format!("\tfrom {}", entry));
                    }
                    lines.push(format!("{}: {}", origin, detail));
                } else {
                    lines.push(format!("{}: {}", origin, detail));
                    for entry in &rest {
                        lines.push(format!("\tfrom {}", entry));
                    }
                    if left_out > 0 {
                        lines.push(format!("\t ... {} levels...", left_out));
                    }
                }
                let mut rendered = lines.join("\n");
                rendered.push('\n');
                // Ruby appends each cause after the exception's own report,
                // so the whole chain is visible.
                let mut cause = exception.borrow().cause.clone();
                while let Some(next) = cause {
                    let next = *next;
                    let Object::Exception(details) = &next else {
                        break;
                    };
                    let reported = match self.send_to_object(
                        next.clone(),
                        "detailed_message",
                        vec![],
                        position,
                    )? {
                        Object::String(text) => text.as_str().to_string(),
                        other => other.to_string(),
                    };
                    // A cause is reported the way the exception itself is:
                    // where it was raised, then what it was raised through.
                    let held = details.borrow().backtrace.clone().unwrap_or_default();
                    match held.first() {
                        Some(first) => {
                            rendered.push_str(&format!("{}: {}\n", first, reported));
                            for entry in held.iter().skip(1) {
                                rendered.push_str(&format!("\tfrom {}\n", entry));
                            }
                        }
                        None => rendered.push_str(&format!("{}\n", reported)),
                    }
                    cause = details.borrow().cause.clone();
                }
                Ok(Some(Object::string(rendered)))
            }
            // Ruby's `#detailed_message` decorates the message with the class
            // name, or stands in for an empty message. `highlight: true` wraps
            // it in the escape sequences a terminal shows in bold.
            "detailed_message" => {
                let highlight = matches!(
                    keyword_argument(arguments, "highlight"),
                    Some(Object::Bool(true))
                );
                let (message, class_name, given) = {
                    let details = exception.borrow();
                    let class_name = match &details.class {
                        Some(class) => class.ruby_name(),
                        None => details.exception_type.clone(),
                    };
                    (details.message.clone(), class_name, details.message_given)
                };
                // The report shows the message `#message` answers, which for
                // an exception built with none is the class's own name.
                let message = if message.is_empty() && !given {
                    class_name.clone()
                } else {
                    message
                };
                let rendered = if message.is_empty() {
                    // An empty message shows the class name instead, except
                    // for RuntimeError, which Ruby labels this way.
                    let stand_in = if class_name == "RuntimeError" {
                        "unhandled exception".to_string()
                    } else if class_name.is_empty() {
                        match &exception.borrow().class {
                            Some(class) => class.inspect_name(),
                            None => String::new(),
                        }
                    } else {
                        class_name
                    };
                    if highlight {
                        format!("\u{1b}[1;4m{}\u{1b}[m", stand_in)
                    } else {
                        stand_in
                    }
                } else if class_name.is_empty() {
                    // An anonymous class has no name to decorate with.
                    message
                } else {
                    // The class is named at the end of the first line, and
                    // the lines under it follow as they were written.
                    let mut lines = message.split('\n');
                    let first = lines.next().unwrap_or_default();
                    let rest: Vec<&str> = lines.collect();
                    let mut written = if highlight {
                        format!(
                            "\u{1b}[1m{} (\u{1b}[1;4m{}\u{1b}[m\u{1b}[1m)\u{1b}[m",
                            first, class_name
                        )
                    } else {
                        format!("{} ({})", first, class_name)
                    };
                    for line in rest {
                        written.push('\n');
                        if highlight {
                            written.push_str(&format!("\u{1b}[1m{}\u{1b}[m", line));
                        } else {
                            written.push_str(line);
                        }
                    }
                    written
                };
                Ok(Some(Object::string(rendered)))
            }
            // `Exception#cause` is the exception a rescue clause was handling
            // when this one was raised, nil when there was none.
            "cause" => {
                let cause = exception.borrow().cause.clone();
                Ok(Some(match cause {
                    Some(value) => *value,
                    None => Object::Nil,
                }))
            }
            // `SystemCallError#errno` is the number it was built with, and
            // failing that the one its class carries.
            "errno" => {
                if let Some(number) = exception
                    .borrow()
                    .instance_vars
                    .get(crate::vm::system_call_error::ERRNO_VALUE_KEY)
                {
                    return Ok(Some(number.clone()));
                }
                let exception_type = exception.borrow().exception_type.clone();
                let Some(Object::Class(class)) = self
                    .globals()
                    .get(&exception_type)
                    .or_else(|| self.resolve_qualified_constant(&exception_type))
                else {
                    return Ok(Some(Object::Nil));
                };
                Ok(Some(class.get_class_var("Errno").unwrap_or(Object::Nil)))
            }
            // Like `#backtrace`, nil until the exception is raised, and the
            // same Array on every call afterwards.
            "backtrace_locations" => {
                if let Some(cached) = exception.borrow().backtrace_locations_array.clone() {
                    return Ok(Some(cached));
                }
                let Some(sites) = exception.borrow().backtrace_sites.clone() else {
                    return Ok(Some(Object::Nil));
                };
                let location_class = self.backtrace_location_class();
                let entries: Vec<Object> = sites
                    .iter()
                    .map(|(path, line, label)| {
                        let instance =
                            crate::object::Instance::new(std::rc::Rc::clone(&location_class));
                        {
                            let mut filling = instance.borrow_mut();
                            filling.set_var("path".to_string(), Object::string(path.clone()));
                            filling.set_var("lineno".to_string(), Object::Int(*line as i64));
                            filling.set_var("label".to_string(), Object::string(label.clone()));
                            filling.set_var(
                                "absolute_path".to_string(),
                                Object::string(absolute_path(path)),
                            );
                        }
                        Object::Instance(instance)
                    })
                    .collect();
                let array = Object::Array(Rc::new(RefCell::new(entries)));
                exception.borrow_mut().backtrace_locations_array = Some(array.clone());
                Ok(Some(array))
            }
            // An exception that was never raised has no backtrace at all, so
            // Ruby answers nil rather than an empty Array. The Array itself is
            // built once and kept, so `backtrace.equal?(backtrace)` holds and
            // an update through it survives.
            "backtrace" => {
                if let Some(cached) = exception.borrow().backtrace_array.clone() {
                    return Ok(Some(cached));
                }
                let Some(trace) = exception.borrow().backtrace.clone() else {
                    return Ok(Some(Object::Nil));
                };
                let entries: Vec<Object> = trace
                    .iter()
                    .map(|line| Object::string(line.clone()))
                    .collect();
                let array = Object::Array(Rc::new(RefCell::new(entries)));
                exception.borrow_mut().backtrace_array = Some(array.clone());
                Ok(Some(array))
            }
            // Ruby's `Exception#to_s` is the message alone, and the class
            // name when there is no message. Neither the location nor the
            // backtrace appears here.
            "to_s" => {
                let exc = exception.borrow();
                let rendered = if exc.message_given {
                    exc.message.clone()
                } else {
                    // An exception built with no message says what class it
                    // is, which for an anonymous one is how that class writes
                    // itself rather than the name of an ancestor.
                    match &exc.class {
                        Some(class) if class.ruby_name().is_empty() => class.inspect_name(),
                        Some(class) => class.ruby_name(),
                        None => exc.exception_type.clone(),
                    }
                };
                Ok(Some(Object::string(rendered)))
            }
            _ => Ok(None), // No native method found, let it fall through
        }
    }

    /// The class an exception names itself by where its message is not there
    /// to be shown, painted the way a report paints it.
    fn exception_class_label(
        &self,
        exception: &Rc<RefCell<crate::object::Exception>>,
        highlight: bool,
    ) -> String {
        let details = exception.borrow();
        let named = match &details.class {
            Some(class) => class.ruby_name(),
            None => details.exception_type.clone(),
        };
        if highlight {
            format!("\u{1b}[1;4m{}\u{1b}[m", named)
        } else {
            named
        }
    }

    /// The text a `detailed_message` answered with stands for, which Ruby
    /// reads through `to_str` where the answer is not a String already.
    fn message_text_of(
        &mut self,
        held: Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if self.responds_to(&held, "to_str")
            && let Object::String(text) =
                self.send_to_object(held.clone(), "to_str", vec![], position)?
        {
            return Ok(text.as_str().to_string());
        }
        match self.send_to_object(held, "to_s", vec![], position)? {
            Object::String(text) => Ok(text.as_str().to_string()),
            other => Ok(other.to_string()),
        }
    }
}

/// The TypeError `set_backtrace` raises for anything that is not a String.
fn backtrace_type_error(value: &Object, position: Position) -> MetorexError {
    let message = format!(
        "backtrace must be an Array of String, got {}",
        crate::vm::native_methods::define_method::ruby_class_name(value)
    );
    MetorexError::UncaughtException {
        exception: Object::exception("TypeError", message.clone()),
        location: crate::vm::utils::position_to_location(position),
        message,
    }
}

/// The methods every Exception answers natively, whatever its class.
pub(crate) const EVERY_EXCEPTION_METHOD: &[&str] = &[
    "message",
    "detailed_message",
    "full_message",
    "to_s",
    "inspect",
    "backtrace",
    "backtrace_locations",
    "set_backtrace",
    "cause",
    "exception",
    "==",
];

/// The methods an Exception answers natively. A class carries no entry for
/// any of them, so `undef_method` reads this list to know they are there.
pub(crate) const NATIVE_EXCEPTION_METHODS: &[&str] = &[
    "message",
    "detailed_message",
    "full_message",
    "to_s",
    "inspect",
    "backtrace",
    "backtrace_locations",
    "set_backtrace",
    "cause",
    "exception",
    "==",
    "key",
    "matchee",
    "name",
    "receiver",
    "status",
    "success?",
];

/// One keyword argument from a trailing keyword Hash, when the call had one.
fn keyword_argument(arguments: &[Object], name: &str) -> Option<Object> {
    let Some(Object::Dict(entries)) = arguments.last() else {
        return None;
    };
    entries.borrow().get(&format!(":{}", name)).cloned()
}

/// A backtrace path as an absolute one, which `Location#absolute_path` answers
/// alongside the path as written.
fn absolute_path(path: &str) -> String {
    std::path::Path::new(path)
        .canonicalize()
        .map(|resolved| resolved.display().to_string())
        .unwrap_or_else(|_| path.to_string())
}
