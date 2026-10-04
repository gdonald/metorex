//! Calling a method a C extension defined, with the arguments laid out the
//! way its arity says the C function takes them.

use super::handles::{QNIL, Value, to_object, to_value};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;

/// The C function behind a method, and the arity `rb_define_method` gave it:
/// a count of fixed arguments, -1 for `(argc, argv, self)`, or -2 for
/// `(self, args)` with the arguments in an Array. An enumerator's size
/// function carries `ENUMERATOR_SIZE_ARITY` instead, and a TracePoint's hook
/// `TRACEPOINT_HOOK_ARITY` and a block function `BLOCK_FUNCTION_ARITY`, each
/// with the pointer or VALUE C asked to have handed back to it as `data`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CFunction {
    pub address: usize,
    pub arity: i32,
    pub data: usize,
}

/// The most fixed arguments a C method can take.
pub(crate) const MAX_FIXED_ARITY: i32 = 15;

/// The arity of a size function `rb_enumeratorize_with_size` was handed,
/// which takes the object walked, the Array of arguments it is walked with,
/// and the enumerator. `rb_define_method` refuses it, so no method has it.
pub(crate) const ENUMERATOR_SIZE_ARITY: i32 = i32::MIN;

/// The arity of a hook `rb_tracepoint_new` was handed, which takes the
/// TracePoint and the data pointer and answers nothing.
pub(crate) const TRACEPOINT_HOOK_ARITY: i32 = i32::MIN + 1;

/// The arity of a block function, `RB_BLOCK_CALL_FUNC_ARGLIST`, which takes
/// the first value yielded, the VALUE C asked to have handed back to it as
/// `data`, the count and pointer of every value yielded, and the block.
pub(crate) const BLOCK_FUNCTION_ARITY: i32 = i32::MIN + 2;

/// The arity of the function `rb_thread_create` was handed, which takes the
/// pointer C asked to have handed back to it as `data` and answers the
/// thread's value.
pub(crate) const THREAD_FUNCTION_ARITY: i32 = i32::MIN + 3;

macro_rules! call_fixed {
    ($address:expr, $receiver:expr, $values:expr, $($index:literal)*) => {{
        // SAFETY: `rb_define_method` was handed a function taking self and
        // this many VALUEs, and the arity was checked against that count.
        let function: extern "C-unwind" fn(Value $(, call_fixed!(@value $index))*) -> Value =
            unsafe { std::mem::transmute($address) };
        function($receiver $(, $values[$index])*)
    }};
    (@value $index:literal) => { Value };
}

impl CFunction {
    fn call_with_values(self, receiver: Value, values: &[Value], block_argument: Value) -> Value {
        let address = self.address as *const ();
        match self.arity {
            BLOCK_FUNCTION_ARITY => {
                // SAFETY: a block function takes the first value yielded, the
                // data VALUE, the count and pointer of the values yielded,
                // and the block.
                let function: extern "C-unwind" fn(
                    Value,
                    Value,
                    i32,
                    *const Value,
                    Value,
                ) -> Value = unsafe { std::mem::transmute(address) };
                let first = values.first().copied().unwrap_or(super::handles::QNIL);
                function(
                    first,
                    self.data,
                    values.len() as i32,
                    values.as_ptr(),
                    block_argument,
                )
            }
            TRACEPOINT_HOOK_ARITY => {
                // SAFETY: a TracePoint hook takes the TracePoint and the data
                // pointer C handed `rb_tracepoint_new`.
                let function: extern "C-unwind" fn(Value, usize) =
                    unsafe { std::mem::transmute(address) };
                function(values[0], self.data);
                super::handles::QNIL
            }
            THREAD_FUNCTION_ARITY => {
                // SAFETY: a thread function takes the data pointer C handed
                // `rb_thread_create`.
                let function: extern "C-unwind" fn(usize) -> Value =
                    unsafe { std::mem::transmute(address) };
                function(self.data)
            }
            ENUMERATOR_SIZE_ARITY => {
                // SAFETY: a size function takes the object walked, its
                // arguments, and the enumerator.
                let function: extern "C-unwind" fn(Value, Value, Value) -> Value =
                    unsafe { std::mem::transmute(address) };
                function(values[0], values[1], values[2])
            }
            -2 => {
                let arguments: Vec<Object> = values.iter().map(|held| to_object(*held)).collect();
                // SAFETY: arity -2 is a function of self and an Array.
                let function: extern "C-unwind" fn(Value, Value) -> Value =
                    unsafe { std::mem::transmute(address) };
                function(receiver, to_value(&Object::array(arguments)))
            }
            -1 => {
                // SAFETY: arity -1 is a function of a count, a pointer to
                // that many VALUEs, and self.
                let function: extern "C-unwind" fn(i32, *const Value, Value) -> Value =
                    unsafe { std::mem::transmute(address) };
                function(values.len() as i32, values.as_ptr(), receiver)
            }
            0 => call_fixed!(address, receiver, values,),
            1 => call_fixed!(address, receiver, values, 0),
            2 => call_fixed!(address, receiver, values, 0 1),
            3 => call_fixed!(address, receiver, values, 0 1 2),
            4 => call_fixed!(address, receiver, values, 0 1 2 3),
            5 => call_fixed!(address, receiver, values, 0 1 2 3 4),
            6 => call_fixed!(address, receiver, values, 0 1 2 3 4 5),
            7 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6),
            8 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7),
            9 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7 8),
            10 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7 8 9),
            11 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7 8 9 10),
            12 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7 8 9 10 11),
            13 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7 8 9 10 11 12),
            14 => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7 8 9 10 11 12 13),
            _ => call_fixed!(address, receiver, values, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14),
        }
    }
}

impl VirtualMachine {
    /// Runs the C function behind a method on `receiver`. `owner` and
    /// `name` say which method it was called as, for `rb_call_super`.
    pub(crate) fn call_c_function(
        &mut self,
        function: CFunction,
        owner: std::rc::Rc<crate::class::Class>,
        name: String,
        receiver: Object,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if function.arity >= 0 && arguments.len() != function.arity as usize {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!(
                    "wrong number of arguments (given {}, expected {})",
                    arguments.len(),
                    function.arity
                ),
                position,
            ));
        }
        let arguments = if function.arity == ENUMERATOR_SIZE_ARITY {
            self.size_function_arguments(receiver.clone(), position)?
        } else {
            arguments
        };
        let (arguments, keywords_given) = without_keyword_marker(arguments);
        let method = super::RunningMethod {
            receiver: receiver.clone(),
            name,
            owner,
        };
        let receiver = to_value(&receiver);
        let values: Vec<Value> = arguments.iter().map(to_value).collect();
        // A block function is handed the block its Proc was called with as
        // an argument, and `rb_block_given_p` in it speaks for the method
        // that made the Proc rather than for that call.
        let block = self.pending_block.take();
        let block_from_ampersand = self.pending_block_from_ampersand;
        let (block, block_argument) = if function.arity == BLOCK_FUNCTION_ARITY {
            (
                super::calls::block_of_block_call(function),
                block.as_ref().map_or(QNIL, to_value),
            )
        } else {
            (block, QNIL)
        };
        let caller = super::Caller {
            position,
            block,
            block_from_ampersand,
            keywords_given,
            method: Some(method),
        };
        let answered = super::enter(self, caller, || {
            function.call_with_values(receiver, &values, block_argument)
        })?;
        Ok(to_object(answered))
    }
}

/// The arguments with the mark metorex puts on a keyword Hash taken off, so
/// C sees a plain Hash, and whether there was one.
fn without_keyword_marker(mut arguments: Vec<Object>) -> (Vec<Object>, bool) {
    let marked = matches!(
        arguments.last(),
        Some(Object::Dict(pairs)) if pairs.borrow().contains_key("__MX_KWARGS__")
    );
    if !marked {
        return (arguments, false);
    }
    if let Some(Object::Dict(pairs)) = arguments.pop() {
        let mut plain = pairs.borrow().clone();
        plain.shift_remove("__MX_KWARGS__");
        arguments.push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
            plain,
        ))));
    }
    (arguments, true)
}
