// What a program reports about the call that is running.

use super::*;

impl VirtualMachine {
    pub(crate) fn format_string(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                "sprintf requires at least 1 argument".to_string(),
                crate::vm::utils::position_to_location(position),
            ));
        }
        // Ruby converts the format with `#to_str`, so a non-String
        // that answers it works and anything else raises TypeError.
        let fmt = match &arguments[0] {
            held @ Object::String(_) => held.clone(),
            other => Object::string(self.coerce_name_argument(other, position)?),
        };
        let rest: Vec<Object> = arguments.into_iter().skip(1).collect();
        self.format_with_values(&fmt, rest, position)
    }

    /// `__method__` names the method as it was defined, `__callee__`
    /// as it was called. They differ inside an aliased method. Both
    /// look through block frames to the method that encloses them and
    /// stop at a class body or file scope, where neither has an answer.
    pub(crate) fn running_method_name(&mut self, name: &str) -> Result<Object, MetorexError> {
        Ok(match self.enclosing_method_names() {
            Some((callee, defined)) => {
                let chosen = if name == "__callee__" {
                    callee
                } else {
                    defined
                };
                Object::symbol(chosen)
            }
            None => Object::Nil,
        })
    }

    /// `caller` reports the same frames `caller_locations` does, each
    /// rendered the way a backtrace entry reads.
    pub(crate) fn caller_backtrace(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let Some(locations) = self.sliced_caller_locations(&arguments, position)? else {
            return Ok(Object::Nil);
        };
        let entries: Vec<Object> = locations
            .iter()
            .map(|location| {
                let read = |name: &str| match location {
                    Object::Instance(instance) => instance
                        .borrow()
                        .get_var(name)
                        .cloned()
                        .unwrap_or(Object::Nil),
                    _ => Object::Nil,
                };
                let path = match read("path") {
                    Object::String(path) => path.as_str().to_string(),
                    _ => String::new(),
                };
                let line = match read("lineno") {
                    Object::Int(line) => line,
                    _ => 0,
                };
                let label = match read("label") {
                    Object::String(label) => label.as_str().to_string(),
                    _ => String::new(),
                };
                Object::string(if label.is_empty() {
                    format!("{}:{}", path, line)
                } else {
                    format!("{}:{}:in '{}'", path, line, label)
                })
            })
            .collect();
        Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            entries,
        ))))
    }

    /// `caller_locations(start = 1, length = nil)` — walk the VM call
    /// stack. Each frame stores the source position it was called
    /// from, so level 1 (the caller of the current method) reads the
    /// top frame's recorded location. Returns Location objects
    /// responding to `lineno` and `path`.
    pub(crate) fn caller_location_list(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match self.sliced_caller_locations(&arguments, position)? {
            Some(locations) => Ok(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                locations,
            )))),
            None => Ok(Object::Nil),
        }
    }
}
