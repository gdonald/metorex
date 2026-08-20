//! Struct: the class builder `Struct.new` and the instance methods every
//! generated struct class inherits.

use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Instance, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Class variable holding the ordered member names of a generated struct class.
const MEMBERS_VAR: &str = "__struct_members__";
/// Class variable holding the `keyword_init:` value the struct was built with.
const KEYWORD_INIT_VAR: &str = "__struct_keyword_init__";

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

fn name_error(message: String, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception("NameError", message.clone()),
        location: position_to_location(position),
        message,
    }
}

/// Pull the parser-marked keyword-argument hash off the end of an argument
/// list, leaving the positional arguments behind.
fn take_keyword_arguments(arguments: &[Object]) -> (Vec<Object>, HashMap<String, Object>) {
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
    (arguments.to_vec(), HashMap::new())
}

/// The value stored for `member` on a struct instance.
fn member_value(receiver: &Object, member: &str) -> Object {
    match receiver {
        Object::Instance(instance) => instance
            .borrow()
            .instance_vars
            .get(member)
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
                    name: member.clone(),
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
                    name: member.clone(),
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
            generated.declare_instance_var(member);
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
        let by_keyword = matches!(keyword_init(class_rc), Object::Bool(true))
            || (positional.is_empty() && !keywords.is_empty());

        let mut instance = Instance::new(Rc::clone(class_rc));

        if by_keyword {
            for (name, value) in &keywords {
                if !members.iter().any(|member| member == name) {
                    return Err(argument_error(
                        format!("unknown keywords: :{}", name),
                        position,
                    ));
                }
                instance.instance_vars.insert(name.clone(), value.clone());
            }
        } else {
            if positional.len() > members.len() {
                return Err(argument_error("struct size differs".to_string(), position));
            }
            for (member, value) in members.iter().zip(positional.iter()) {
                instance.instance_vars.insert(member.clone(), value.clone());
            }
        }

        for member in members {
            instance
                .instance_vars
                .entry(member.clone())
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
        match method_name {
            "members" => Ok(Some(symbols(members))),
            "size" | "length" => Ok(Some(Object::Int(members.len() as i64))),
            "to_a" | "values" | "deconstruct" => Ok(Some(Object::Array(Rc::new(RefCell::new(
                member_values(receiver, members),
            ))))),
            "to_h" => {
                let mut pairs = HashMap::new();
                for member in members {
                    pairs.insert(format!(":{}", member), member_value(receiver, member));
                }
                Ok(Some(Object::Dict(Rc::new(RefCell::new(pairs)))))
            }
            "deconstruct_keys" => {
                let mut pairs = HashMap::new();
                for member in members {
                    pairs.insert(format!(":{}", member), member_value(receiver, member));
                }
                Ok(Some(Object::Dict(Rc::new(RefCell::new(pairs)))))
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
                if let Object::Instance(instance) = receiver {
                    instance
                        .borrow_mut()
                        .instance_vars
                        .insert(member, arguments[1].clone());
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
                let mut value = member_value(receiver, &member);
                for key in &arguments[1..] {
                    if matches!(value, Object::Nil) {
                        return Ok(Some(Object::Nil));
                    }
                    value = self.dig_into(&value, key, position)?;
                }
                Ok(Some(value))
            }
            "values_at" => {
                let mut picked = Vec::with_capacity(arguments.len());
                for key in arguments {
                    let member = resolve_member(members, key, class_rc, position)?;
                    picked.push(member_value(receiver, &member));
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(picked)))))
            }
            "each" | "each_pair" => {
                let Some(Object::Block(block)) = self.pending_block.take() else {
                    return Err(MetorexError::runtime_error(
                        format!("{} requires a block", method_name),
                        position_to_location(position),
                    ));
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
                Ok(Some(Object::Bool(
                    self.struct_equals(class_rc, members, receiver, other),
                )))
            }
            "hash" => {
                let rendered = format!(
                    "{}({})",
                    class_rc.ruby_name(),
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
                let name = class_rc.ruby_name();
                Ok(Some(Object::string(if name.is_empty() {
                    format!("#<struct {}>", body)
                } else {
                    format!("#<struct {} {}>", name, body)
                })))
            }
            _ => Ok(None),
        }
    }

    /// One step of `dig`: call `dig` on `value` if it answers one, otherwise
    /// fall back to the native table for arrays and hashes.
    pub(crate) fn dig_into(
        &mut self,
        value: &Object,
        key: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some((class, method)) = self.lookup_method(value, "dig") {
            return self.invoke_method(class, method, value.clone(), vec![key.clone()], position);
        }
        let class = self.builtins().class_of(value);
        Ok(self
            .call_native_method(&class, value, "dig", std::slice::from_ref(key), position)?
            .unwrap_or(Object::Nil))
    }

    /// Two structs are equal when they share a class and every member value
    /// compares equal.
    fn struct_equals(
        &mut self,
        class_rc: &Rc<Class>,
        members: &[String],
        receiver: &Object,
        other: &Object,
    ) -> bool {
        let Object::Instance(other_instance) = other else {
            return false;
        };
        if !Rc::ptr_eq(&other_instance.borrow().class, class_rc) {
            return false;
        }
        members
            .iter()
            .all(|member| member_value(receiver, member).equals(&member_value(other, member)))
    }
}
