// `Struct.new`, and the class it hands back.

use super::*;

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
        // `Struct.new(:a)` builds a class, and so does `new` on a subclass of
        // Struct that has no members yet: naming them is what turns it into a
        // struct class of its own.
        if method_name == "new"
            && struct_members(class_rc).is_none()
            && (class_rc.name() == "Struct"
                || crate::vm::method_invocation::descends_from(class_rc, "Struct"))
        {
            return self
                .build_struct_class(class_rc, arguments, position)
                .map(Some);
        }

        let Some(members) = struct_members(class_rc) else {
            return Ok(None);
        };

        match method_name {
            // A class of the program's own that wrote its own `initialize`
            // shapes the instance itself, so construction goes the ordinary
            // way and reaches that method.
            "new"
                if class_rc.find_method("initialize").is_some_and(|found| {
                    !found.body.is_empty()
                        && found
                            .owner_class
                            .as_ref()
                            .is_some_and(|owner| owner.name() != "Struct")
                }) =>
            {
                self.invoke_class(Rc::clone(class_rc), arguments.to_vec(), position)
                    .map(Some)
            }
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
    pub(crate) fn build_struct_class(
        &mut self,
        parent: &Rc<Class>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (positional, keywords) = take_keyword_arguments(arguments);

        // A leading String names a constant the class is registered under
        // rather than contributing a member, and `nil` names none at all.
        let mut index = 0;
        let mut constant_name = None;
        if let Some(first) = positional.first() {
            match first {
                Object::Nil => index = 1,
                Object::String(held) => {
                    constant_name = Some(held.as_str().to_string());
                    index = 1;
                }
                Object::Symbol(_) => {}
                other if self.answers_to(other, "to_str", position)? => {
                    let named = self.send_to_object(other.clone(), "to_str", vec![], position)?;
                    if let Object::String(held) = named {
                        constant_name = Some(held.as_str().to_string());
                        index = 1;
                    }
                }
                _ => {}
            }
        }
        if let Some(named) = &constant_name
            && !named.chars().next().is_some_and(|held| held.is_uppercase())
        {
            let message = format!("identifier {} needs to be constant", named);
            return Err(crate::vm::errors::simple_exception(
                "NameError",
                &message,
                position,
            ));
        }

        let mut members: Vec<String> = Vec::new();
        for argument in &positional[index..] {
            let named = match argument {
                Object::Symbol(name) => name.as_str().to_string(),
                Object::String(name) => name.as_str().to_string(),
                other => {
                    return Err(MetorexError::type_error(
                        format!("{} is not a symbol nor a string", other),
                        position_to_location(position),
                    ));
                }
            };
            if members.contains(&named) {
                return Err(argument_error(
                    format!("duplicate member: {}", named),
                    position,
                ));
            }
            members.push(named);
        }

        let Some(Object::Class(struct_class)) = self.globals().get("Struct") else {
            return Err(MetorexError::runtime_error(
                "Struct is not defined",
                position_to_location(position),
            ));
        };

        // A struct class built from a subclass of Struct stands under that
        // subclass, so a method it wrote is on the chain.
        let generated = Rc::new(Class::new("", Some(Rc::clone(parent))));
        parent.add_subclass(&generated);
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
                    name: crate::object::UNNAMED_PARAMETER.to_string(),
                    position,
                },
                position,
            }];
            generated.define_method(
                &writer_name,
                Rc::new(crate::object::Method::new(
                    writer_name.clone(),
                    vec![crate::object::UNNAMED_PARAMETER.to_string()],
                    writer_body,
                )),
            );
            generated.declare_instance_var(member_slot(member));
        }

        if let Some(name) = constant_name {
            // A struct built from a subclass of Struct registers its constant
            // on that subclass, which is where a program looks for it.
            let owner = if Rc::ptr_eq(parent, &struct_class) {
                Rc::clone(&struct_class)
            } else {
                Rc::clone(parent)
            };
            if owner.get_class_var(&name).is_some() {
                let message = format!(
                    "warning: already initialized constant {}::{}",
                    owner.ruby_name(),
                    name
                );
                self.emit_warning_to_stderr(&message, position);
            }
            let qualified = format!("{}::{}", owner.ruby_name(), name);
            generated.assign_name_recursive(&qualified);
            owner.set_class_var(&name, Object::Class(Rc::clone(&generated)));
            self.globals_mut()
                .set(qualified, Object::Class(Rc::clone(&generated)));
        }

        if let Some(Object::Block(block)) = self.pending_block.take() {
            self.apply_block_as_class_body(&generated, &block, position)?;
        }

        Ok(Object::Class(generated))
    }

    /// `Point.new(1, 2)` / `Point.new(x: 1, y: 2)` for a generated struct class.
    pub(crate) fn build_struct_instance(
        &mut self,
        class_rc: &Rc<Class>,
        members: &[String],
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (mut positional, keywords) = take_keyword_arguments(arguments);
        let named_only = keyword_init(class_rc).is_truthy();
        let mut keywords = keywords;
        // A struct built with `keyword_init: true` takes one Hash, written
        // either as keywords or as a Hash of its own.
        if named_only && keywords.is_empty() && positional.len() == 1 {
            let Some(Object::Dict(held)) = crate::vm::native_methods::as_dict(&positional[0])
            else {
                return Err(argument_error(
                    format!(
                        "wrong number of arguments (given {}, expected 0)",
                        positional.len()
                    ),
                    position,
                ));
            };
            keywords = held
                .borrow()
                .iter()
                .map(|(key, value)| {
                    (
                        key.strip_prefix(':').unwrap_or(key).to_string(),
                        value.clone(),
                    )
                })
                .collect();
            positional.clear();
        }
        if named_only && !positional.is_empty() {
            return Err(argument_error(
                format!(
                    "wrong number of arguments (given {}, expected 0)",
                    positional.len()
                ),
                position,
            ));
        }
        // Without that option, keywords standing alone still name the
        // members, and keywords alongside a positional argument are one more
        // positional argument: the Hash they spell.
        let alone = positional.is_empty() && !keywords.is_empty();
        if !named_only && !keywords.is_empty() && !alone {
            let mut held: indexmap::IndexMap<String, Object> = indexmap::IndexMap::new();
            for (name, value) in &keywords {
                held.insert(format!(":{}", name), value.clone());
            }
            positional.push(Object::Dict(Rc::new(RefCell::new(held))));
            keywords = IndexMap::new();
        }
        let by_keyword = named_only || alone;

        let mut instance = Instance::new(Rc::clone(class_rc));

        if by_keyword {
            let unknown: Vec<String> = keywords
                .keys()
                .filter(|name| !members.iter().any(|member| &member == name))
                .cloned()
                .collect();
            if !unknown.is_empty() {
                return Err(argument_error(
                    format!("unknown keywords: {}", unknown.join(", ")),
                    position,
                ));
            }
            for (name, value) in &keywords {
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
}
