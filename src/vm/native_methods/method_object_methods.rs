use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::parser::{ANONYMOUS_BLOCK, ANONYMOUS_KWREST, ANONYMOUS_SPLAT};
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn call_method_object_methods(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Object::Method(method_obj) = receiver {
            match method_name {
                "bind" => {
                    let target = arguments.first().cloned().unwrap_or(Object::Nil);
                    let bound = method_obj.bind(target);
                    return Ok(Some(Object::Method(Rc::new(bound))));
                }
                // `bind_call(receiver, *args)` binds and calls in one step,
                // without an intermediate Method object.
                "bind_call" => {
                    if arguments.is_empty() {
                        return Err(crate::vm::errors::argument_count_error(
                            crate::vm::errors::Arity::AtLeast(1),
                            0,
                            position,
                        ));
                    }
                    let target = arguments[0].clone();
                    let bound = method_obj.bind(target.clone());
                    let owner = match &bound.owner_class {
                        Some(owner) => Rc::clone(owner),
                        None => self.builtins().class_of(&target),
                    };
                    let result = self.invoke_method(
                        owner,
                        Rc::new(bound),
                        target,
                        arguments[1..].to_vec(),
                        position,
                    )?;
                    return Ok(Some(result));
                }
                "call" | "[]" | "===" => {
                    let bound = method_obj
                        .receiver
                        .as_ref()
                        .map(|b| (**b).clone())
                        .unwrap_or(Object::Nil);
                    // Module-private mixin hooks (`append_features` and
                    // friends) can't be invoked on a Class receiver; Ruby
                    // raises TypeError when the bound target is a Class.
                    if matches!(
                        method_obj.name.as_str(),
                        "append_features" | "prepend_features" | "extend_object"
                    ) && matches!(&bound, Object::Class(_))
                    {
                        let msg = format!(
                            "bind argument must be an instance of Module: {}",
                            method_obj.name
                        );
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let owner = match &method_obj.owner_class {
                        Some(owner) => Rc::clone(owner),
                        None => self.builtins().class_of(&bound),
                    };
                    let result = self.invoke_method(
                        owner,
                        Rc::clone(method_obj),
                        bound,
                        arguments.to_vec(),
                        position,
                    )?;
                    return Ok(Some(result));
                }
                // A method names itself with a Symbol, and `original_name`
                // gives the name it was cut from when it was aliased.
                "name" => {
                    return Ok(Some(Object::symbol(method_obj.name.clone())));
                }
                "original_name" => {
                    let original = method_obj
                        .original_name
                        .clone()
                        .unwrap_or_else(|| method_obj.name.clone());
                    return Ok(Some(Object::symbol(original)));
                }
                "receiver" => {
                    return Ok(Some(
                        method_obj
                            .receiver
                            .as_ref()
                            .map(|bound| (**bound).clone())
                            .unwrap_or(Object::Nil),
                    ));
                }
                // Ruby renders a Method as the receiver's class, the module
                // the method came from when that differs, the name, the
                // parameters, and where it was written.
                "inspect" | "to_s" => {
                    let owner = method_obj
                        .owner_class
                        .as_ref()
                        .map(|owner| owner.ruby_name().to_string())
                        .or_else(|| method_obj.owner.clone())
                        .unwrap_or_default();
                    let shape = method_parameter_shape(method_obj);
                    let where_written = self.method_source_label(method_obj);
                    let rendered = match &method_obj.receiver {
                        None => {
                            format!(
                                "#<UnboundMethod: {}#{}{}{}>",
                                owner, method_obj.name, shape, where_written
                            )
                        }
                        Some(bound) => {
                            let from = self.builtins().class_of(bound).ruby_name().to_string();
                            let defining = if from == owner || owner.is_empty() {
                                String::new()
                            } else {
                                format!("({})", owner)
                            };
                            format!(
                                "#<Method: {}{}#{}{}{}>",
                                from, defining, method_obj.name, shape, where_written
                            )
                        }
                    };
                    return Ok(Some(Object::string(rendered)));
                }
                // The method `super` in this one would reach: the next
                // definition of the same name past the module this one was
                // found in.
                "super_method" => {
                    return Ok(Some(self.method_defined_above(method_obj)));
                }
                "unbind" => {
                    let mut unbound = (**method_obj).clone();
                    unbound.receiver = None;
                    return Ok(Some(Object::Method(Rc::new(unbound))));
                }
                // `Method#to_proc` stays attached to the receiver it was
                // extracted from, so the Proc keeps calling against that
                // object even after `define_method` installs it elsewhere.
                "to_proc" => {
                    let mut as_proc = (**method_obj).clone();
                    as_proc.bound_self = method_obj.receiver.clone();
                    return Ok(Some(Object::Method(Rc::new(as_proc))));
                }
                "owner" => {
                    if let Some(owner) = &method_obj.owner_class {
                        let owner = Rc::clone(owner);
                        return Ok(Some(if owner.is_module() {
                            Object::Module(owner)
                        } else {
                            Object::Class(owner)
                        }));
                    }
                    let owner_name = method_obj.owner.as_deref().unwrap_or("main");
                    // A native stub records its owner by name; answer the
                    // module itself when that name resolves to one.
                    if let Some(owner @ (Object::Class(_) | Object::Module(_))) =
                        self.globals().get(owner_name)
                    {
                        return Ok(Some(owner));
                    }
                    return Ok(Some(Object::string(owner_name.to_string())));
                }
                // Ruby answers `[path, lineno]`, or nil for a method with no
                // Ruby source behind it.
                "source_location" => {
                    let Some(location) = &method_obj.source_location else {
                        return Ok(Some(Object::Nil));
                    };
                    let path = location
                        .filename
                        .clone()
                        .or_else(|| {
                            self.current_file
                                .as_ref()
                                .map(|file| file.display().to_string())
                        })
                        .unwrap_or_default();
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(vec![
                        Object::string(path),
                        Object::Int(location.line as i64),
                    ])))));
                }
                "parameters" => {
                    return Ok(Some(method_parameter_list(method_obj)));
                }
                "body" => {
                    return Ok(Some(super::ast_methods::serialize_statements(
                        &method_obj.body,
                    )));
                }
                "arity" => {
                    return Ok(Some(Object::Int(method_arity(method_obj))));
                }
                _ => {}
            }
        }

        if let Object::Block(block_obj) = receiver {
            match method_name {
                "statements" => {
                    return Ok(Some(super::ast_methods::serialize_statements(
                        &block_obj.body,
                    )));
                }
                "arity" => {
                    return Ok(Some(Object::Int(block_arity(block_obj))));
                }
                "parameters" => {
                    return Ok(Some(block_parameter_list(block_obj)));
                }
                "lambda?" => {
                    return Ok(Some(Object::Bool(block_obj.is_lambda)));
                }
                _ => {}
            }
        }

        Ok(None)
    }
}

/// The name a parameter reports, where the placeholder metorex gives an
/// unnamed splat is not one Ruby names at all.
fn declared_parameter_name(name: &str) -> Option<Object> {
    if name == ANONYMOUS_SPLAT || name == ANONYMOUS_KWREST || name == ANONYMOUS_BLOCK {
        return None;
    }
    Some(Object::symbol(name.to_string()))
}

/// One `[kind, name]` pair of a parameter list, where an unnamed parameter
/// reports its kind alone.
fn parameter_pair(kind: &str, name: &str) -> Object {
    let mut pair = vec![Object::symbol(kind.to_string())];
    if let Some(named) = declared_parameter_name(name) {
        pair.push(named);
    }
    Object::array(pair)
}

/// The parameter list Ruby reports: positionals in the order they were
/// declared, then the keywords, the keyword rest, and the block.
fn method_parameter_list(method_obj: &crate::object::Method) -> Object {
    let splat_index = method_obj.variadic_param.as_ref().map(|(index, _)| *index);
    let mut listed = Vec::new();
    for (index, name) in method_obj.parameters.iter().enumerate() {
        let kind = if Some(index) == splat_index {
            "rest"
        } else if method_obj
            .default_parameters
            .iter()
            .any(|(defaulted, _)| *defaulted == index)
        {
            "opt"
        } else {
            "req"
        };
        listed.push(parameter_pair(kind, name));
    }
    for (name, default) in &method_obj.keyword_parameters {
        let kind = if default.is_some() { "key" } else { "keyreq" };
        listed.push(parameter_pair(kind, name));
    }
    if let Some(name) = &method_obj.keyword_rest_parameter {
        listed.push(parameter_pair("keyrest", name));
    }
    if let Some(name) = &method_obj.block_parameter {
        listed.push(parameter_pair("block", name));
    }
    Object::array(listed)
}

/// Ruby's arity: the count of required parameters, made negative when a call
/// may pass more or fewer than that.
fn method_arity(method_obj: &crate::object::Method) -> i64 {
    let splat_index = method_obj.variadic_param.as_ref().map(|(index, _)| *index);
    let mut required = 0i64;
    let mut optional_positional = false;
    for index in 0..method_obj.parameters.len() {
        if Some(index) == splat_index {
            continue;
        }
        if method_obj
            .default_parameters
            .iter()
            .any(|(defaulted, _)| *defaulted == index)
        {
            optional_positional = true;
        } else {
            required += 1;
        }
    }
    let required_keyword = method_obj
        .keyword_parameters
        .iter()
        .any(|(_, default)| default.is_none());
    if required_keyword {
        required += 1;
    }
    let optional_keyword = method_obj
        .keyword_parameters
        .iter()
        .any(|(_, default)| default.is_some())
        || method_obj.keyword_rest_parameter.is_some();
    let open_ended =
        optional_positional || splat_index.is_some() || (!required_keyword && optional_keyword);
    if open_ended {
        -(required + 1)
    } else {
        required
    }
}

/// What one block parameter stands for, read off the prefix its name carries.
enum BlockParameterKind {
    Positional,
    Rest,
    Keyword,
    KeywordRest,
    Block,
}

/// The kind and declared name of one block parameter, or None for the marker
/// a trailing comma leaves behind.
fn block_parameter_parts(name: &str) -> Option<(BlockParameterKind, String)> {
    if name == crate::object::TRAILING_COMMA_PARAM {
        return None;
    }
    if let Some(rest) = name.strip_prefix("**") {
        return Some((BlockParameterKind::KeywordRest, rest.to_string()));
    }
    if let Some(rest) = name.strip_prefix('*') {
        return Some((BlockParameterKind::Rest, rest.to_string()));
    }
    if let Some(rest) = name.strip_prefix('&') {
        return Some((BlockParameterKind::Block, rest.to_string()));
    }
    if let Some(rest) = name.strip_prefix(crate::object::KEYWORD_PARAM_PREFIX) {
        return Some((BlockParameterKind::Keyword, rest.to_string()));
    }
    if name.starts_with(crate::object::DESTRUCTURED_GROUP_PREFIX) {
        return Some((BlockParameterKind::Positional, String::new()));
    }
    Some((BlockParameterKind::Positional, name.to_string()))
}

/// The parameter list a Proc reports. A proc that is not a lambda takes what
/// it is handed, so it reports its positional parameters as optional.
fn block_parameter_list(block_obj: &crate::object::BlockStatement) -> Object {
    let required_kind = if block_obj.is_lambda { "req" } else { "opt" };
    let mut listed = Vec::new();
    for (index, name) in block_obj.parameters.iter().enumerate() {
        let Some((kind, declared)) = block_parameter_parts(name) else {
            continue;
        };
        let defaulted = block_obj
            .parameter_defaults
            .iter()
            .any(|(position, _)| *position == index);
        let label = match kind {
            BlockParameterKind::Positional if defaulted => "opt",
            BlockParameterKind::Positional => required_kind,
            BlockParameterKind::Rest => "rest",
            BlockParameterKind::Keyword if defaulted => "key",
            BlockParameterKind::Keyword => "keyreq",
            BlockParameterKind::KeywordRest => "keyrest",
            BlockParameterKind::Block => "block",
        };
        let mut pair = vec![Object::symbol(label.to_string())];
        if !declared.is_empty() {
            pair.push(Object::symbol(declared));
        }
        listed.push(Object::array(pair));
    }
    Object::array(listed)
}

/// A Proc's arity, counted the same way a method's is.
fn block_arity(block_obj: &crate::object::BlockStatement) -> i64 {
    let mut required = 0i64;
    let mut optional_positional = false;
    let mut splat = false;
    let mut required_keyword = false;
    let mut optional_keyword = false;
    for (index, name) in block_obj.parameters.iter().enumerate() {
        let Some((kind, _)) = block_parameter_parts(name) else {
            continue;
        };
        let defaulted = block_obj
            .parameter_defaults
            .iter()
            .any(|(position, _)| *position == index);
        match kind {
            BlockParameterKind::Positional if defaulted => optional_positional = true,
            BlockParameterKind::Positional => required += 1,
            BlockParameterKind::Rest => splat = true,
            BlockParameterKind::Keyword if defaulted => optional_keyword = true,
            BlockParameterKind::Keyword => required_keyword = true,
            BlockParameterKind::KeywordRest => optional_keyword = true,
            BlockParameterKind::Block => {}
        }
    }
    if required_keyword {
        required += 1;
    }
    // A proc that is not a lambda takes what it is handed, so only a splat
    // leaves the count open. A lambda is strict the way a method is.
    let open_ended = if block_obj.is_lambda {
        optional_positional || splat || (!required_keyword && optional_keyword)
    } else {
        splat
    };
    if open_ended {
        -(required + 1)
    } else {
        required
    }
}

/// How a method's parameters are written inside its rendering, which shows
/// each one's shape rather than any default it carries.
fn method_parameter_shape(method_obj: &crate::object::Method) -> String {
    let splat_index = method_obj.variadic_param.as_ref().map(|(index, _)| *index);
    let mut parts = Vec::new();
    for (index, name) in method_obj.parameters.iter().enumerate() {
        let shown = if name == ANONYMOUS_SPLAT {
            ""
        } else {
            name.as_str()
        };
        if Some(index) == splat_index {
            parts.push(format!("*{}", shown));
        } else if method_obj
            .default_parameters
            .iter()
            .any(|(defaulted, _)| *defaulted == index)
        {
            parts.push(format!("{}=...", shown));
        } else {
            parts.push(shown.to_string());
        }
    }
    for (name, default) in &method_obj.keyword_parameters {
        if default.is_some() {
            parts.push(format!("{}: ...", name));
        } else {
            parts.push(format!("{}:", name));
        }
    }
    if let Some(name) = &method_obj.keyword_rest_parameter {
        let shown = if name == ANONYMOUS_KWREST {
            ""
        } else {
            name.as_str()
        };
        parts.push(format!("**{}", shown));
    }
    if let Some(name) = &method_obj.block_parameter {
        let shown = if name == ANONYMOUS_BLOCK {
            ""
        } else {
            name.as_str()
        };
        parts.push(format!("&{}", shown));
    }
    format!("({})", parts.join(", "))
}

impl VirtualMachine {
    /// The file and line a method was written at, as its rendering shows it.
    /// A method with no recorded place reports none.
    fn method_source_label(&mut self, method_obj: &crate::object::Method) -> String {
        let Some(location) = &method_obj.source_location else {
            return String::new();
        };
        let path = location
            .filename
            .clone()
            .or_else(|| {
                self.current_file
                    .as_ref()
                    .map(|file| file.display().to_string())
            })
            .unwrap_or_default();
        format!(" {}:{}", path, location.line)
    }
}

impl VirtualMachine {
    /// The next definition of a method's name above the module it was found
    /// in, which is what `super` in its body would reach. A name that was
    /// undefined further up, or one with nothing above it, answers nil.
    fn method_defined_above(&mut self, method_obj: &crate::object::Method) -> Object {
        let Some(owner) = method_obj.owner_class.as_ref().map(Rc::clone) else {
            return Object::Nil;
        };
        let start = match &method_obj.receiver {
            Some(bound) => self.builtins().class_of(bound),
            None => method_obj
                .origin_class
                .as_ref()
                .map(Rc::clone)
                .unwrap_or_else(|| Rc::clone(&owner)),
        };
        let mut chain: Vec<Object> = Vec::new();
        let mut seen: Vec<*const crate::class::Class> = Vec::new();
        super::class_methods::push_class_ancestors(&start, &mut chain, &mut seen);
        // A method reached through a module extended onto the object alone
        // has an owner the class chain does not hold, so the walk starts at
        // the top of that chain.
        let owner_in_chain = chain.iter().any(|ancestor| {
            matches!(ancestor, Object::Class(current) | Object::Module(current)
                if Rc::ptr_eq(current, &owner))
        });
        // An alias carries the body of whatever the name it was cut from
        // resolved to, so the walk passes that definition as well.
        let lookup = method_obj
            .original_name
            .clone()
            .unwrap_or_else(|| method_obj.name.clone());
        let mut past_source = lookup == method_obj.name;
        let mut past_owner = !owner_in_chain;
        for ancestor in &chain {
            let (Object::Class(current) | Object::Module(current)) = ancestor else {
                continue;
            };
            if !past_owner {
                past_owner = Rc::ptr_eq(current, &owner);
                continue;
            }
            let Some(found) = current.find_own_method(&lookup) else {
                continue;
            };
            if !past_source {
                past_source = true;
                continue;
            }
            if found.is_undefined {
                return Object::Nil;
            }
            let mut above = (*found).clone();
            above.name = method_obj.name.clone();
            above.receiver = method_obj.receiver.clone();
            above.owner_class = Some(Rc::clone(current));
            above.owner = Some(current.ruby_name().to_string());
            return Object::Method(Rc::new(above));
        }
        Object::Nil
    }
}
