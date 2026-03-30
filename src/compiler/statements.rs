// Statement compilation: AST Statement nodes → bytecode

use crate::ast::{Expression, Statement};
use crate::bytecode::opcode::OpCode;
use crate::error::{MetorexError, SourceLocation};

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

            // Remaining statement types are stubs for now — will be expanded
            // in sections 12.3-12.8
            _ => {
                // For unimplemented statement types, emit nothing but don't error
                // so partial compilation can proceed
                Ok(())
            }
        }
    }
}
