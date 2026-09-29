// The Regexp class methods.

use super::*;

impl VirtualMachine {
    /// Building a pattern, escaping a string into one, and the match the
    /// last one left behind.
    pub(crate) fn call_regexp_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // `Module.nesting` returns the modules/classes currently being defined
        // (innermost first). We approximate with the def_scope_stack snapshot.
        // Regexp.escape / Regexp.quote: the string with every regex
        // metacharacter escaped, so it matches itself literally.
        if class_rc.name() == "Regexp"
            && matches!(method_name, "escape" | "quote")
            && arguments.len() == 1
        {
            let source = self.coerce_name_argument(&arguments[0], position)?;
            let named = match &arguments[0] {
                Object::String(held) => held.encoding_name(),
                _ => "US-ASCII".to_string(),
            };
            let escaped = quoted_for_pattern(&source);
            // Ruby tags the answer US-ASCII when nothing but ASCII went in,
            // and keeps the source's own encoding otherwise.
            let tagged = if source.is_ascii() {
                "US-ASCII".to_string()
            } else {
                named
            };
            let made = crate::object::StringValue::with_encoding(escaped, tagged);
            // Characters standing for bytes go on standing for them in the
            // pattern built out of them.
            if matches!(&arguments[0], Object::String(held) if held.holds_bytes()) {
                made.mark_bytes();
            }
            return Ok(Answered(Object::String(Rc::new(made))));
        }
        // `Regexp.last_match` answers the whole MatchData, and with a number
        // the capture that number names.
        if class_rc.name() == "Regexp" && method_name == "last_match" {
            let last = self
                .globals()
                .get(crate::vm::native_methods::LAST_MATCH)
                .unwrap_or(Object::Nil);
            return match arguments.first() {
                None => Ok(Answered(last)),
                Some(_) if matches!(last, Object::Nil) => Ok(Answered(Object::Nil)),
                Some(index) => self
                    .send_to_object(last, "[]", vec![index.clone()], position)
                    .map(Answered),
            };
        }
        // `Regexp.new` / `Regexp.compile` build a pattern from a source
        // string, with the second argument turning case folding on.
        // `__literal__` is how a pattern written out with `#{}` in it is
        // built once its parts are known. A literal names its encoding with
        // `n` and `u`, which `new` refuses.
        if class_named_in_chain(class_rc, "Regexp")
            && matches!(method_name, "new" | "compile" | "__literal__")
        {
            let written_out = method_name == "__literal__";
            // A pattern written with `o` is built once and stands for every
            // time the line it is written on is reached again.
            let built_once = match arguments.get(2) {
                Some(Object::String(site)) => Some(site.as_str().to_string()),
                _ => None,
            };
            if let Some(site) = &built_once
                && let Some(held) = self.patterns_built_once.get(site)
            {
                return Ok(Answered(held.clone()));
            }
            let source_encoding = match arguments.first() {
                Some(Object::String(text)) => Some(text.encoding_name()),
                _ => None,
            };
            let source = match arguments.first() {
                Some(Object::Regex(pattern, flags)) => {
                    // A pattern built from another one keeps that one's flags,
                    // and says so about any written alongside it.
                    if arguments.len() > 1 && !matches!(arguments[1], Object::Nil) {
                        self.emit_warning_to_stderr("warning: flags ignored", position);
                    }
                    let made = Object::Regex(Rc::clone(pattern), Rc::clone(flags));
                    if class_rc.name() == "Regexp" {
                        return Ok(Answered(made));
                    }
                    // A subclass keeps the pattern it was built from.
                    let instance = crate::object::Instance::new(Rc::clone(class_rc));
                    instance.borrow_mut().set_var(
                        crate::vm::native_methods::REGEXP_SUBCLASS_VAR.to_string(),
                        made,
                    );
                    return Ok(Answered(Object::Instance(instance)));
                }
                // Only a String, or something answering `to_str`, spells a
                // pattern: a Symbol names no source.
                Some(Object::String(text)) => text.as_str().to_string(),
                Some(argument) => {
                    let named = self.builtins().class_of(argument).name().to_string();
                    if !self.answers_to(argument, "to_str", position)? {
                        let message = format!("no implicit conversion of {} into String", named);
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            position,
                        ));
                    }
                    match self.send_to_object(argument.clone(), "to_str", vec![], position)? {
                        Object::String(text) => text.as_str().to_string(),
                        _ => {
                            let message = format!("can\'t convert {} into String", named);
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &message,
                                position,
                            ));
                        }
                    }
                }
                None => {
                    return Err(method_argument_error("new", 1, 0, position));
                }
            };
            refuse_bad_pattern(&source, position)?;
            let mut flags = String::new();
            match arguments.get(1) {
                Some(Object::Int(options)) => {
                    if options & 1 != 0 {
                        flags.push('i');
                    }
                    if options & 2 != 0 {
                        flags.push('x');
                    }
                    if options & 4 != 0 {
                        flags.push('m');
                    }
                    // A pattern told to match in one encoding whatever the
                    // text is tagged with carries that as the `u` a literal
                    // would have been written with, and one told to match
                    // bytes carries the `n`.
                    if options & 16 != 0 {
                        flags.push('u');
                    }
                    if options & 32 != 0 {
                        flags.push('n');
                    }
                }
                // A String names the flags by the letters a literal is
                // written with, which are the three that change how the
                // pattern reads and the two that name the encoding it
                // matches in.
                Some(Object::String(written)) => {
                    let spelling = written.as_str().to_string();
                    if spelling.chars().any(|held| {
                        !(matches!(held, 'i' | 'm' | 'x')
                            || (written_out && matches!(held, 'n' | 'u' | 'o')))
                    }) {
                        let message = format!("unknown regexp option: {}", spelling);
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &message,
                            position,
                        ));
                    }
                    // `o` says when the pattern is built rather than what it
                    // matches, so the built one does not carry it.
                    for held in ['i', 'm', 'x', 'n', 'u'] {
                        if spelling.contains(held) {
                            flags.push(held);
                        }
                    }
                }
                Some(Object::Bool(true)) => flags.push('i'),
                Some(Object::Bool(false)) | Some(Object::Nil) | None => {}
                // A keyword hash in that place names `timeout:`, which is
                // read below rather than as an ignorecase argument.
                Some(Object::Dict(_)) => {}
                Some(other) => {
                    // Anything else is read as a plain truth, which Ruby says
                    // so about rather than asking it for a number.
                    let shown = self.send_to_object(other.clone(), "inspect", vec![], position)?;
                    let message =
                        format!("warning: expected true or false as ignorecase: {}", shown);
                    self.emit_warning_to_stderr(&message, position);
                    flags.push('i');
                }
            }
            // `timeout:` says how long a match of this pattern may take,
            // which stands in place of what the class names.
            if let Some(Object::Dict(options)) = arguments.get(1).or(arguments.get(2))
                && let Some(named) = options.borrow().get(":timeout")
            {
                let held = match named {
                    Object::Nil => None,
                    other => {
                        let seconds = self.float_value_of(other, position)?;
                        if seconds <= 0.0 {
                            let shown =
                                self.send_to_object(other.clone(), "to_s", vec![], position)?;
                            let message = match &shown {
                                Object::String(held) => {
                                    format!("invalid timeout: {}", held.as_str())
                                }
                                held => format!("invalid timeout: {held}"),
                            };
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                &message,
                                position,
                            ));
                        }
                        Some(std::time::Duration::from_secs_f64(seconds))
                    }
                };
                self.pattern_timeouts.insert(source.clone(), held);
            }
            let built = Rc::new(source);
            // A pattern built here is not frozen, which is what tells it
            // apart from one written as a literal.
            self.record_built_pattern(&built);
            if let Some(named) = source_encoding {
                self.record_pattern_encoding(&built, named);
            }
            let made = Object::Regex(built, Rc::new(flags));
            if let Some(site) = built_once {
                self.patterns_built_once.insert(site, made.clone());
            }
            if class_rc.name() == "Regexp" {
                return Ok(Answered(made));
            }
            // A subclass answers Regexp's methods through the pattern it
            // keeps, which is what lets it be a Regexp and its own class at
            // once.
            let instance = crate::object::Instance::new(Rc::clone(class_rc));
            instance.borrow_mut().set_var(
                crate::vm::native_methods::REGEXP_SUBCLASS_VAR.to_string(),
                made,
            );
            let built = Object::Instance(instance);
            // A subclass writing its own `initialize` sees the arguments the
            // pattern was built from.
            if let Some((owner, method)) = self.lookup_method(&built, "initialize")
                && !method.is_undefined
                && !method.body.is_empty()
            {
                if let Object::Instance(held) = &built {
                    held.borrow_mut()
                        .set_var("__building_regexp__".to_string(), Object::Bool(true));
                }
                let outcome =
                    self.invoke_method(owner, method, built.clone(), arguments.to_vec(), position);
                if let Object::Instance(held) = &built {
                    held.borrow_mut()
                        .instance_vars
                        .shift_remove("__building_regexp__");
                }
                outcome?;
            }
            return Ok(Answered(built));
        }
        // `Regexp.union` matches any of what it was given.
        if class_rc.name() == "Regexp" && method_name == "union" {
            return self.pattern_union(arguments, position).map(Answered);
        }
        // Module.used_refinements: the refinements `using` has brought into
        // the current scope.
        if method_name == "used_refinements" && class_rc.name() == "Module" {
            let refinements = self.active_refinements();
            return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                refinements,
            )))));
        }
        if method_name == "nesting" && class_rc.name() == "Module" {
            // Inside a method body the answer is the nesting captured where
            // the method was defined; elsewhere it is the scopes open here.
            let scopes = match self.method_nesting_stack.last() {
                Some(captured) => captured.clone(),
                None => self.snapshot_lexical_nesting(),
            };
            let nesting: Vec<Object> = scopes
                .into_iter()
                .map(|scope| {
                    if scope.is_module() {
                        Object::Module(scope)
                    } else {
                        Object::Class(scope)
                    }
                })
                .collect();
            return Ok(Answered(Object::Array(Rc::new(std::cell::RefCell::new(
                nesting,
            )))));
        }
        Ok(Unclaimed)
    }
}

/// Where a pattern given to `Regexp.union` stands on encoding: in one that
/// spells ASCII another way, fixed to one that spells ASCII as ASCII, or
/// plain ASCII that reads in any of them.
enum UnionEncoding {
    AsciiIncompatible(String),
    Fixed(String),
    AsciiOnly,
}

impl VirtualMachine {
    /// A pattern matching any of the ones given. A String is quoted and a
    /// Regexp is written with its own options, and the union takes the
    /// encoding of whichever patterns need one, refusing ones that cannot be
    /// read together.
    fn pattern_union(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let parts: Vec<Object> = match arguments {
            [Object::Array(array)] => array.borrow().clone(),
            _ => arguments.to_vec(),
        };
        if parts.is_empty() {
            return Ok(Object::Regex(
                Rc::new("(?!)".to_string()),
                Rc::new(String::new()),
            ));
        }
        // A single pattern is answered as it stands.
        if let [only] = parts.as_slice()
            && let Some(pattern) = self.union_pattern_argument(only, position)?
        {
            return Ok(pattern);
        }
        let mut incompatible: Option<String> = None;
        let mut fixed: Option<String> = None;
        let mut ascii_only = false;
        let mut sources = Vec::with_capacity(parts.len());
        for part in &parts {
            let (source, standing) = match self.union_pattern_argument(part, position)? {
                Some(Object::Regex(pattern, flags)) => {
                    let named = self.pattern_encoding_name(&pattern, &flags);
                    let standing = if !encoding_reads_alongside_ascii(&named) {
                        UnionEncoding::AsciiIncompatible(named)
                    } else if self.pattern_fixes_encoding(&pattern, &flags) {
                        UnionEncoding::Fixed(named)
                    } else {
                        UnionEncoding::AsciiOnly
                    };
                    let written = self.send_to_object(
                        Object::Regex(pattern, flags),
                        "to_s",
                        vec![],
                        position,
                    )?;
                    (written.to_string(), standing)
                }
                _ => {
                    let text = self.union_string_argument(part, position)?;
                    let named = text.encoding_name();
                    let standing = if !encoding_reads_alongside_ascii(&named) {
                        UnionEncoding::AsciiIncompatible(named)
                    } else if crate::vm::native_methods::string_methods::binary_bytes(&text)
                        .is_ascii()
                    {
                        UnionEncoding::AsciiOnly
                    } else {
                        UnionEncoding::Fixed(named)
                    };
                    (crate::regexp::escape(&text.as_str()), standing)
                }
            };
            match standing {
                UnionEncoding::AsciiIncompatible(named) => match &incompatible {
                    None => incompatible = Some(named),
                    Some(held) if *held != named => {
                        return Err(incompatible_union(held, &named, position));
                    }
                    Some(_) => {}
                },
                UnionEncoding::Fixed(named) => match &fixed {
                    None => fixed = Some(named),
                    Some(held) if *held != named => {
                        return Err(incompatible_union(held, &named, position));
                    }
                    Some(_) => {}
                },
                UnionEncoding::AsciiOnly => ascii_only = true,
            }
            if let Some(held) = &incompatible {
                if ascii_only {
                    let message = format!("ASCII incompatible encoding: {held}");
                    return Err(crate::vm::errors::simple_exception(
                        "ArgumentError",
                        &message,
                        position,
                    ));
                }
                if let Some(other) = &fixed {
                    return Err(incompatible_union(held, other, position));
                }
            }
            sources.push(source);
        }
        let built = Rc::new(sources.join("|"));
        // The union is built from its source tagged with the encoding that
        // settled, so a source that stays within ASCII reads as US-ASCII
        // unless that encoding spells ASCII another way.
        let settled = incompatible.or(fixed.filter(|_| pattern_reaches_beyond_ascii(&built)));
        let mut flags = String::new();
        if let Some(named) = settled {
            flags.push('u');
            self.record_pattern_encoding(&built, named);
        }
        self.record_built_pattern(&built);
        Ok(Object::Regex(built, Rc::new(flags)))
    }

    /// A pattern given to `Regexp.union` as a Regexp, or as an object that
    /// converts to one through `to_regexp`. Anything else is None.
    fn union_pattern_argument(
        &mut self,
        part: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match part {
            Object::Regex(..) => Ok(Some(part.clone())),
            Object::String(_) | Object::Symbol(_) => Ok(None),
            other if self.responds_to(other, "to_regexp") => {
                match self.send_to_object(other.clone(), "to_regexp", vec![], position)? {
                    converted @ Object::Regex(..) => Ok(Some(converted)),
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    /// A pattern given to `Regexp.union` as text: a String, a Symbol's name,
    /// or what `to_str` answers.
    fn union_string_argument(
        &mut self,
        part: &Object,
        position: Position,
    ) -> Result<Rc<crate::object::StringValue>, MetorexError> {
        match part {
            Object::String(text) | Object::Symbol(text) => Ok(Rc::clone(text)),
            other if self.responds_to(other, "to_str") => {
                match self.send_to_object(other.clone(), "to_str", vec![], position)? {
                    Object::String(text) => Ok(text),
                    _ => Err(self.string_conversion_error(other, position)),
                }
            }
            other => Err(self.string_conversion_error(other, position)),
        }
    }
}

/// The error `Regexp.union` raises for two patterns whose encodings cannot be
/// read together.
fn incompatible_union(first: &str, second: &str, position: Position) -> MetorexError {
    let message = format!("incompatible encodings: {first} and {second}");
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
}
