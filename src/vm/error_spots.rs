// Where in the program's own code an error was raised, which error_highlight
// reads to underline the part of the line that failed.

use crate::ast::{Expression, UnaryOp};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;

/// Where an exception keeps the file, line, column and name of the
/// expression it was raised in. Not an `@` name, so a program's own instance
/// variables cannot collide with it.
pub(crate) const SPOT_HINT_KEY: &str = "__spot_hint__";

/// The error classes error_highlight underlines a spot for.
const SPOTTED_ERRORS: &[&str] = &["NameError", "TypeError", "ArgumentError"];

impl VirtualMachine {
    /// Record on an error the expression it was raised in, when that is the
    /// first expression of the program's own code it passes back through.
    /// One the core library raises inside a method it writes in Ruby is
    /// recorded at the call the program made, the way Ruby names the line of
    /// a call into a method it writes in C.
    #[inline(never)]
    pub(crate) fn note_error_spot(
        &mut self,
        result: &Result<Object, MetorexError>,
        expression: &Expression,
    ) {
        let Err(MetorexError::UncaughtException { exception, .. }) = result else {
            return;
        };
        let Some((start, name)) = spot_of(expression) else {
            return;
        };
        self.record_spot_hint(exception, start, name);
    }

    /// Record on the exception a `raise` statement raises the place of the
    /// `raise`, which Ruby's node for the call to `raise` or `fail` names.
    #[inline(never)]
    pub(crate) fn note_raise_spot(&mut self, exception: &Object, position: Position) {
        self.record_spot_hint(exception, position, None);
    }

    /// Record where an error was raised, unless a place nearer the raise is
    /// already recorded.
    fn record_spot_hint(&mut self, exception: &Object, start: Position, name: Option<String>) {
        let Object::Exception(details) = exception else {
            return;
        };
        if details.borrow().instance_vars.contains_key(SPOT_HINT_KEY) {
            return;
        }
        if start.prelude {
            return;
        }
        let Some(path) = self.current_source_file.clone().or_else(|| {
            self.reported_current_file()
                .map(|file| file.display().to_string())
        }) else {
            return;
        };
        // Code `eval` ran has no file to read the line back from, so an error
        // raised there is marked as having none, which the call to `eval`
        // outside leaves as it is.
        if path.starts_with(crate::vm::EVAL_FILE_PREFIX) || self.eval_named_files.contains(&path) {
            details
                .borrow_mut()
                .instance_vars
                .insert(SPOT_HINT_KEY.to_string(), Object::Nil);
            return;
        }
        if !SPOTTED_ERRORS
            .iter()
            .any(|class| self.exception_descends_from(details, class))
        {
            return;
        }
        let hint = Object::array(vec![
            Object::string(path),
            Object::Int(start.line as i64),
            Object::Int(start.column.saturating_sub(1) as i64),
            name.map_or(Object::Nil, Object::symbol),
        ]);
        details
            .borrow_mut()
            .instance_vars
            .insert(SPOT_HINT_KEY.to_string(), hint);
    }
}

/// Where an expression that can raise one of the spotted errors starts, the
/// way Ruby's node for it starts, and the name of the method or constant it
/// reaches.
fn spot_of(expression: &Expression) -> Option<(Position, Option<String>)> {
    match expression {
        Expression::Identifier { name, position } => Some((*position, Some(name.clone()))),
        Expression::TopLevelConstant { name, position } => Some((*position, Some(name.clone()))),
        Expression::ScopeResolution {
            namespace, name, ..
        } => Some((start_of(namespace), Some(name.clone()))),
        Expression::Call { callee, .. } => match callee.as_ref() {
            Expression::Identifier { name, position } => Some((*position, Some(name.clone()))),
            _ => None,
        },
        Expression::MethodCall {
            receiver, method, ..
        } => Some((start_of(receiver), Some(method.clone()))),
        Expression::BinaryOp { op, left, .. } => Some((
            start_of(left),
            crate::vm::eval::binary_op_method_name(op).map(str::to_string),
        )),
        Expression::UnaryOp { op, position, .. } => {
            let name = match op {
                UnaryOp::Minus => "-@",
                UnaryOp::Plus => "+@",
                UnaryOp::Not => "!",
            };
            Some((*position, Some(name.to_string())))
        }
        Expression::Index { array, .. } => Some((start_of(array), Some("[]".to_string()))),
        _ => None,
    }
}

/// Where the leftmost part of an expression starts, which is where Ruby's
/// node for it starts.
fn start_of(expression: &Expression) -> Position {
    match expression {
        Expression::MethodCall { receiver, .. } => start_of(receiver),
        Expression::BinaryOp { left, .. } => start_of(left),
        Expression::Index { array, .. } => start_of(array),
        Expression::ScopeResolution { namespace, .. } => start_of(namespace),
        Expression::Call { callee, .. } => start_of(callee),
        other => other.position(),
    }
}

impl VirtualMachine {
    /// An ArgumentError for a call to a method the program wrote, whose
    /// arguments did not fit, with the method itself as the first place its
    /// backtrace names. Ruby enters the method before it counts what it was
    /// handed, so the line the method is defined on comes first and the call
    /// that made it second.
    #[inline(never)]
    pub(crate) fn arity_error_in_callee(
        &mut self,
        error: MetorexError,
        method: &crate::object::Method,
        class: &std::rc::Rc<crate::class::Class>,
        position: Position,
    ) -> MetorexError {
        let MetorexError::UncaughtException {
            exception: exception @ Object::Exception(_),
            ..
        } = &error
        else {
            return error;
        };
        let Some(location) = &method.source_location else {
            return error;
        };
        let Some(file) = location.filename.clone() else {
            return error;
        };
        let traced = self.add_stack_trace_to_exception(exception.clone(), position);
        let Object::Exception(details) = traced else {
            return error;
        };
        let label = match method.name.strip_prefix("__class__") {
            Some(name) => format!("{}.{}", class.ruby_name(), name),
            None => format!(
                "{}#{}",
                method.owner.clone().unwrap_or_else(|| class.ruby_name()),
                method.name
            ),
        };
        let label = crate::vm::native_functions::backtrace_label(&label);
        let mut held = details.borrow_mut();
        if let Some(sites) = held.backtrace_sites.as_mut() {
            sites.insert(0, (file.clone(), location.line, label.clone()));
            held.site_columns.insert(0, None);
        }
        if let Some(lines) = held.backtrace.as_mut() {
            lines.insert(0, format!("{}:{}:in '{}'", file, location.line, label));
        }
        drop(held);
        error
    }
}

impl VirtualMachine {
    /// An error raised while a method binds its keywords, with its backtrace
    /// taken now, while the method's own frame stands, and starting on the
    /// line the method is defined on, which is where Ruby reports the method
    /// standing when it refuses the keywords it was handed.
    #[inline(never)]
    pub(crate) fn keyword_error_in_callee(
        &mut self,
        error: MetorexError,
        method: &crate::object::Method,
    ) -> MetorexError {
        if let MetorexError::UncaughtException {
            exception: exception @ Object::Exception(_),
            ..
        } = &error
            && let Some(location) = &method.source_location
        {
            self.add_stack_trace_to_exception(
                exception.clone(),
                Position::new(location.line, location.column, location.offset),
            );
        }
        error
    }
}

impl VirtualMachine {
    /// Record on an error an assignment through a setter raised the setter
    /// it called, which Ruby's node for the assignment names.
    #[inline(never)]
    pub(crate) fn note_assignment_spot(
        &mut self,
        result: &Result<Object, MetorexError>,
        target: &Expression,
    ) {
        let Err(MetorexError::UncaughtException { exception, .. }) = result else {
            return;
        };
        let (start, name) = match target {
            Expression::MethodCall {
                receiver, method, ..
            } => {
                let setter = if method.ends_with('=') {
                    method.clone()
                } else {
                    format!("{method}=")
                };
                (start_of(receiver), setter)
            }
            Expression::Index { array, .. } => (start_of(array), "[]=".to_string()),
            _ => return,
        };
        self.record_spot_hint(exception, start, Some(name));
    }
}
