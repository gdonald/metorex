// Walking an array while a block may be changing it.

use super::*;

impl VirtualMachine {
    /// Walk `array_rc` and keep the elements the block answers for, writing
    /// the result back as it goes. An element the block raises on is kept,
    /// along with everything it has not reached yet, which is what Ruby
    /// leaves behind. Answers how many elements survived.
    pub(crate) fn filter_in_place(
        &mut self,
        array_rc: &Rc<RefCell<Vec<Object>>>,
        block: &Rc<crate::object::BlockStatement>,
        rejecting: bool,
        position: Position,
    ) -> Result<usize, MetorexError> {
        let mut kept: Vec<Object> = Vec::new();
        let mut index = 0;
        while let Some(element) = element_at(array_rc, index) {
            index += 1;
            let verdict = match self.execute_block_callable(block, vec![element.clone()], position)
            {
                Ok(verdict) => verdict,
                Err(error) => {
                    let mut remaining = kept;
                    remaining.push(element);
                    let tail = array_rc.borrow()[index..].to_vec();
                    remaining.extend(tail);
                    *array_rc.borrow_mut() = remaining;
                    return Err(error);
                }
            };
            if verdict.is_truthy() != rejecting {
                kept.push(element);
            }
        }
        let survived = kept.len();
        *array_rc.borrow_mut() = kept;
        Ok(survived)
    }

    /// Ruby warns when a method was handed both an argument and a block and
    /// the argument is the one it uses.
    pub(crate) fn warn_unused_block(&mut self, position: Position) -> Result<(), MetorexError> {
        if self.pending_block.take().is_none() {
            return Ok(());
        }
        let file = self
            .current_source_file
            .clone()
            .unwrap_or_else(|| "-".to_string());
        let message = format!(
            "{}:{}: warning: given block not used\n",
            file, position.line
        );
        self.warn_through_warning_module(message, position)
    }
}

/// Whether a failure is the one an object raises for a call it has no method
/// for, which is what a `method_missing` that passes the call along raises.
pub(crate) fn refused_the_call(problem: &MetorexError) -> bool {
    matches!(problem, MetorexError::UncaughtException { exception, .. }
        if matches!(exception, Object::Exception(held)
            if held.borrow().exception_type == "NoMethodError"))
}

/// One element of an array, read without holding a borrow, so the block a
/// walk is running may modify the array it is walking.
pub(crate) fn element_at(array_rc: &Rc<RefCell<Vec<Object>>>, index: usize) -> Option<Object> {
    array_rc.borrow().get(index).cloned()
}

/// The backing array of an instance of an Array subclass, so it compares the
/// way the Array it stands for does.
pub(crate) fn elements_of(value: &Object) -> Option<Object> {
    crate::vm::native_methods::array_subclass_value(value)
}
