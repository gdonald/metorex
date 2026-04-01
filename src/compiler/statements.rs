// Statement compilation: AST Statement nodes → bytecode

use crate::ast::{Expression, Statement};
use crate::bytecode::opcode::OpCode;
use crate::error::{MetorexError, SourceLocation};
use crate::object::Object;

use super::Compiler;

impl Compiler {
    /// Compile a single statement.
    pub fn compile_statement(&mut self, stmt: &Statement) -> Result<(), MetorexError> {
        match stmt {
            Statement::Expression { expression, .. } => {
                self.compile_expression(expression)?;
                self.emit_op(OpCode::Pop, 0);
                Ok(())
            }

            Statement::Assignment {
                target,
                value,
                position,
            } => {
                let line = Self::pos_line(position);
                self.compile_expression(value)?;

                match target {
                    Expression::Identifier { name, .. } => {
                        if let Some(slot) = self.resolve_local(name) {
                            self.emit_op_u8(OpCode::SetLocal, slot, line);
                        } else if self.scope_depth > 0 {
                            // New local variable in a local scope
                            self.add_local(name.clone());
                            self.mark_initialized();
                            // Value is already on the stack in the right slot
                        } else {
                            let idx = self.identifier_constant(name)?;
                            self.emit_op_u8(OpCode::DefineGlobal, idx, line);
                        }
                    }
                    Expression::InstanceVariable { name, .. } => {
                        let idx = self.identifier_constant(name)?;
                        self.emit_op_u8(OpCode::SetInstance, idx, line);
                    }
                    Expression::Index { array, index, .. } => {
                        // Compile: array[index] = value
                        // Stack: value is already on stack from above
                        // We need: collection, index, value → IndexSet
                        // So compile collection and index, then the value is under them.
                        // Reorder: emit collection, index, then re-emit value.
                        // Simplest approach: pop value, compile target parts, push value back.
                        // For now, compile as: collection, index, value, IndexSet
                        // This means we need to re-order. Use a simpler approach:
                        // compile collection, index, value in order.
                        self.emit_op(OpCode::Pop, line); // pop value temporarily
                        self.compile_expression(array)?;
                        self.compile_expression(index)?;
                        self.compile_expression(value)?; // recompile value
                        self.emit_op(OpCode::IndexSet, line);
                    }
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "Invalid assignment target",
                            SourceLocation::new(line, 0, 0),
                        ));
                    }
                }
                Ok(())
            }

            Statement::Return {
                value, position, ..
            } => {
                let line = Self::pos_line(position);
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                } else {
                    self.emit_op(OpCode::Nil, line);
                }
                self.emit_op(OpCode::Return, line);
                Ok(())
            }

            Statement::Block {
                statements,
                position,
            } => {
                let line = Self::pos_line(position);
                self.begin_scope();
                for s in statements {
                    self.compile_statement(s)?;
                }
                self.end_scope(line);
                Ok(())
            }

            // ── Control flow (12.4) ────────────────────────────────────
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position,
            } => {
                let line = Self::pos_line(position);

                // Compile condition
                self.compile_expression(condition)?;
                // Jump past then-branch if false
                let then_jump = self.emit_jump(OpCode::JumpIfFalse, line);
                self.emit_op(OpCode::Pop, line); // pop condition (truthy path)

                // Compile then-branch in its own scope
                self.begin_scope();
                for s in then_branch {
                    self.compile_statement(s)?;
                }
                self.end_scope(line);

                // Jump over else/elsif after then-branch executes
                let mut end_jumps = vec![self.emit_jump(OpCode::Jump, line)];

                // Patch the then_jump to land here (start of elsif/else chain)
                self.patch_jump(then_jump);
                self.emit_op(OpCode::Pop, line); // pop condition (falsy path)

                // Compile elsif branches
                for elsif in elsif_branches {
                    let elsif_line = Self::pos_line(&elsif.position);
                    self.compile_expression(&elsif.condition)?;
                    let elsif_jump = self.emit_jump(OpCode::JumpIfFalse, elsif_line);
                    self.emit_op(OpCode::Pop, elsif_line);

                    self.begin_scope();
                    for s in &elsif.body {
                        self.compile_statement(s)?;
                    }
                    self.end_scope(elsif_line);

                    end_jumps.push(self.emit_jump(OpCode::Jump, elsif_line));

                    self.patch_jump(elsif_jump);
                    self.emit_op(OpCode::Pop, elsif_line);
                }

                // Compile else branch
                if let Some(else_body) = else_branch {
                    self.begin_scope();
                    for s in else_body {
                        self.compile_statement(s)?;
                    }
                    self.end_scope(line);
                }

                // Patch all end_jumps to land here
                for ej in end_jumps {
                    self.patch_jump(ej);
                }

                Ok(())
            }

            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                position,
            } => {
                let line = Self::pos_line(position);

                // Unless is if-not: jump to then-branch when condition is false
                self.compile_expression(condition)?;
                let then_jump = self.emit_jump(OpCode::JumpIfFalse, line);
                self.emit_op(OpCode::Pop, line); // pop condition (truthy → skip)

                // Truthy path: execute else_branch (if any), then jump to end
                if let Some(else_body) = else_branch {
                    self.begin_scope();
                    for s in else_body {
                        self.compile_statement(s)?;
                    }
                    self.end_scope(line);
                }

                let end_jump = self.emit_jump(OpCode::Jump, line);

                // Falsy path: execute then_branch
                self.patch_jump(then_jump);
                self.emit_op(OpCode::Pop, line);

                self.begin_scope();
                for s in then_branch {
                    self.compile_statement(s)?;
                }
                self.end_scope(line);

                self.patch_jump(end_jump);

                Ok(())
            }

            Statement::While {
                condition,
                body,
                position,
            } => {
                let line = Self::pos_line(position);

                let loop_start = self.chunk.len();
                self.begin_loop(loop_start);

                // Compile condition
                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump(OpCode::JumpIfFalse, line);
                self.emit_op(OpCode::Pop, line); // pop condition (truthy)

                // Compile body in its own scope
                self.begin_scope();
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope(line);

                // Loop back to condition
                self.emit_loop(loop_start, line);

                // Patch exit jump
                self.patch_jump(exit_jump);
                self.emit_op(OpCode::Pop, line); // pop condition (falsy)

                // Patch all break jumps to land here
                let ctx = self.end_loop();
                for bj in ctx.break_jumps {
                    self.patch_jump(bj);
                }

                Ok(())
            }

            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => {
                // For loops desugar to: compile iterable, then loop with
                // an index counter. For simplicity we emit:
                //   iterable → stack
                //   i = 0 → stack
                //   loop_start:
                //     if i >= iterable.length then jump exit
                //     variable = iterable[i]
                //     body...
                //     i += 1
                //     jump loop_start
                //   exit:
                //     pop i, pop iterable
                let line = Self::pos_line(position);

                self.begin_scope();

                // Push iterable onto stack as a local
                self.compile_expression(iterable)?;
                self.add_local("__iter__".to_string());
                self.mark_initialized();
                let iter_slot = self.resolve_local("__iter__").unwrap();

                // Push index (0) onto stack as a local
                self.emit_constant(Object::Int(0), line)?;
                self.add_local("__idx__".to_string());
                self.mark_initialized();
                let idx_slot = self.resolve_local("__idx__").unwrap();

                let loop_start = self.chunk.len();
                self.begin_loop(loop_start);

                // Condition: idx < iterable.length
                self.emit_op_u8(OpCode::GetLocal, idx_slot, line);
                self.emit_op_u8(OpCode::GetLocal, iter_slot, line);
                // Encode length method call: name_idx << 8 | arg_count
                let length_idx = self.identifier_constant("length")?;
                let invoke_operand = (length_idx as u16) << 8;
                self.emit_op_u16(OpCode::Invoke, invoke_operand, line);
                self.emit_op(OpCode::Less, line);

                let exit_jump = self.emit_jump(OpCode::JumpIfFalse, line);
                self.emit_op(OpCode::Pop, line); // pop condition (truthy)

                // variable = iterable[idx]
                self.begin_scope();
                self.emit_op_u8(OpCode::GetLocal, iter_slot, line);
                self.emit_op_u8(OpCode::GetLocal, idx_slot, line);
                self.emit_op(OpCode::IndexGet, line);
                self.add_local(variable.clone());
                self.mark_initialized();

                // Compile body
                for s in body {
                    self.compile_statement(s)?;
                }
                self.end_scope(line);

                // idx += 1
                self.emit_op_u8(OpCode::GetLocal, idx_slot, line);
                self.emit_constant(Object::Int(1), line)?;
                self.emit_op(OpCode::Add, line);
                self.emit_op_u8(OpCode::SetLocal, idx_slot, line);
                self.emit_op(OpCode::Pop, line); // pop SetLocal result

                // Loop back
                self.emit_loop(loop_start, line);

                // Patch exit
                self.patch_jump(exit_jump);
                self.emit_op(OpCode::Pop, line); // pop condition (falsy)

                // Patch break jumps
                let ctx = self.end_loop();
                for bj in ctx.break_jumps {
                    self.patch_jump(bj);
                }

                self.end_scope(line); // pop __iter__ and __idx__

                Ok(())
            }

            Statement::Break { position } => {
                let line = Self::pos_line(position);
                if self.loop_stack.is_empty() {
                    return Err(MetorexError::runtime_error(
                        "break outside of loop",
                        SourceLocation::new(line, 0, 0),
                    ));
                }
                // Pop locals down to loop scope
                self.emit_pops_to_loop_scope(line);
                // Emit jump placeholder — will be patched at loop end
                let jump = self.emit_jump(OpCode::Jump, line);
                self.loop_stack.last_mut().unwrap().break_jumps.push(jump);
                Ok(())
            }

            Statement::Continue { position } => {
                let line = Self::pos_line(position);
                if self.loop_stack.is_empty() {
                    return Err(MetorexError::runtime_error(
                        "continue outside of loop",
                        SourceLocation::new(line, 0, 0),
                    ));
                }
                // Pop locals down to loop scope
                self.emit_pops_to_loop_scope(line);
                // Jump back to loop start
                let loop_start = self.loop_stack.last().unwrap().start;
                self.emit_loop(loop_start, line);
                Ok(())
            }

            // Remaining statement types are stubs for now — will be expanded
            // in sections 12.5-12.8
            _ => {
                // For unimplemented statement types, emit nothing but don't error
                // so partial compilation can proceed
                Ok(())
            }
        }
    }
}
