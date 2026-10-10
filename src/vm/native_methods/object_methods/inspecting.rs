// What an object reads back as when it defines nothing of its own.

use super::*;

thread_local! {
    /// The instances whose default `inspect` is being rendered now.
    static INSPECTING_INSTANCES: std::cell::RefCell<Vec<usize>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

impl VirtualMachine {
    /// `Object#inspect` for an instance with no `inspect` of its own: the
    /// class and address, then the instance variables and their values. A
    /// private `instance_variables_to_inspect` chooses which to show, and nil
    /// from it means all of them.
    pub(crate) fn default_instance_inspect(
        &mut self,
        receiver: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Object::Instance(instance) = receiver else {
            return Ok(Object::string(receiver.to_string()));
        };
        let header = receiver.to_string();
        // An object inside its own instance variables shows as its header
        // with `...` in place of them.
        let address = std::rc::Rc::as_ptr(instance) as usize;
        if INSPECTING_INSTANCES.with(|held| held.borrow().contains(&address)) {
            return Ok(Object::string(format!(
                "{} ...>",
                header.trim_end_matches('>')
            )));
        }
        INSPECTING_INSTANCES.with(|held| held.borrow_mut().push(address));
        let shown = self.instance_inspect_with_variables(receiver, instance, header, position);
        INSPECTING_INSTANCES.with(|held| held.borrow_mut().retain(|seen| *seen != address));
        shown
    }

    fn instance_inspect_with_variables(
        &mut self,
        receiver: &Object,
        instance: &std::rc::Rc<std::cell::RefCell<crate::object::Instance>>,
        header: String,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let chosen = match self.lookup_method(receiver, "instance_variables_to_inspect") {
            Some((class, method)) if !method.is_undefined => {
                let answer =
                    self.invoke_method(class, method, receiver.clone(), Vec::new(), position)?;
                match answer {
                    Object::Nil => None,
                    Object::Array(names) => Some(
                        names
                            .borrow()
                            .iter()
                            .map(|name| match name {
                                Object::Symbol(text) | Object::String(text) => {
                                    text.as_str().to_string()
                                }
                                other => other.to_string(),
                            })
                            .collect::<Vec<_>>(),
                    ),
                    other => {
                        let message = format!(
                            "Expected #instance_variables_to_inspect to return an Array or nil, but it returned {}",
                            self.builtins().class_of(&other).name()
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: Object::exception("TypeError", message.clone()),
                            location: position_to_location(position),
                            message,
                        });
                    }
                }
            }
            _ => None,
        };
        let variables: Vec<(String, Object)> = {
            let borrowed = instance.borrow();
            let names: Vec<String> = match &chosen {
                Some(chosen) => chosen.clone(),
                None => borrowed.instance_vars.keys().cloned().collect(),
            };
            names
                .into_iter()
                .filter(|name| !name.starts_with("__"))
                .filter_map(|name| {
                    let key = name.trim_start_matches('@').to_string();
                    borrowed
                        .instance_vars
                        .get(&key)
                        .map(|value| (format!("@{}", key), value.clone()))
                })
                .collect()
        };
        if variables.is_empty() {
            return Ok(Object::string(header));
        }
        let mut rendered = Vec::with_capacity(variables.len());
        for (name, value) in variables {
            let shown = self.inspect_value(&value, position)?;
            rendered.push(format!("{}={}", name, shown));
        }
        let trimmed = header.trim_end_matches('>').to_string();
        Ok(Object::string(format!(
            "{} {}>",
            trimmed,
            rendered.join(", ")
        )))
    }

    /// A value as `inspect` renders it.
    pub(crate) fn inspect_value(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        // A user-defined `inspect` wins; otherwise the value's own class
        // renders it, which is what quotes a String and shows a nil.
        if let Some((class, method)) = self.lookup_method(value, "inspect")
            && !method.body.is_empty()
        {
            let shown = self.invoke_method(class, method, value.clone(), Vec::new(), position)?;
            return Ok(shown.to_string());
        }
        // An object of the program's own answers `inspect` the way any call
        // reaches it, which shows its instance variables.
        if let Object::Instance(_) = value {
            let shown = self.send_to_object(value.clone(), "inspect", Vec::new(), position)?;
            return Ok(shown.to_string());
        }
        let class = self.builtins().class_of(value);
        match self.call_native_method(&class, value, "inspect", &[], position)? {
            Some(shown) => Ok(shown.to_string()),
            None => Ok(value.to_string()),
        }
    }
}
