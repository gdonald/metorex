// Upvalue capture and close helpers for the bytecode VM

use std::cell::RefCell;
use std::rc::Rc;

use super::BytecodeVm;
use super::types::UpvalueObj;

impl BytecodeVm {
    /// Capture a local variable as an upvalue (public for testing).
    pub fn capture_upvalue_public(&mut self, stack_index: usize) -> Rc<RefCell<UpvalueObj>> {
        self.capture_upvalue(stack_index)
    }

    /// Close upvalues at or above `last` (public for testing).
    pub fn close_upvalues_public(&mut self, last: usize) {
        self.close_upvalues(last);
    }

    /// Capture a local variable as an upvalue. If an open upvalue for this
    /// stack slot already exists, reuse it (so multiple closures share the
    /// same upvalue cell).
    pub(super) fn capture_upvalue(&mut self, stack_index: usize) -> Rc<RefCell<UpvalueObj>> {
        // Check if we already have an open upvalue for this slot
        for uv in &self.open_upvalues {
            if uv.borrow().stack_index == stack_index {
                return Rc::clone(uv);
            }
        }
        // Create a new open upvalue
        let value = self.stack[stack_index].clone();
        let uv = Rc::new(RefCell::new(UpvalueObj::new_open(stack_index, value)));
        self.open_upvalues.push(Rc::clone(&uv));
        uv
    }

    /// Close all upvalues that point to stack slots at or above `last`.
    /// This moves the value from the stack into the upvalue cell.
    pub(super) fn close_upvalues(&mut self, last: usize) {
        self.open_upvalues.retain(|uv| {
            let mut uv_ref = uv.borrow_mut();
            if uv_ref.stack_index >= last {
                // Close this upvalue: copy the current stack value into it
                let value = self.stack[uv_ref.stack_index].clone();
                uv_ref.close(value);
                false // remove from open list
            } else {
                true // keep in open list
            }
        });
    }
}
