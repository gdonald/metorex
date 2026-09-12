//! The interpreter's side of `TracePoint`: which traces are switched on, and
//! calling them as the events they asked for happen.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use std::rc::Rc;

impl VirtualMachine {
    /// Take a tracepoint's own record of whether it is on and bring the
    /// interpreter's list into line with it.
    pub(crate) fn register_tracepoint(&mut self, tracepoint: &Object) {
        let switched_on = matches!(
            self.read_instance_var(tracepoint, "enabled"),
            Some(Object::Bool(true))
        );
        self.tracepoints
            .retain(|held| !same_tracepoint(held, tracepoint));
        if switched_on {
            self.tracepoints.push(tracepoint.clone());
        }
    }

    /// Whether a tracepoint handler is running, which is what
    /// `TracePoint.allow_reentry` is only allowed inside.
    pub(crate) fn is_tracing(&self) -> bool {
        self.tracing
    }

    /// Switch tracing off and answer what it was, so a block can fire events
    /// of its own from inside a handler. The line already reported is carried
    /// along too: the lines the block runs are its own, and the line the
    /// handler was called from still counts as reported.
    pub(crate) fn take_tracing(&mut self) -> (bool, Option<(String, usize)>) {
        (
            std::mem::replace(&mut self.tracing, false),
            self.traced_line.take(),
        )
    }

    /// Put the tracing flag and the reported line back to what
    /// `take_tracing` found.
    pub(crate) fn restore_tracing(&mut self, held: (bool, Option<(String, usize)>)) {
        self.tracing = held.0;
        self.traced_line = held.1;
    }

    /// Fire a `:line` event for the statement about to run. A statement on a
    /// line already traced fires nothing, so one line is one event however
    /// many statements share it.
    pub(crate) fn fire_line_event(&mut self, position: Position) -> Result<(), MetorexError> {
        // The core library stands in for Ruby's C code, which a trace never
        // sees.
        if self.tracepoints.is_empty() || self.tracing || position.prelude {
            return Ok(());
        }
        // One line reports once, and the same line number in another file is
        // a line of its own.
        let reached = (self.traced_path(), position.line);
        if self.traced_line.as_ref() == Some(&reached) {
            return Ok(());
        }
        self.traced_line = Some(reached);
        self.fire_event("line", position, Vec::new())
    }

    /// Hand one event to every tracepoint that asked for it.
    pub(crate) fn fire_event(
        &mut self,
        event: &str,
        position: Position,
        extra: Vec<(&str, Object)>,
    ) -> Result<(), MetorexError> {
        if self.tracepoints.is_empty() || self.tracing {
            return Ok(());
        }
        // Everything from here on runs the trace's own code, which never
        // fires events of its own.
        self.tracing = true;
        let outcome = self.deliver_event(event, position, &extra);
        self.tracing = false;
        outcome
    }

    /// Hand one event to each listening tracepoint. Called with tracing
    /// already marked, so nothing here fires again.
    fn deliver_event(
        &mut self,
        event: &str,
        position: Position,
        extra: &[(&str, Object)],
    ) -> Result<(), MetorexError> {
        let listening = self.tracepoints.clone();
        let name = Object::symbol(event.to_string());
        for tracepoint in listening {
            let wants = self.send_to_object(
                tracepoint.clone(),
                "__wants__",
                vec![name.clone()],
                position,
            )?;
            if !matches!(wants, Object::Bool(true)) {
                continue;
            }
            let details = self.event_details(event, position, extra);
            self.send_to_object(tracepoint.clone(), "__handle__", vec![details], position)?;
        }
        Ok(())
    }

    /// What the event says about itself, as the Hash a tracepoint reads its
    /// answers out of.
    fn event_details(
        &mut self,
        event: &str,
        position: Position,
        extra: &[(&str, Object)],
    ) -> Object {
        let mut entries = indexmap::IndexMap::new();
        entries.insert("event".to_string(), Object::symbol(event.to_string()));
        entries.insert("lineno".to_string(), Object::Int(position.line as i64));
        entries.insert("path".to_string(), Object::string(self.traced_path()));
        entries.insert(
            "self".to_string(),
            self.environment().get("self").unwrap_or(Object::Nil),
        );
        for (name, value) in extra {
            entries.insert((*name).to_string(), value.clone());
        }
        Object::Dict(Rc::new(std::cell::RefCell::new(entries)))
    }

    /// The file the interpreter is running, as `path` reports it.
    fn traced_path(&self) -> String {
        match &self.current_file {
            Some(path) => path.to_string_lossy().to_string(),
            None => "-e".to_string(),
        }
    }

    /// One instance variable off an object, where it has one.
    fn read_instance_var(&self, holder: &Object, name: &str) -> Option<Object> {
        let Object::Instance(instance) = holder else {
            return None;
        };
        instance.borrow().get_var(name).cloned()
    }
}

/// Whether two references name the same tracepoint.
fn same_tracepoint(one: &Object, other: &Object) -> bool {
    match (one, other) {
        (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

impl VirtualMachine {
    /// Fire a `:call` or `:return` event for a method written in Ruby. The
    /// method's own name, the class it was found on, and what it answered are
    /// what a trace reads back.
    pub(crate) fn fire_method_event(
        &mut self,
        event: &str,
        method_name: &str,
        method: &Rc<crate::object::Method>,
        found_on: &Rc<crate::class::Class>,
        value: Option<&Object>,
        position: Position,
    ) -> Result<(), MetorexError> {
        if self.tracepoints.is_empty() || self.tracing || position.prelude {
            return Ok(());
        }
        // The core library stands in for Ruby's C code, so a method written
        // there fires nothing however it was reached.
        if method
            .body
            .first()
            .is_some_and(|held| held.position().prelude)
        {
            return Ok(());
        }
        let plain = method_name
            .strip_prefix("__class__")
            .unwrap_or(method_name)
            .to_string();
        let mut extra = vec![
            ("method_id", Object::symbol(plain.clone())),
            ("callee_id", Object::symbol(plain)),
            // Ruby names the class the method was written on, which is not
            // always the one the call found it through.
            ("defined_class", {
                // A module the method was written on stands as a module, not
                // as a class, so the object a trace reads is the same one the
                // name reaches.
                let owner = method
                    .owner_class
                    .clone()
                    .unwrap_or_else(|| Rc::clone(found_on));
                if owner.is_module() {
                    Object::Module(owner)
                } else {
                    Object::Class(owner)
                }
            }),
            // A trace reads the parameters a method declared off either of
            // the events that name the method.
            (
                "parameters",
                crate::vm::native_methods::method_parameter_list(method),
            ),
        ];
        if let Some(value) = value {
            extra.push(("return_value", value.clone()));
        }
        if event == "return"
            && let Some(held) = self.traced_binding.take()
        {
            extra.push(("binding", held));
        }
        self.fire_event(event, position, extra)
    }
}
