//! Native method implementations for the Object class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Execute native methods for the Object class.
    pub(crate) fn call_object_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Nil-specific conversions: in Ruby `nil.to_i == 0`, `nil.to_s == ""`,
        // `nil.to_a == []`, `nil.to_f == 0.0`. The dispatch above checks the
        // class of the receiver first, so we have to intercept here for Nil
        // before falling through to the generic Object methods.
        if matches!(receiver, Object::Nil) {
            match method_name {
                "to_i" => return Ok(Some(Object::Int(0))),
                "to_f" => return Ok(Some(Object::Float(0.0))),
                "to_a" => {
                    return Ok(Some(Object::Array(std::rc::Rc::new(
                        std::cell::RefCell::new(Vec::new()),
                    ))));
                }
                "to_h" => {
                    return Ok(Some(Object::Dict(std::rc::Rc::new(
                        std::cell::RefCell::new(std::collections::HashMap::new()),
                    ))));
                }
                "to_s" => return Ok(Some(Object::string(""))),
                "inspect" => return Ok(Some(Object::string("nil"))),
                _ => {}
            }
        }
        match method_name {
            "to_s" | "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::string(receiver.to_string())))
            }
            "class" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Class(self.builtins().class_of(receiver))))
            }
            "respond_to?" => {
                // Accept String or Symbol method name; ignore optional second
                // `include_private` argument (Ruby's signature).
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method_query = match &arguments[0] {
                    Object::String(name) => name.as_str().to_string(),
                    Object::Symbol(name) => name.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                Ok(Some(Object::Bool(
                    self.lookup_method(receiver, &method_query).is_some(),
                )))
            }
            "nil?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(matches!(receiver, Object::Nil))))
            }
            "get_source" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let query = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                match self.lookup_method(receiver, &query) {
                    Some((_class, method)) => Ok(Some(Object::Method(method))),
                    None => Ok(Some(Object::Nil)),
                }
            }
            "is_a?" | "kind_of?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_class = match &arguments[0] {
                    Object::Class(c) => c,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Class",
                            other,
                            position,
                        ));
                    }
                };
                Ok(Some(Object::Bool(
                    self.builtins().is_instance_of(receiver, target_class),
                )))
            }
            "instance_variables" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let vars = if let Object::Instance(inst_rc) = receiver {
                    let inst = inst_rc.borrow();
                    inst.instance_vars
                        .keys()
                        .map(|k| Object::string(format!("@{}", k)))
                        .collect()
                } else {
                    vec![]
                };
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(vars),
                ))))
            }
            "instance_variable_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                // Strip leading @ if present
                let clean_name = var_name.strip_prefix('@').unwrap_or(&var_name);
                match receiver {
                    Object::Instance(inst_rc) => {
                        let inst = inst_rc.borrow();
                        Ok(Some(
                            inst.instance_vars
                                .get(clean_name)
                                .cloned()
                                .unwrap_or(Object::Nil),
                        ))
                    }
                    Object::Class(class_rc) => Ok(Some(
                        class_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    Object::Module(module_rc) => Ok(Some(
                        module_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    _ => Ok(Some(Object::Nil)),
                }
            }
            "instance_variable_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name = match &arguments[0] {
                    Object::String(s) => s.strip_prefix('@').unwrap_or(s).to_string(),
                    Object::Symbol(s) => s.strip_prefix('@').unwrap_or(s).to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let value = arguments[1].clone();
                match receiver {
                    Object::Instance(instance_rc) => {
                        instance_rc.borrow_mut().set_var(var_name, value.clone());
                        Ok(Some(value))
                    }
                    Object::Class(class_rc) => {
                        class_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    Object::Module(module_rc) => {
                        module_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    _ => Ok(Some(Object::Nil)),
                }
            }
            "instance_of?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_class = match &arguments[0] {
                    Object::Class(c) => c,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Class",
                            other,
                            position,
                        ));
                    }
                };
                let obj_class = self.builtins().class_of(receiver);
                Ok(Some(Object::Bool(obj_class.name() == target_class.name())))
            }
            "methods" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let class = self.builtins().class_of(receiver);
                let mut names = class.method_names();
                // Walk the superclass chain to collect inherited methods
                let mut current = class.superclass();
                while let Some(parent) = current {
                    for name in parent.method_names() {
                        if !names.contains(&name) {
                            names.push(name);
                        }
                    }
                    current = parent.superclass();
                }
                // For instances, also include methods from the instance's class
                if let Object::Instance(inst_rc) = receiver {
                    let inst = inst_rc.borrow();
                    for name in inst.class.method_names() {
                        if !names.contains(&name) {
                            names.push(name);
                        }
                    }
                    let mut parent = inst.class.superclass();
                    while let Some(p) = parent {
                        for name in p.method_names() {
                            if !names.contains(&name) {
                                names.push(name);
                            }
                        }
                        parent = p.superclass();
                    }
                }
                names.sort();
                names.dedup();
                let method_strings: Vec<Object> = names.into_iter().map(Object::string).collect();
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(method_strings),
                ))))
            }
            "send" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let send_args = arguments[1..].to_vec();
                // Try user-defined method lookup
                if let Some((class, method)) = self.lookup_method(receiver, &target_method)
                    && !method.is_undefined
                {
                    return self
                        .invoke_method(class, method, receiver.clone(), send_args, position)
                        .map(Some);
                }
                // Try native method
                let class = self.builtins().class_of(receiver);
                let native_result = self.call_native_method(
                    &class,
                    receiver,
                    &target_method,
                    &send_args,
                    position,
                )?;
                if let Some(result) = native_result {
                    return Ok(Some(result));
                }
                // Try object methods
                let object_result =
                    self.call_object_method(receiver, &target_method, &send_args, position)?;
                if let Some(result) = object_result {
                    return Ok(Some(result));
                }
                Err(undefined_method_error(&target_method, receiver, position))
            }
            "equal?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = &arguments[0];
                let identity = match (receiver, other) {
                    (Object::Instance(a), Object::Instance(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Array(a), Object::Array(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Dict(a), Object::Dict(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Class(a), Object::Class(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Module(a), Object::Module(b)) => std::rc::Rc::ptr_eq(a, b),
                    // For value types, equal? is the same as ==
                    (a, b) => a == b,
                };
                Ok(Some(Object::Bool(identity)))
            }
            "dup" | "clone" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                match receiver {
                    Object::Instance(inst_rc) => {
                        let inst = inst_rc.borrow();
                        let mut new_inst =
                            crate::object::Instance::new(std::rc::Rc::clone(&inst.class));
                        for (k, v) in &inst.instance_vars {
                            new_inst.set_var(k.clone(), v.clone());
                        }
                        Ok(Some(Object::Instance(std::rc::Rc::new(
                            std::cell::RefCell::new(new_inst),
                        ))))
                    }
                    Object::Array(arr_rc) => {
                        let arr = arr_rc.borrow().clone();
                        Ok(Some(Object::Array(std::rc::Rc::new(
                            std::cell::RefCell::new(arr),
                        ))))
                    }
                    Object::Dict(dict_rc) => {
                        let dict = dict_rc.borrow().clone();
                        Ok(Some(Object::Dict(std::rc::Rc::new(
                            std::cell::RefCell::new(dict),
                        ))))
                    }
                    // Immutable types return themselves
                    _ => Ok(Some(receiver.clone())),
                }
            }
            "=~" => {
                // Regex match: string =~ regex or regex =~ string
                if arguments.len() != 1 {
                    return Err(method_argument_error("=~", 1, arguments.len(), position));
                }
                match (receiver, &arguments[0]) {
                    (Object::Regex(pattern, flags), Object::String(s))
                    | (Object::String(s), Object::Regex(pattern, flags)) => {
                        let case_insensitive = flags.contains('i');
                        let re_pattern = if case_insensitive {
                            format!("(?i){}", pattern)
                        } else {
                            pattern.as_ref().clone()
                        };
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => {
                                if let Some(m) = re.find(s.as_ref()) {
                                    Ok(Some(Object::Int(m.start() as i64)))
                                } else {
                                    Ok(Some(Object::Nil))
                                }
                            }
                            Err(_) => Ok(Some(Object::Nil)),
                        }
                    }
                    _ => Ok(Some(Object::Nil)),
                }
            }
            "!~" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error("!~", 1, arguments.len(), position));
                }
                match (receiver, &arguments[0]) {
                    (Object::Regex(pattern, flags), Object::String(s))
                    | (Object::String(s), Object::Regex(pattern, flags)) => {
                        let case_insensitive = flags.contains('i');
                        let re_pattern = if case_insensitive {
                            format!("(?i){}", pattern)
                        } else {
                            pattern.as_ref().clone()
                        };
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => Ok(Some(Object::Bool(!re.is_match(s.as_ref())))),
                            Err(_) => Ok(Some(Object::Bool(true))),
                        }
                    }
                    _ => Ok(Some(Object::Bool(true))),
                }
            }
            "instance_exec" | "instance_eval" => {
                let block = self.pending_block.take().or_else(|| {
                    if !arguments.is_empty()
                        && let Object::Block(_) = &arguments[0]
                    {
                        Some(arguments[0].clone())
                    } else {
                        None
                    }
                });
                match block {
                    Some(Object::Block(b)) => {
                        let args: Vec<Object> = arguments
                            .iter()
                            .filter(|a| !matches!(a, Object::Block(_)))
                            .cloned()
                            .collect();
                        let result =
                            self.execute_block_with_receiver(&b, receiver.clone(), args, position)?;
                        Ok(Some(result))
                    }
                    _ => Err(MetorexError::runtime_error(
                        format!("{} requires a block", method_name),
                        position_to_location(position),
                    )),
                }
            }
            _ => Ok(None),
        }
    }
}
