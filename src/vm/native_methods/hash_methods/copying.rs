// A hash derived from another, and the errors a lookup raises.

use super::*;

impl VirtualMachine {
    /// Ruby warns when `fetch` was handed both a default value and a block,
    /// since the block is the one it uses.
    pub(crate) fn warn_hash_block_supersedes(
        &mut self,
        position: Position,
    ) -> Result<(), MetorexError> {
        let file = self
            .current_source_file
            .clone()
            .unwrap_or_else(|| "-".to_string());
        let message = format!(
            "{}:{}: warning: block supersedes default value argument\n",
            file, position.line
        );
        self.warn_through_warning_module(message, position)
    }

    /// Record the hash and the key a KeyError was raised for, which `receiver`
    /// and `key` report.
    pub(crate) fn with_key_error_details(
        &self,
        error: MetorexError,
        receiver: &Object,
        key: &Object,
    ) -> MetorexError {
        let MetorexError::UncaughtException {
            exception,
            location,
            message,
        } = error
        else {
            return error;
        };
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.receiver = Some(Box::new(receiver.clone()));
            details
                .instance_vars
                .insert(crate::vm::KEY_ERROR_KEY.to_string(), key.clone());
        }
        MetorexError::UncaughtException {
            exception,
            location,
            message,
        }
    }
}

/// A hash derived from `source` keeps its compare-by-identity setting, which
/// Ruby carries into `slice`, `merge`, `select`, and the rest.
pub(crate) fn carry_identity(
    source: &Rc<RefCell<indexmap::IndexMap<String, Object>>>,
    mut built: indexmap::IndexMap<String, Object>,
) -> Object {
    if source.borrow().contains_key(BY_IDENTITY_KEY) {
        built.insert(BY_IDENTITY_KEY.to_string(), Object::Bool(true));
    }
    Object::Dict(Rc::new(RefCell::new(built)))
}
