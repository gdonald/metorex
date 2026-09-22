// The example scripts this topic covers, each run and compared against
// the output it is expected to write.

pub(super) use crate::common::EXAMPLES_DIR;
pub(super) use std::process::Command;

mod backtraces;
mod exceptions;
mod exiting;
mod named_errors;
mod raising;
mod rescuing;

/// The expected output of both `errors/raised_position` variants, which differ
/// only in whether the calls are written with parentheses.
pub(super) const RAISED_POSITION_OUTPUT: &str = "ZeroDivisionError\nArray\nArray\nErrno::ENOENT\n";

/// The expected output of both `errors/handled_then_outer` variants, which differ only in whether
/// the calls are written with parentheses.
pub(super) const HANDLED_THEN_OUTER_OUTPUT: &str =
    "\"outer\"\n\"inner\"\n\"outer\"\nRuntimeError\nnil\n:swallowed\n\"again\"\n";

/// The expected output of both `errors/name_and_method_messages` variants, which show
/// how a NameError and a NoMethodError name what they were raised for and differ only in whether the calls are
/// written with parentheses.
pub(super) const NAME_AND_METHOD_MESSAGES_OUTPUT: &str = "undefined local variable or method 'not_defined_anywhere' for main\nuninitialized constant NotDefinedAnywhere\nundefined method 'missing' for class Named\nundefined method 'missing' for an instance of Named\nundefined method 'missing' for #<Object:0xADDRESS>\n";

/// The expected output of both `errors/backtrace/frame_labels` variants, which show
/// how a backtrace names the frame each entry belongs to and differ only in whether the calls are
/// written with parentheses.
pub(super) const BACKTRACE_FRAME_LABELS_OUTPUT: &str = "[\"'Held::Raiser.from_a_class_method'\", \"'<main>'\"]\n[\"'Held::Raiser#from_an_instance_method'\", \"'<main>'\"]\n\"'BasicObject#instance_exec'\"\n";

/// The expected output of both `errors/rescue_handler_shapes` variants.
pub(super) const RESCUE_HANDLER_SHAPES_OUTPUT: &str = concat!(
    "caught\n",
    "by_case_equality\n",
    "class or module required for rescue clause\n",
    "held\n",
    "global\n",
    "[1, 2]\n",
    "SyntaxError\n",
    "SyntaxError\n",
    "SyntaxError\n"
);

/// The expected output of both `errors/rescue_names_in_scope` variants, which
/// differ only in whether the calls are written with parentheses.
pub(super) const RESCUE_NAMES_IN_SCOPE_OUTPUT: &str = concat!(
    "\"nothing named tea\"\n",
    "true\n",
    "Loose\n",
    "\"ab\"\n",
    "1\n",
    "true\n",
);

/// The expected output of both `errors/exception_reports` variants, which
/// differ only in whether the calls are written with parentheses.
pub(super) const EXCEPTION_REPORTS_OUTPUT: &str = "\"first line (RuntimeError)\\nsecond line\"\n\"unhandled exception\"\n\"StandardError\"\n\"RuntimeError\"\n\"RuntimeError\"\ntrue\nnil\ntrue\n[\"/dir/foo.rb:10:in `raising'\"]\n\"outer\"\n\"inner\"\n";

/// The expected output of both `errors/cause/named` variants, which differ only in whether
/// the calls are written with parentheses.
pub(super) const CAUSE_NAMED_OUTPUT: &str = concat!(
    "RuntimeError: the new one <- #<StandardError: named instead>\n",
    "RuntimeError: the new one <- nil\n",
    "ArgumentError: only cause is given with no arguments <- nil\n",
    "TypeError: exception object expected <- nil\n",
    "StandardError: itself <- nil\n",
    "ArgumentError: circular causes <- #<RuntimeError: three>\n",
    "RuntimeError: one <- nil\n"
);
