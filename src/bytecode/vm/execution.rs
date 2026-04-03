// Main execution loop for the bytecode VM

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::opcode::OpCode;
use crate::class::Class;
use crate::error::{MetorexError, SourceLocation};
use crate::object::{Instance, Method, Object};

use super::BytecodeVm;
use super::arithmetic::*;
use super::collections::*;
use super::comparison::*;
use super::types::CallFrame;

impl BytecodeVm {
    /// Main execution loop — fetch, decode, execute.
    pub(super) fn run(&mut self) -> Result<Object, MetorexError> {
        loop {
            let frame = self.current_frame_mut()?;
            let chunk_len = frame.chunk().len();

            if frame.ip >= chunk_len {
                return Ok(Object::Nil);
            }

            let op = frame.read_opcode();
            let Some(op) = op else {
                let line = self.current_frame()?.current_line();
                return Err(MetorexError::runtime_error(
                    "Invalid opcode",
                    SourceLocation::new(line, 0, 0),
                ));
            };

            match op {
                OpCode::Return => {
                    let result = self.pop().unwrap_or(Object::Nil);
                    let finished_frame = self.pop_frame()?;

                    if self.frames.is_empty() {
                        return Ok(result);
                    }

                    // Discard the finished frame's stack slots
                    self.stack.truncate(finished_frame.slot_offset);

                    // If returning from an initializer, push the instance
                    if let Some(inst) = self.pending_instance.take() {
                        self.push(Object::Instance(inst));
                    } else {
                        self.push(result);
                    }
                }

                OpCode::Constant => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let val = self.current_frame()?.chunk().get_constant(idx).clone();
                    self.push(val);
                }

                OpCode::ConstantLong => {
                    let idx = self.current_frame_mut()?.read_u16() as usize;
                    let val = self.current_frame()?.chunk().get_constant(idx).clone();
                    self.push(val);
                }

                OpCode::Nil => self.push(Object::Nil),
                OpCode::True => self.push(Object::Bool(true)),
                OpCode::False => self.push(Object::Bool(false)),
                OpCode::Pop => {
                    self.pop()?;
                }

                // ── Arithmetic ─────────────────────────────────────────
                OpCode::Add => self.binary_op(binary_add)?,
                OpCode::Subtract => self.binary_op(binary_sub)?,
                OpCode::Multiply => self.binary_op(binary_mul)?,
                OpCode::Divide => self.binary_op(binary_div)?,
                OpCode::Modulo => self.binary_op(binary_mod)?,

                OpCode::Negate => {
                    let val = self.pop()?;
                    let result = match val {
                        Object::Int(n) => Object::Int(-n),
                        Object::Float(f) => Object::Float(-f),
                        _ => {
                            return Err(self.runtime_err("Cannot negate non-numeric value"));
                        }
                    };
                    self.push(result);
                }

                OpCode::Not => {
                    let val = self.pop()?;
                    self.push(Object::Bool(val.is_falsy()));
                }

                // ── Comparison ─────────────────────────────────────────
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Object::Bool(a.equals(&b)));
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Object::Bool(!a.equals(&b)));
                }
                OpCode::Less => self.binary_op(compare_less)?,
                OpCode::Greater => self.binary_op(compare_greater)?,
                OpCode::LessEqual => self.binary_op(compare_less_equal)?,
                OpCode::GreaterEqual => self.binary_op(compare_greater_equal)?,

                // ── Variables ──────────────────────────────────────────
                OpCode::GetLocal => {
                    let slot = self.current_frame_mut()?.read_byte() as usize;
                    let offset = self.current_frame()?.slot_offset;
                    let val = self.stack[offset + slot].clone();
                    self.push(val);
                }

                OpCode::SetLocal => {
                    let slot = self.current_frame_mut()?.read_byte() as usize;
                    let offset = self.current_frame()?.slot_offset;
                    let val = self.peek(0)?.clone();
                    self.stack[offset + slot] = val;
                }

                OpCode::GetGlobal => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(idx)?;
                    match self.globals.get(&name) {
                        Some(val) => {
                            let val = val.clone();
                            self.push(val);
                        }
                        None => {
                            return Err(self.runtime_err(&format!("Undefined variable '{}'", name)));
                        }
                    }
                }

                OpCode::SetGlobal => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(idx)?;
                    let val = self.peek(0)?.clone();
                    if !self.set_global(&name, val) {
                        return Err(self.runtime_err(&format!("Undefined variable '{}'", name)));
                    }
                }

                OpCode::DefineGlobal => {
                    let idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(idx)?;
                    let val = self.pop()?;
                    self.define_global(name, val);
                }

                // ── Control flow ───────────────────────────────────────
                OpCode::Jump => {
                    let offset = self.current_frame_mut()?.read_u16() as usize;
                    self.current_frame_mut()?.ip += offset;
                }

                OpCode::JumpIfFalse => {
                    let offset = self.current_frame_mut()?.read_u16() as usize;
                    if self.peek(0)?.is_falsy() {
                        self.current_frame_mut()?.ip += offset;
                    }
                }

                OpCode::Loop => {
                    let offset = self.current_frame_mut()?.read_u16() as usize;
                    self.current_frame_mut()?.ip -= offset;
                }

                // ── Function calls ─────────────────────────────────────
                OpCode::Call => {
                    let arg_count = self.current_frame_mut()?.read_byte() as usize;
                    let callee_idx = self.stack.len() - 1 - arg_count;
                    let callee = self.stack[callee_idx].clone();
                    match callee {
                        Object::CompiledFunction(func) => {
                            if func.arity as usize != arg_count {
                                return Err(self.runtime_err(&format!(
                                    "Expected {} arguments but got {}",
                                    func.arity, arg_count
                                )));
                            }
                            // Look up closure upvalues by function pointer address
                            let key = Rc::as_ptr(&func) as usize;
                            let mut frame = CallFrame::new(func, callee_idx + 1);
                            if let Some(upvalues) = self.closure_upvalues.get(&key) {
                                frame.upvalues = upvalues.clone();
                            }
                            self.push_frame(frame);
                        }
                        Object::NativeFunction(name) => {
                            let args: Vec<Object> = self.stack.drain(callee_idx + 1..).collect();
                            let result = self.call_native(&name, &args)?;
                            self.stack.truncate(callee_idx);
                            self.push(result);
                        }
                        Object::Class(class) => {
                            // Calling a class as a function creates an instance
                            let args: Vec<Object> = self.stack.drain(callee_idx + 1..).collect();
                            let instance = Instance::new(Rc::clone(&class));
                            let inst_rc = Rc::new(RefCell::new(instance));

                            // Call initialize if it exists
                            let init_key = format!("{}#initialize", class.name());
                            if let Some(Object::CompiledFunction(init_func)) =
                                self.globals.get(&init_key).cloned()
                            {
                                // Push instance as receiver, then args
                                self.stack.truncate(callee_idx);
                                self.push(Object::Instance(Rc::clone(&inst_rc)));
                                for arg in &args {
                                    self.push(arg.clone());
                                }
                                let slot = callee_idx;
                                let frame = CallFrame::new(init_func, slot);
                                self.push_frame(frame);
                                // After init returns, the instance will be on the stack
                                // We store it so Return can put it back
                                self.pending_instance = Some(inst_rc);
                            } else {
                                self.stack.truncate(callee_idx);
                                self.push(Object::Instance(inst_rc));
                            }
                        }
                        _ => {
                            return Err(self.runtime_err("Can only call functions and classes"));
                        }
                    }
                }

                // ── Collections ────────────────────────────────────────
                OpCode::Array => {
                    let count = self.current_frame_mut()?.read_byte() as usize;
                    let start = self.stack.len() - count;
                    let elements: Vec<Object> = self.stack.drain(start..).collect();
                    self.push(Object::Array(Rc::new(std::cell::RefCell::new(elements))));
                }

                OpCode::Hash => {
                    let count = self.current_frame_mut()?.read_byte() as usize;
                    let pair_count = count * 2;
                    let start = self.stack.len() - pair_count;
                    let items: Vec<Object> = self.stack.drain(start..).collect();
                    let mut map = HashMap::new();
                    for pair in items.chunks(2) {
                        let key = match &pair[0] {
                            Object::String(s) => s.to_string(),
                            other => other.to_string(),
                        };
                        map.insert(key, pair[1].clone());
                    }
                    self.push(Object::Dict(Rc::new(std::cell::RefCell::new(map))));
                }

                OpCode::IndexGet => {
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    let result = index_get(&collection, &index)?;
                    self.push(result);
                }

                OpCode::IndexSet => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    index_set(&collection, &index, &value)?;
                    self.push(value);
                }

                // ── Closures and upvalues (13.8) ─────────────────────
                OpCode::Closure => {
                    let uv_count = self.current_frame_mut()?.read_byte() as usize;
                    // The function is on top of the stack
                    let func_val = self.peek(0)?.clone();
                    let func = match func_val {
                        Object::CompiledFunction(f) => f,
                        _ => return Err(self.runtime_err("Expected function for closure")),
                    };

                    let mut upvalues = Vec::with_capacity(uv_count);
                    for _ in 0..uv_count {
                        let is_local = self.current_frame_mut()?.read_byte() != 0;
                        let index = self.current_frame_mut()?.read_byte() as usize;

                        if is_local {
                            // Capture a local from the enclosing frame
                            let slot = self.current_frame()?.slot_offset + index;
                            let uv = self.capture_upvalue(slot);
                            upvalues.push(uv);
                        } else {
                            // Forward an upvalue from the enclosing closure
                            let enclosing_uv = self.current_frame()?.upvalues.get(index).cloned();
                            match enclosing_uv {
                                Some(uv) => upvalues.push(uv),
                                None => {
                                    return Err(self.runtime_err("Invalid upvalue index"));
                                }
                            }
                        }
                    }

                    // Store the upvalues keyed by the Rc pointer address of the function.
                    // When this function is later called (even after being stored in a
                    // global and retrieved), the upvalues are looked up by this key.
                    let key = Rc::as_ptr(&func) as usize;
                    self.closure_upvalues.insert(key, upvalues);
                }

                OpCode::GetUpvalue => {
                    let index = self.current_frame_mut()?.read_byte() as usize;
                    let val = {
                        let frame = self.current_frame()?;
                        match frame.upvalues.get(index) {
                            Some(uv) => uv.borrow().value.borrow().clone(),
                            None => {
                                return Err(self.runtime_err("Invalid upvalue index"));
                            }
                        }
                    };
                    self.push(val);
                }

                OpCode::SetUpvalue => {
                    let index = self.current_frame_mut()?.read_byte() as usize;
                    let val = self.peek(0)?.clone();
                    let frame = self.current_frame()?;
                    match frame.upvalues.get(index) {
                        Some(uv) => {
                            *uv.borrow().value.borrow_mut() = val;
                        }
                        None => {
                            return Err(self.runtime_err("Invalid upvalue index"));
                        }
                    }
                }

                OpCode::CloseUpvalue => {
                    let stack_top = self.stack.len() - 1;
                    self.close_upvalues(stack_top);
                    self.pop()?;
                }

                // ── Classes and objects (13.9) ─────────────────────────
                OpCode::Class => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    let class = Class::new(name, None);
                    self.push(Object::Class(Rc::new(class)));
                }

                OpCode::Method => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    // Stack: [..., class, compiled_function]
                    let func_val = self.pop()?;
                    let class_val = self.peek(0)?;
                    match (class_val, func_val) {
                        (Object::Class(class), Object::CompiledFunction(func)) => {
                            let method = Method::new(name.clone(), vec![], vec![]);
                            // Store the compiled function in the method's closure vars
                            // so the VM can look it up when calling
                            let mut m = method;
                            m.owner = Some(class.name().to_string());
                            let method_rc = Rc::new(m);
                            class.define_method(&name, method_rc);
                            // Also store the compiled function for bytecode execution
                            let key = format!("{}#{}", class.name(), name);
                            self.define_global(key, Object::CompiledFunction(func));
                        }
                        _ => {
                            return Err(
                                self.runtime_err("OP_METHOD requires class and function on stack")
                            );
                        }
                    }
                }

                OpCode::Invoke => {
                    let operand = self.current_frame_mut()?.read_u16();
                    let name_idx = (operand >> 8) as usize;
                    let arg_count = (operand & 0xFF) as usize;
                    let method_name = self.read_string_constant(name_idx)?;

                    // The receiver is below the arguments on the stack
                    let receiver = self.peek(arg_count)?.clone();

                    match &receiver {
                        Object::Instance(inst) => {
                            let class_name = inst.borrow().class_name().to_string();
                            let key = format!("{}#{}", class_name, method_name);
                            if let Some(Object::CompiledFunction(func)) =
                                self.globals.get(&key).cloned()
                            {
                                if func.arity as usize != arg_count {
                                    return Err(self.runtime_err(&format!(
                                        "Expected {} arguments but got {}",
                                        func.arity, arg_count
                                    )));
                                }
                                let callee_idx = self.stack.len() - 1 - arg_count;
                                let frame = CallFrame::new(func, callee_idx + 1);
                                self.push_frame(frame);
                            } else {
                                return Err(self.runtime_err(&format!(
                                    "Undefined method '{}' for {}",
                                    method_name, class_name
                                )));
                            }
                        }
                        Object::Class(class) => {
                            // Calling ClassName() — instantiation
                            if method_name == "new" || method_name == class.name() {
                                let instance = Instance::new(Rc::clone(class));
                                // Pop receiver + args, push instance
                                for _ in 0..arg_count {
                                    self.pop()?;
                                }
                                // Pop the class (receiver)
                                let receiver_idx = self.stack.len() - 1;
                                self.stack[receiver_idx] =
                                    Object::Instance(Rc::new(RefCell::new(instance)));
                            } else {
                                return Err(self.runtime_err(&format!(
                                    "Undefined class method '{}'",
                                    method_name
                                )));
                            }
                        }
                        _ => {
                            // For non-instance receivers, try native methods
                            // (length, push, etc.) — return an error for now
                            return Err(self.runtime_err(&format!(
                                "Cannot call method '{}' on {}",
                                method_name,
                                receiver.type_name()
                            )));
                        }
                    }
                }

                OpCode::GetInstance => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    // 'self' is the first local in the current frame (slot 0)
                    let offset = self.current_frame()?.slot_offset;
                    if offset < self.stack.len() {
                        let receiver = self.stack[offset].clone();
                        match receiver {
                            Object::Instance(inst) => {
                                let val =
                                    inst.borrow().get_var(&name).cloned().unwrap_or(Object::Nil);
                                self.push(val);
                            }
                            _ => {
                                return Err(self.runtime_err(
                                    "Cannot access instance variable outside of instance",
                                ));
                            }
                        }
                    } else {
                        self.push(Object::Nil);
                    }
                }

                OpCode::SetInstance => {
                    let name_idx = self.current_frame_mut()?.read_byte() as usize;
                    let name = self.read_string_constant(name_idx)?;
                    let value = self.peek(0)?.clone();
                    let offset = self.current_frame()?.slot_offset;
                    if offset < self.stack.len() {
                        let receiver = self.stack[offset].clone();
                        match receiver {
                            Object::Instance(inst) => {
                                inst.borrow_mut().set_var(name, value);
                            }
                            _ => {
                                return Err(self.runtime_err(
                                    "Cannot set instance variable outside of instance",
                                ));
                            }
                        }
                    }
                }

                // ── Exception handling (placeholder) ──────────────────
                OpCode::Try => {
                    let _operand = self.current_frame_mut()?.read_u16();
                }
                OpCode::Catch | OpCode::Finally | OpCode::EndTry | OpCode::Raise => {}
                OpCode::Match => {}
                OpCode::MatchPattern => {
                    let _operand = self.current_frame_mut()?.read_u16();
                }
            }
        }
    }
}
