// Writing a warning out, and the name a refusal calls a value by.

use super::*;

impl VirtualMachine {
    /// Emit a warning line. If `$stderr` has been reassigned to an object that
    /// responds to `write` / `<<` (e.g. mspec's `IOStub` for the `complain`
    /// matcher), route the message there so tests can capture it. Otherwise
    /// fall back to writing the line directly to the process's stderr.
    /// Write the warnings reading `named` gave, each with its line, unless
    /// `-W0` turned warnings off.
    pub fn report_parse_warnings(&mut self, named: &str, warnings: &[(Position, String)]) {
        if matches!(self.globals().get("VERBOSE"), Some(Object::Nil)) {
            return;
        }
        for (at, warning) in warnings {
            self.emit_warning_to_stderr(&format!("{named}:{}: warning: {warning}", at.line), *at);
        }
    }

    pub(crate) fn emit_warning_to_stderr(&mut self, msg: &str, position: Position) {
        // Re-running an already-required file to satisfy an autoload repeats
        // assignments Ruby would have run once, so the warnings they produce
        // describe the re-run rather than the program.
        if self.autoload_reload_depth > 0 {
            return;
        }
        let stderr_obj = self.globals().get("stderr");
        let placeholder = matches!(
            &stderr_obj,
            Some(Object::String(s)) if *s.as_str() == *"$stderr"
        );
        if !placeholder && let Some(obj) = stderr_obj {
            let line = format!("{}\n", msg);
            for cand in ["write", "<<"] {
                if let Some((cls, method)) = self.lookup_method(&obj, cand) {
                    let arg = Object::string(line.clone());
                    let _ = self.invoke_method(cls, method, obj.clone(), vec![arg], position);
                    return;
                }
            }
        }
        eprintln!("{}", msg);
    }

    /// The name a conversion error calls a value by. Ruby names `nil`,
    /// `true` and `false` outright, and everything else by its class.
    pub(crate) fn conversion_name(&self, value: &Object) -> String {
        match value {
            Object::Nil => "nil".to_string(),
            Object::Bool(true) => "true".to_string(),
            Object::Bool(false) => "false".to_string(),
            other => self.builtins().class_of(other).name().to_string(),
        }
    }

    /// Coerce an object into a method-name `String`. Strings and symbols are
    /// taken at face value; for other receivers we invoke `to_str` (matching
    /// Ruby's implicit type coercion). A receiver that lacks `to_str` raises
    /// `TypeError`; a `to_str` that returns a non-String also raises
    /// `TypeError`. Errors raised inside `to_str` (e.g. `NoMethodError`)
    /// propagate unchanged so callers see the original error class.
    pub(crate) fn coerce_method_name(
        &mut self,
        arg: &Object,
        caller: &str,
        position: Position,
    ) -> Result<String, MetorexError> {
        match arg {
            Object::String(s) => Ok(name_text(s)),
            Object::Symbol(s) => Ok(s.as_str().to_string()),
            _ => {
                if let Some((cls, m)) = self.lookup_method(arg, "to_str")
                    && !m.is_undefined
                {
                    let result = self.invoke_method(cls, m, arg.clone(), vec![], position)?;
                    if let Object::String(s) = result {
                        return Ok(s.as_str().to_string());
                    }
                    let source_class = self.builtins().class_of(arg).name().to_string();
                    let msg = format!("can't convert {} into String", source_class);
                    let _ = result;
                    let exc = Object::exception("TypeError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: crate::vm::utils::position_to_location(position),
                        message: msg,
                    });
                }
                let msg = format!(
                    "{} is not a symbol nor a string",
                    match arg {
                        Object::Instance(inst) => format!("#<{}>", inst.borrow().class.name()),
                        _ => arg.to_string(),
                    }
                );
                let _ = caller;
                let exc = Object::exception("TypeError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: crate::vm::utils::position_to_location(position),
                    message: msg,
                })
            }
        }
    }
}
