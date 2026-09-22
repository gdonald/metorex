// The order a File or Dir class method is looked for in. Each step answers
// `None` when the name is none of its own, and the next step is tried.

use super::*;

/// One step of the chain: every group reads the same arguments and answers
/// the same way.
type FileMethodStep = fn(
    &mut VirtualMachine,
    &Rc<Class>,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

/// The groups that answer for File itself, tried once the receiver is known
/// to be File.
const FILE_STEPS: &[FileMethodStep] = &[
    VirtualMachine::call_file_ownership_methods,
    VirtualMachine::call_file_reading_methods,
    VirtualMachine::call_file_opening_methods,
    VirtualMachine::call_file_link_methods,
    VirtualMachine::call_file_path_methods,
    VirtualMachine::call_file_attribute_methods,
];

impl VirtualMachine {
    /// The File or Dir class method `method_name` names, or `None` when no
    /// group answers that name.
    pub(crate) fn call_file_dir_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // A path may arrive as an object that names one through `to_path` or
        // `to_str`, which Ruby asks for before the path reaches the
        // filesystem.
        let coerced;
        let arguments = match arguments.first() {
            Some(first)
                if !matches!(first, Object::String(_))
                    && matches!(class_rc.name(), "File" | "Dir")
                    && names_a_path(method_name) =>
            {
                match self.path_naming_answer(first, position)? {
                    Some(named) => {
                        let mut rest = arguments.to_vec();
                        rest[0] = named;
                        coerced = rest;
                        coerced.as_slice()
                    }
                    None => arguments,
                }
            }
            _ => arguments,
        };
        let coerced_second;
        let arguments = match arguments.get(1) {
            Some(second)
                if !matches!(second, Object::String(_))
                    && class_rc.name() == "File"
                    && names_a_second_path(method_name) =>
            {
                match self.path_naming_answer(second, position)? {
                    Some(named) => {
                        let mut rest = arguments.to_vec();
                        rest[1] = named;
                        coerced_second = rest;
                        coerced_second.as_slice()
                    }
                    None => arguments,
                }
            }
            _ => arguments,
        };
        const DIR_STEPS: &[FileMethodStep] = &[
            VirtualMachine::call_dir_class_methods,
            VirtualMachine::call_dir_entry_methods,
        ];
        for step in DIR_STEPS {
            if let Some(result) = step(self, class_rc, method_name, arguments, position)? {
                return Ok(Some(result));
            }
        }
        if class_rc.name() != "File" {
            return Ok(None);
        }
        for step in FILE_STEPS {
            if let Some(result) = step(self, class_rc, method_name, arguments, position)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}
