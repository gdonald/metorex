//! Struct: the class builder `Struct.new` and the instance methods every
//! generated struct class inherits.

use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Instance, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

/// Class variable holding the ordered member names of a generated struct class.
const MEMBERS_VAR: &str = "__struct_members__";
/// Class variable holding the `keyword_init:` value the struct was built with.
const KEYWORD_INIT_VAR: &str = "__struct_keyword_init__";
/// Member values live under a prefixed instance-variable name, since a struct
/// member is not an instance variable: `Struct.new(:a).new(1).instance_variables`
/// is empty, and `@a` stays free for the holder to use.
const MEMBER_PREFIX: &str = "__struct_member_";

/// The instance-variable slot a member's value is stored in.
pub(crate) fn member_slot(member: &str) -> String {
    format!("{}{}", MEMBER_PREFIX, member)
}

/// Whether an instance-variable name is a struct member slot rather than one
/// the holder set.
pub(crate) fn is_member_slot(name: &str) -> bool {
    name.starts_with(MEMBER_PREFIX)
}

/// The member names of a generated struct class, or None when `class_rc` is
/// not one. `Struct` itself has no members and answers None.
pub(crate) fn struct_members(class_rc: &Rc<Class>) -> Option<Vec<String>> {
    match class_rc.lookup_class_var(MEMBERS_VAR) {
        Some(Object::Array(names)) => Some(
            names
                .borrow()
                .iter()
                .map(|name| match name {
                    Object::Symbol(s) => (**s).clone(),
                    other => other.to_string(),
                })
                .collect(),
        ),
        _ => None,
    }
}

fn keyword_init(class_rc: &Rc<Class>) -> Object {
    class_rc
        .lookup_class_var(KEYWORD_INIT_VAR)
        .unwrap_or(Object::Nil)
}

fn symbols(names: &[String]) -> Object {
    Object::Array(Rc::new(RefCell::new(
        names
            .iter()
            .map(|name| Object::Symbol(Rc::new(name.clone())))
            .collect(),
    )))
}

fn argument_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("ArgumentError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

/// One `values_at` subscript, counting from the end when negative.
fn indexed_value(
    values: &[Object],
    index: i64,
    position: Position,
) -> Result<Object, MetorexError> {
    let length = values.len() as i64;
    let resolved = if index < 0 { index + length } else { index };
    if resolved < 0 {
        return Err(index_error(
            format!("offset {} too small for struct(size:{})", index, length),
            position,
        ));
    }
    if resolved >= length {
        return Err(index_error(
            format!("offset {} too large for struct(size:{})", index, length),
            position,
        ));
    }
    Ok(values[resolved as usize].clone())
}

/// A `values_at` Range subscript. Elements past the end read as nil, while a
/// negative start that falls off the front is a RangeError.
fn range_values(
    values: &[Object],
    range: &Object,
    position: Position,
) -> Result<Vec<Object>, MetorexError> {
    let Object::Range {
        start,
        end,
        exclusive,
    } = range
    else {
        return Ok(Vec::new());
    };
    let length = values.len() as i64;
    let first = match start.as_ref() {
        Object::Nil => 0,
        Object::Int(value) if *value < 0 => value + length,
        Object::Int(value) => *value,
        _ => 0,
    };
    if first < 0 || first > length {
        return Err(MetorexError::UncaughtException {
            exception: Object::exception("RangeError", format!("{} out of range", range)),
            location: position_to_location(position),
            message: format!("{} out of range", range),
        });
    }
    let mut last = match end.as_ref() {
        Object::Nil => length - 1,
        Object::Int(value) if *value < 0 => value + length,
        Object::Int(value) => {
            if *exclusive {
                value - 1
            } else {
                *value
            }
        }
        _ => length - 1,
    };
    if matches!(end.as_ref(), Object::Int(value) if *value < 0) && *exclusive {
        last -= 1;
    }
    let mut picked = Vec::new();
    for index in first..=last.max(first - 1) {
        picked.push(values.get(index as usize).cloned().unwrap_or(Object::Nil));
    }
    Ok(picked)
}

/// A hash key that is not a primitive is recorded in the sentinel sub-map, so
/// the original object comes back when the hash is walked.
fn remember_key_object(pairs: &mut IndexMap<String, Object>, rendered: &str, key: &Object) {
    let sentinel = "__MX_KEY_OBJECTS__".to_string();
    let mut objects = match pairs.get(&sentinel) {
        Some(Object::Dict(existing)) => existing.borrow().clone(),
        _ => IndexMap::new(),
    };
    objects.insert(rendered.to_string(), key.clone());
    pairs.insert(sentinel, Object::Dict(Rc::new(RefCell::new(objects))));
}

fn index_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("IndexError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

fn name_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("NameError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

/// Pull the parser-marked keyword-argument hash off the end of an argument
/// list, leaving the positional arguments behind.
fn take_keyword_arguments(arguments: &[Object]) -> (Vec<Object>, IndexMap<String, Object>) {
    if let Some(Object::Dict(dict_rc)) = arguments.last() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            let keywords = dict
                .iter()
                .filter(|(key, _)| key.as_str() != "__MX_KWARGS__")
                .map(|(key, value)| {
                    (
                        key.strip_prefix(':').unwrap_or(key).to_string(),
                        value.clone(),
                    )
                })
                .collect();
            return (arguments[..arguments.len() - 1].to_vec(), keywords);
        }
    }
    (arguments.to_vec(), IndexMap::new())
}

/// The value stored for `member` on a struct instance.
fn member_value(receiver: &Object, member: &str) -> Object {
    match receiver {
        Object::Instance(instance) => instance
            .borrow()
            .instance_vars
            .get(&member_slot(member))
            .cloned()
            .unwrap_or(Object::Nil),
        _ => Object::Nil,
    }
}

fn member_values(receiver: &Object, members: &[String]) -> Vec<Object> {
    members
        .iter()
        .map(|member| member_value(receiver, member))
        .collect()
}

/// Resolve `[]` / `[]=` / `dig` subscripts, which accept a member name or a
/// positional index counting from either end.
fn resolve_member(
    members: &[String],
    key: &Object,
    class_rc: &Rc<Class>,
    position: Position,
) -> Result<String, MetorexError> {
    match key {
        Object::Int(index) => {
            let length = members.len() as i64;
            let resolved = if *index < 0 { index + length } else { *index };
            if resolved < 0 || resolved >= length {
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception(
                        "IndexError",
                        format!("offset {} too large for struct(size:{})", index, length),
                    ),
                    location: position_to_location(position),
                    message: format!("offset {} too large for struct(size:{})", index, length),
                });
            }
            Ok(members[resolved as usize].clone())
        }
        Object::Symbol(name) => resolve_named_member(members, name, class_rc, position),
        Object::String(name) => resolve_named_member(members, name, class_rc, position),
        other => Err(MetorexError::type_error(
            format!(
                "no implicit conversion of {} into Integer",
                other.type_name()
            ),
            position_to_location(position),
        )),
    }
}

fn resolve_named_member(
    members: &[String],
    name: &str,
    class_rc: &Rc<Class>,
    position: Position,
) -> Result<String, MetorexError> {
    if members.iter().any(|member| member == name) {
        Ok(name.to_string())
    } else {
        Err(name_error(
            format!("no member '{}' in struct {}", name, class_rc.ruby_name()),
            position,
        ))
    }
}

impl VirtualMachine {
    /// `Struct.new(...)` on Struct itself, plus the class-level methods a
    /// generated struct class answers.
    pub(crate) fn call_struct_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if class_rc.name() == "Struct" && method_name == "new" {
            return self.build_struct_class(arguments, position).map(Some);
        }

        let Some(members) = struct_members(class_rc) else {
            return Ok(None);
        };

        match method_name {
            "new" | "[]" => self
                .build_struct_instance(class_rc, &members, arguments, position)
                .map(Some),
            "members" => Ok(Some(symbols(&members))),
            // `keyword_init?` reports a truthy setting as `true` and keeps
            // `nil` for a struct that never named one.
            "keyword_init?" => Ok(Some(match keyword_init(class_rc) {
                Object::Nil => Object::Nil,
                other => Object::Bool(other.is_truthy()),
            })),
            _ => Ok(None),
        }
    }

    /// Create the anonymous class `Struct.new` returns, with a reader and a
    /// writer per member and the block (if any) run as its class body.
    fn build_struct_class(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (positional, keywords) = take_keyword_arguments(arguments);
        if positional.is_empty() {
            return Err(argument_error(
                "wrong number of arguments (given 0, expected 1+)".to_string(),
                position,
            ));
        }

        // A leading String naming a constant registers the class under
        // `Struct::Name` rather than contributing a member.
        let mut index = 0;
        let mut constant_name = None;
        if let Object::String(first) = &positional[0]
            && first.chars().next().is_some_and(|c| c.is_uppercase())
        {
            constant_name = Some((**first).clone());
            index = 1;
        }

        let mut members = Vec::new();
        for argument in &positional[index..] {
            match argument {
                Object::Symbol(name) => members.push((**name).clone()),
                Object::String(name) => members.push((**name).clone()),
                other => {
                    return Err(MetorexError::type_error(
                        format!("{} is not a symbol nor a string", other),
                        position_to_location(position),
                    ));
                }
            }
        }

        let Some(Object::Class(struct_class)) = self.globals().get("Struct") else {
            return Err(MetorexError::runtime_error(
                "Struct is not defined",
                position_to_location(position),
            ));
        };

        let generated = Rc::new(Class::new("", Some(Rc::clone(&struct_class))));
        struct_class.add_subclass(&generated);
        generated.set_class_var(MEMBERS_VAR, symbols(&members));
        generated.set_class_var(
            KEYWORD_INIT_VAR,
            keywords.get("keyword_init").cloned().unwrap_or(Object::Nil),
        );

        for member in &members {
            let reader_body = vec![crate::ast::Statement::Return {
                value: Some(crate::ast::Expression::InstanceVariable {
                    name: member_slot(member),
                    position,
                }),
                position,
            }];
            generated.define_method(
                member,
                Rc::new(crate::object::Method::new(
                    member.clone(),
                    vec![],
                    reader_body,
                )),
            );

            let writer_name = format!("{}=", member);
            let writer_body = vec![crate::ast::Statement::Assignment {
                target: crate::ast::Expression::InstanceVariable {
                    name: member_slot(member),
                    position,
                },
                value: crate::ast::Expression::Identifier {
                    name: "value".to_string(),
                    position,
                },
                position,
            }];
            generated.define_method(
                &writer_name,
                Rc::new(crate::object::Method::new(
                    writer_name.clone(),
                    vec!["value".to_string()],
                    writer_body,
                )),
            );
            generated.declare_instance_var(member_slot(member));
        }

        if let Some(name) = constant_name {
            generated.assign_name_recursive(&format!("Struct::{}", name));
            struct_class.set_class_var(&name, Object::Class(Rc::clone(&generated)));
            self.globals_mut().set(
                format!("Struct::{}", name),
                Object::Class(Rc::clone(&generated)),
            );
        }

        if let Some(Object::Block(block)) = self.pending_block.take() {
            self.apply_block_as_class_body(&generated, &block, position)?;
        }

        Ok(Object::Class(generated))
    }

    /// `Point.new(1, 2)` / `Point.new(x: 1, y: 2)` for a generated struct class.
    fn build_struct_instance(
        &mut self,
        class_rc: &Rc<Class>,
        members: &[String],
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (positional, keywords) = take_keyword_arguments(arguments);
        let by_keyword =
            keyword_init(class_rc).is_truthy() || (positional.is_empty() && !keywords.is_empty());

        let mut instance = Instance::new(Rc::clone(class_rc));

        if by_keyword {
            for (name, value) in &keywords {
                if !members.iter().any(|member| member == name) {
                    return Err(argument_error(
                        format!("unknown keywords: :{}", name),
                        position,
                    ));
                }
                instance
                    .instance_vars
                    .insert(member_slot(name), value.clone());
            }
        } else {
            if positional.len() > members.len() {
                return Err(argument_error("struct size differs".to_string(), position));
            }
            for (member, value) in members.iter().zip(positional.iter()) {
                instance
                    .instance_vars
                    .insert(member_slot(member), value.clone());
            }
        }

        for member in members {
            instance
                .instance_vars
                .entry(member_slot(member))
                .or_insert(Object::Nil);
        }

        Ok(Object::Instance(Rc::new(RefCell::new(instance))))
    }

    /// The instance methods every generated struct class inherits from Struct.
    pub(crate) fn call_struct_instance_method(
        &mut self,
        class_rc: &Rc<Class>,
        members: &[String],
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // A method the struct class or one of its mixins defines wins over
        // Struct's own, which is how `include`ing a module that defines
        // `hash` replaces it.
        if let Some((_, method)) = self.lookup_method(receiver, method_name)
            && !method.body.is_empty()
            && !members.iter().any(|member| member == method_name)
            && !self.enumerable_stands_in(receiver, method_name)
        {
            return Ok(None);
        }

        // A member accessor wins over Struct's own method of the same name,
        // so `Struct.new(:length).new(42).length` answers 42, not 1.
        if arguments.is_empty() && members.iter().any(|member| member == method_name) {
            return Ok(Some(member_value(receiver, method_name)));
        }
        if arguments.len() == 1
            && let Some(target) = method_name.strip_suffix('=')
            && members.iter().any(|member| member == target)
            && let Object::Instance(instance) = receiver
        {
            if self.object_is_frozen(receiver) {
                return Err(self.frozen_modification_error(receiver, position));
            }
            instance
                .borrow_mut()
                .instance_vars
                .insert(member_slot(target), arguments[0].clone());
            return Ok(Some(arguments[0].clone()));
        }

        match method_name {
            "members" => Ok(Some(symbols(members))),
            "size" | "length" => Ok(Some(Object::Int(members.len() as i64))),
            "to_a" | "values" | "deconstruct" => Ok(Some(Object::Array(Rc::new(RefCell::new(
                member_values(receiver, members),
            ))))),
            "to_h" => {
                let block = match self.pending_block.take() {
                    Some(Object::Block(block)) => Some(block),
                    _ => None,
                };
                let mut pairs = IndexMap::new();
                for member in members {
                    let value = member_value(receiver, member);
                    let (key, value) = match &block {
                        None => (Object::Symbol(Rc::new(member.clone())), value),
                        Some(block) => {
                            let produced = self.execute_block_callable(
                                block,
                                vec![Object::Symbol(Rc::new(member.clone())), value],
                                position,
                            )?;
                            self.pair_from_block_result(produced, position)?
                        }
                    };
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut pairs, &rendered, &key);
                    }
                    pairs.insert(rendered, value);
                }
                Ok(Some(Object::Dict(Rc::new(RefCell::new(pairs)))))
            }
            "deconstruct_keys" => {
                if arguments.len() != 1 {
                    return Err(argument_error(
                        format!(
                            "wrong number of arguments (given {}, expected 1)",
                            arguments.len()
                        ),
                        position,
                    ));
                }
                self.deconstruct_struct_keys(members, receiver, &arguments[0], position)
                    .map(Some)
            }
            "[]" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let member = resolve_member(members, &arguments[0], class_rc, position)?;
                Ok(Some(member_value(receiver, &member)))
            }
            "[]=" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let member = resolve_member(members, &arguments[0], class_rc, position)?;
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                if let Object::Instance(instance) = receiver {
                    instance
                        .borrow_mut()
                        .instance_vars
                        .insert(member_slot(&member), arguments[1].clone());
                }
                Ok(Some(arguments[1].clone()))
            }
            "dig" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let member = match resolve_member(members, &arguments[0], class_rc, position) {
                    Ok(member) => member,
                    Err(_) => return Ok(Some(Object::Nil)),
                };
                let value = member_value(receiver, &member);
                if arguments.len() == 1 || matches!(value, Object::Nil) {
                    return Ok(Some(if arguments.len() == 1 {
                        value
                    } else {
                        Object::Nil
                    }));
                }
                self.dig_into(&value, &arguments[1..], position).map(Some)
            }
            "values_at" => {
                let values = member_values(receiver, members);
                let mut picked = Vec::with_capacity(arguments.len());
                for key in arguments {
                    match key {
                        Object::Int(index) => {
                            picked.push(indexed_value(&values, *index, position)?);
                        }
                        Object::Range { .. } => {
                            picked.extend(range_values(&values, key, position)?);
                        }
                        other => {
                            return Err(MetorexError::type_error(
                                format!(
                                    "no implicit conversion of {} into Integer",
                                    self.builtins().class_of(other).ruby_name()
                                ),
                                position_to_location(position),
                            ));
                        }
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(picked)))))
            }
            "select" | "filter" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    if !arguments.is_empty() {
                        return Err(argument_error(
                            format!(
                                "wrong number of arguments (given {}, expected 0)",
                                arguments.len()
                            ),
                            position,
                        ));
                    }
                    return self
                        .build_enumerator(
                            receiver.clone(),
                            method_name,
                            arguments.to_vec(),
                            Some(members.len() as i64),
                            position,
                        )
                        .map(Some);
                };
                if !arguments.is_empty() {
                    return Err(argument_error(
                        format!(
                            "wrong number of arguments (given {}, expected 0)",
                            arguments.len()
                        ),
                        position,
                    ));
                }
                let mut kept = Vec::new();
                for value in member_values(receiver, members) {
                    let verdict =
                        self.execute_block_callable(&block, vec![value.clone()], position)?;
                    if verdict.is_truthy() {
                        kept.push(value);
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(kept)))))
            }
            "each" | "each_pair" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return self
                        .build_enumerator(
                            receiver.clone(),
                            method_name,
                            arguments.to_vec(),
                            Some(members.len() as i64),
                            position,
                        )
                        .map(Some);
                };
                for member in members {
                    let args = if method_name == "each" {
                        vec![member_value(receiver, member)]
                    } else {
                        vec![
                            Object::Symbol(Rc::new(member.clone())),
                            member_value(receiver, member),
                        ]
                    };
                    self.execute_block_with_control_flow(&block, args)?;
                }
                Ok(Some(receiver.clone()))
            }
            "==" | "eql?" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                Ok(Some(Object::Bool(self.struct_equals(
                    class_rc,
                    members,
                    receiver,
                    other,
                    method_name == "eql?",
                ))))
            }
            "hash" => {
                let rendered = format!(
                    "{}[{}]({})",
                    class_rc.ruby_name(),
                    members.join(","),
                    member_values(receiver, members)
                        .iter()
                        .map(crate::vm::native_methods::array_methods::inspect_element)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let mut digest: i64 = 0;
                for byte in rendered.bytes() {
                    digest = digest.wrapping_mul(31).wrapping_add(byte as i64);
                }
                Ok(Some(Object::Int(digest)))
            }
            "inspect" | "to_s" => {
                let body = members
                    .iter()
                    .map(|member| {
                        format!(
                            "{}={}",
                            member,
                            crate::vm::native_methods::array_methods::inspect_element(
                                &member_value(receiver, member)
                            )
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                // A struct nested in an anonymous class or module has no
                // Ruby name, which the synthesized `#<Class:0x..>::Foo` label
                // stands in for. Print it the way an anonymous struct prints.
                let name = class_rc.ruby_name();
                let anonymous = name.is_empty() || name.contains("#<");
                Ok(Some(Object::string(if anonymous {
                    format!("#<struct {}>", body)
                } else {
                    format!("#<struct {} {}>", name, body)
                })))
            }
            _ => Ok(None),
        }
    }

    /// The rest of a `dig` sequence, handed to the intermediate value's own
    /// `dig` the way Ruby does, so it decides how far the walk goes.
    pub(crate) fn dig_into(
        &mut self,
        value: &Object,
        keys: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some((class, method)) = self.lookup_method(value, "dig") {
            return self.invoke_method(class, method, value.clone(), keys.to_vec(), position);
        }
        let class = self.builtins().class_of(value);
        if let Some(result) = self.call_native_method(&class, value, "dig", keys, position)? {
            return Ok(result);
        }
        Err(MetorexError::type_error(
            format!("{} does not have #dig method", class.ruby_name()),
            position_to_location(position),
        ))
    }

    /// `deconstruct_keys(keys)` for pattern matching. It answers the members
    /// the keys name, stopping at the first key the struct has no value for,
    /// and an empty hash when more keys are asked for than there are members.
    fn deconstruct_struct_keys(
        &mut self,
        members: &[String],
        receiver: &Object,
        keys: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut pairs = IndexMap::new();
        let requested = match keys {
            Object::Nil => {
                for member in members {
                    pairs.insert(format!(":{}", member), member_value(receiver, member));
                }
                return Ok(Object::Dict(Rc::new(RefCell::new(pairs))));
            }
            Object::Array(elements) => elements.borrow().clone(),
            other => {
                let class_name = crate::vm::native_methods::define_method::ruby_class_name(other);
                return Err(MetorexError::type_error(
                    format!("wrong argument type {} (expected Array or nil)", class_name),
                    position_to_location(position),
                ));
            }
        };
        if requested.len() > members.len() {
            return Ok(Object::Dict(Rc::new(RefCell::new(pairs))));
        }
        for key in &requested {
            let found = match key {
                Object::Symbol(name) | Object::String(name) => members
                    .iter()
                    .find(|member| *member == &**name)
                    .map(|member| member_value(receiver, member)),
                _ => {
                    let index = self.key_as_index(key, position)?;
                    let length = members.len() as i64;
                    let resolved = if index < 0 { index + length } else { index };
                    if resolved < 0 || resolved >= length {
                        None
                    } else {
                        Some(member_value(receiver, &members[resolved as usize]))
                    }
                }
            };
            let Some(value) = found else {
                break;
            };
            let rendered = crate::vm::utils::object_to_dict_key(key).unwrap_or_default();
            if !crate::vm::utils::is_primitive_key(key) {
                remember_key_object(&mut pairs, &rendered, key);
            }
            pairs.insert(rendered, value);
        }
        Ok(Object::Dict(Rc::new(RefCell::new(pairs))))
    }

    /// A `deconstruct_keys` key that names a position rather than a member,
    /// coerced through `to_int` when it is not already an Integer.
    fn key_as_index(&mut self, key: &Object, position: Position) -> Result<i64, MetorexError> {
        if let Object::Int(index) = key {
            return Ok(*index);
        }
        let class_name = match key {
            Object::Instance(instance) => Rc::clone(&instance.borrow().class).ruby_name(),
            other => crate::vm::native_methods::define_method::ruby_class_name(other).to_string(),
        };
        let Some((class, method)) = self.lookup_method(key, "to_int") else {
            return Err(MetorexError::type_error(
                format!("no implicit conversion of {} into Integer", class_name),
                position_to_location(position),
            ));
        };
        match self.invoke_method(class, method, key.clone(), vec![], position)? {
            Object::Int(index) => Ok(index),
            _ => Err(MetorexError::type_error(
                format!("can't convert {} into Integer", class_name),
                position_to_location(position),
            )),
        }
    }

    /// The `[key, value]` pair a `to_h` block answers, coerced with `to_ary`
    /// when it is not already an Array.
    pub(crate) fn pair_from_block_result(
        &mut self,
        produced: Object,
        position: Position,
    ) -> Result<(Object, Object), MetorexError> {
        let pair = match &produced {
            Object::Array(_) => produced.clone(),
            other => {
                let coerced = match self.lookup_method(other, "to_ary") {
                    Some((class, method)) => {
                        self.invoke_method(class, method, other.clone(), vec![], position)?
                    }
                    None => Object::Nil,
                };
                if !matches!(coerced, Object::Array(_)) {
                    let class_name = match other {
                        Object::Instance(instance) => {
                            Rc::clone(&instance.borrow().class).ruby_name()
                        }
                        _ => crate::vm::native_methods::define_method::ruby_class_name(other)
                            .to_string(),
                    };
                    return Err(MetorexError::type_error(
                        format!("wrong element type {} (expected array)", class_name),
                        position_to_location(position),
                    ));
                }
                coerced
            }
        };
        let Object::Array(elements) = &pair else {
            unreachable!("pair is an Array by construction");
        };
        let elements = elements.borrow();
        if elements.len() != 2 {
            return Err(argument_error(
                format!(
                    "element has wrong array length (expected 2, was {})",
                    elements.len()
                ),
                position,
            ));
        }
        Ok((elements[0].clone(), elements[1].clone()))
    }

    /// Two structs are equal when they share a class and every member value
    /// compares equal. `strict` selects `eql?` semantics, under which 1998
    /// and 1998.0 differ.
    fn struct_equals(
        &mut self,
        class_rc: &Rc<Class>,
        members: &[String],
        receiver: &Object,
        other: &Object,
        strict: bool,
    ) -> bool {
        let mut in_flight = Vec::new();
        struct_values_equal(class_rc, members, receiver, other, strict, &mut in_flight)
    }
}

/// A cyclic struct compares equal to another cycle of the same shape, so a
/// pair already being compared higher up the stack is taken as equal.
fn struct_values_equal(
    class_rc: &Rc<Class>,
    members: &[String],
    receiver: &Object,
    other: &Object,
    strict: bool,
    in_flight: &mut Vec<(usize, usize)>,
) -> bool {
    let Object::Instance(other_instance) = other else {
        return false;
    };
    if !Rc::ptr_eq(&other_instance.borrow().class, class_rc) {
        return false;
    }
    let pair = match (receiver, other) {
        (Object::Instance(left), Object::Instance(right)) => {
            (Rc::as_ptr(left) as usize, Rc::as_ptr(right) as usize)
        }
        _ => return false,
    };
    if pair.0 == pair.1 || in_flight.contains(&pair) {
        return true;
    }
    in_flight.push(pair);
    let equal = members.iter().all(|member| {
        member_values_equal(
            &member_value(receiver, member),
            &member_value(other, member),
            strict,
            in_flight,
        )
    });
    in_flight.pop();
    equal
}

fn member_values_equal(
    left: &Object,
    right: &Object,
    strict: bool,
    in_flight: &mut Vec<(usize, usize)>,
) -> bool {
    if let Object::Instance(instance) = left
        && let Some(members) = struct_members(&Rc::clone(&instance.borrow().class))
    {
        let class_rc = Rc::clone(&instance.borrow().class);
        return struct_values_equal(&class_rc, &members, left, right, strict, in_flight);
    }
    if strict && std::mem::discriminant(left) != std::mem::discriminant(right) {
        return false;
    }
    left.equals(right)
}
