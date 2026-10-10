// Running a statement again for a continuation called after its `callcc`
// block returned.

use super::*;

/// A statement list running, the statement it is on, and how many frames
/// deep it runs.
#[derive(Clone, Debug)]
pub(crate) struct StatementMark {
    serial: u64,
    depth: usize,
    index: usize,
}

/// The tag a jump back to the statement list `serial` is thrown with.
fn resume_tag(serial: u64) -> Object {
    Object::symbol(format!("__continuation_resume_{serial}__"))
}

impl VirtualMachine {
    /// Run a statement list through `run`, which is handed the statement to
    /// start from and marks each statement as it reaches it. A continuation
    /// that jumps back to the list runs it again from the statement its
    /// `callcc` was written in.
    pub(crate) fn run_restartable<T>(
        &mut self,
        mut run: impl FnMut(&mut Self, usize) -> Result<T, MetorexError>,
    ) -> Result<T, MetorexError> {
        self.next_statement_serial += 1;
        let serial = self.next_statement_serial;
        self.statement_marks.push(StatementMark {
            serial,
            depth: self.call_stack.len(),
            index: 0,
        });
        let mut start = 0;
        let answer = loop {
            match run(self, start) {
                Err(MetorexError::Throw {
                    tag,
                    value: Object::Int(index),
                    ..
                }) if tag == resume_tag(serial) => start = index as usize,
                other => break other,
            }
        };
        if let Some(at) = self
            .statement_marks
            .iter()
            .rposition(|mark| mark.serial == serial)
        {
            self.statement_marks.truncate(at);
        }
        answer
    }

    /// Record that the innermost statement list is on statement `index`.
    pub(crate) fn mark_statement(&mut self, index: usize) {
        if let Some(mark) = self.statement_marks.last_mut() {
            mark.index = index;
        }
    }

    /// `__continuation_site__` — the list and the statement the code that
    /// called the method running now stands at, or nil.
    pub(crate) fn continuation_site(&self) -> Object {
        let depth = self.call_stack.len();
        match self
            .statement_marks
            .iter()
            .rev()
            .find(|mark| mark.depth < depth)
        {
            Some(mark) => Object::array(vec![
                Object::Int(mark.serial as i64),
                Object::Int(mark.index as i64),
            ]),
            None => Object::Nil,
        }
    }

    /// `__continuation_resumed__(serial, index)` — the value a continuation
    /// left for `callcc` to answer at that statement, in an Array, or nil.
    pub(crate) fn continuation_resumed(&mut self, arguments: &[Object]) -> Object {
        let (Some(Object::Int(serial)), Some(Object::Int(index))) =
            (arguments.first(), arguments.get(1))
        else {
            return Object::Nil;
        };
        match self
            .continuation_resumes
            .remove(&(*serial as u64, *index as usize))
        {
            Some(value) => Object::array(vec![value]),
            None => Object::Nil,
        }
    }

    /// `__continuation_resume__(serial, index, value)` — run the statement
    /// again with `callcc` answering `value` there, while the list it is in
    /// still runs. Answers false when the list has finished.
    pub(crate) fn continuation_resume(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let (Some(Object::Int(serial)), Some(Object::Int(index))) =
            (arguments.first(), arguments.get(1))
        else {
            return Ok(Object::Bool(false));
        };
        let serial = *serial as u64;
        if !self
            .statement_marks
            .iter()
            .any(|mark| mark.serial == serial)
        {
            return Ok(Object::Bool(false));
        }
        let value = arguments.get(2).cloned().unwrap_or(Object::Nil);
        self.continuation_resumes
            .insert((serial, *index as usize), value);
        Err(MetorexError::Throw {
            tag: resume_tag(serial),
            value: Object::Int(*index),
            location: crate::vm::utils::position_to_location(position),
        })
    }
}
