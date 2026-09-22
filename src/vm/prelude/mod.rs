// Core library pieces defined in Ruby rather than in Rust.
//
// A method written here is a real user-defined method, so it can be aliased,
// redefined, mocked, and introspected the way MRI's own Ruby-level core
// methods can. Kernel#warn relies on that: the specs alias `Warning.warn`
// away and put their own back.
//
// The Ruby source is split across the sibling modules below. Each holds one
// `SOURCE` string, and they are concatenated in order, so a chunk boundary
// falls between two definitions. A class whose body runs long is carried
// across more than one module, so a boundary may sit inside a class body.

use crate::vm::core::VirtualMachine;

mod collections;
mod core_types;
mod encoding;
mod enumerable;
mod enumerator;
mod enumerator_lazy;
mod enumerator_parts;
mod file;
mod io;
mod io_reading;
mod io_streams;
mod io_writing;
mod kernel_argf;
mod match_data_thread;
mod object_space_gc;
mod proc_random;
mod process;
mod string_io;
mod threading;
mod time;
mod time_reading;
mod tracepoint_marshal;
mod warning_math_numeric;

/// What names the core library's own Ruby source. Ruby reports a method
/// written there as coming from `<internal:...>` rather than from a file.
pub(crate) const PRELUDE_FILE: &str = "<internal:prelude>";

/// The Ruby source evaluated into every fresh VM, in the order it is read.
const PRELUDE_CHUNKS: &[&str] = &[
    warning_math_numeric::SOURCE,
    enumerable::SOURCE,
    match_data_thread::SOURCE,
    string_io::SOURCE,
    enumerator::SOURCE,
    enumerator_lazy::SOURCE,
    enumerator_parts::SOURCE,
    collections::SOURCE,
    time::SOURCE,
    time_reading::SOURCE,
    proc_random::SOURCE,
    process::SOURCE,
    io::SOURCE,
    io_reading::SOURCE,
    io_writing::SOURCE,
    file::SOURCE,
    tracepoint_marshal::SOURCE,
    core_types::SOURCE,
    kernel_argf::SOURCE,
    threading::SOURCE,
    object_space_gc::SOURCE,
    encoding::SOURCE,
    io_streams::SOURCE,
];

impl VirtualMachine {
    /// Build an `Enumerator` over `method_name` sent to `receiver`, which is
    /// what a method that yields answers when called without a block.
    pub(crate) fn build_enumerator(
        &mut self,
        receiver: crate::object::Object,
        method_name: &str,
        arguments: Vec<crate::object::Object>,
        size: Option<i64>,
        position: crate::lexer::Position,
    ) -> Result<crate::object::Object, crate::error::MetorexError> {
        use crate::object::Object;
        let Some(enumerator_class) = self.globals().get("Enumerator") else {
            let message = "uninitialized constant Enumerator".to_string();
            return Err(crate::error::MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let arguments = vec![
            receiver,
            Object::symbol(method_name.to_string()),
            Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arguments))),
            match size {
                Some(size) => Object::Int(size),
                None => Object::Nil,
            },
        ];
        self.send_to_object(enumerator_class, "over", arguments, position)
    }

    /// A walk whose count is named by an object rather than a plain Integer,
    /// which is how an endless walk reports a size of Infinity.
    pub(crate) fn build_enumerator_of_size(
        &mut self,
        receiver: crate::object::Object,
        method_name: &str,
        arguments: Vec<crate::object::Object>,
        size: crate::object::Object,
        position: crate::lexer::Position,
    ) -> Result<crate::object::Object, crate::error::MetorexError> {
        use crate::object::Object;
        let Some(enumerator_class) = self.globals().get("Enumerator") else {
            let message = "uninitialized constant Enumerator".to_string();
            return Err(crate::error::MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        };
        let arguments = vec![
            receiver,
            Object::symbol(method_name.to_string()),
            Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arguments))),
            size,
        ];
        self.send_to_object(enumerator_class, "over", arguments, position)
    }

    /// Evaluate the Ruby-level core library. A parse or runtime failure here
    /// is a defect in `PRELUDE_SOURCE` itself, so it panics rather than
    /// leaving a half-built VM behind.
    pub(crate) fn load_prelude(&mut self) {
        let tokens = crate::lexer::Lexer::for_prelude(&PRELUDE_CHUNKS.concat()).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .unwrap_or_else(|errors| panic!("prelude failed to parse: {:?}", errors));
        // The core library is no file of the program's, so a method written
        // here names itself the way Ruby names its own Ruby-level core.
        let held_file = self
            .current_file
            .replace(std::path::PathBuf::from(PRELUDE_FILE));
        let held_source = self.current_source_file.replace(PRELUDE_FILE.to_string());
        self.execute_program(&statements)
            .unwrap_or_else(|error| panic!("prelude failed to run: {}", error));
        self.current_file = held_file;
        self.current_source_file = held_source;
    }
}
