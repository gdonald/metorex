// Expression compilation: AST Expression nodes → bytecode

use crate::ast::{BinaryOp, Expression, InterpolationPart, UnaryOp};
use crate::bytecode::opcode::OpCode;
use crate::error::{MetorexError, SourceLocation};
use crate::object::{CompiledFunction, Object};
use std::rc::Rc;

use super::Compiler;

impl Compiler {
    /// Compile an expression, leaving its result on the stack.
    pub fn compile_expression(&mut self, expr: &Expression) -> Result<(), MetorexError> {
        match expr {
            // ── Literals ────────────────────────────────────────────────
            Expression::IntLiteral { value, position } => {
                let line = Self::pos_line(position);
                self.emit_constant(Object::Int(*value), line)
            }

            Expression::FloatLiteral { value, position } => {
                let line = Self::pos_line(position);
                self.emit_constant(Object::Float(*value), line)
            }

            Expression::StringLiteral { value, position } => {
                let line = Self::pos_line(position);
                self.emit_constant(Object::String(Rc::new(value.clone())), line)
            }

            Expression::BoolLiteral { value, position } => {
                let line = Self::pos_line(position);
                if *value {
                    self.emit_op(OpCode::True, line);
                } else {
                    self.emit_op(OpCode::False, line);
                }
                Ok(())
            }

            Expression::NilLiteral { position } => {
                self.emit_op(OpCode::Nil, Self::pos_line(position));
                Ok(())
            }

            Expression::Symbol { value, position } => {
                let line = Self::pos_line(position);
                self.emit_constant(Object::Symbol(Rc::new(value.clone())), line)
            }

            // ── String interpolation ────────────────────────────────────
            Expression::InterpolatedString { parts, position } => {
                let line = Self::pos_line(position);
                if parts.is_empty() {
                    return self.emit_constant(Object::String(Rc::new(String::new())), line);
                }

                // Compile each part, then concatenate with Add
                let mut first = true;
                for part in parts {
                    match part {
                        InterpolationPart::Text(text) => {
                            self.emit_constant(Object::String(Rc::new(text.clone())), line)?;
                        }
                        InterpolationPart::Expression(expr) => {
                            self.compile_expression(expr)?;
                        }
                    }
                    if first {
                        first = false;
                    } else {
                        self.emit_op(OpCode::Add, line);
                    }
                }
                Ok(())
            }

            // ── Variable access ─────────────────────────────────────────
            Expression::Identifier { name, position } => {
                let line = Self::pos_line(position);
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_op_u8(OpCode::GetLocal, slot, line);
                } else if let Some(uv) = self.resolve_upvalue(name) {
                    self.emit_op_u8(OpCode::GetUpvalue, uv, line);
                } else {
                    let idx = self.identifier_constant(name)?;
                    self.emit_op_u8(OpCode::GetGlobal, idx, line);
                }
                Ok(())
            }

            Expression::InstanceVariable { name, position } => {
                let line = Self::pos_line(position);
                let idx = self.identifier_constant(name)?;
                self.emit_op_u8(OpCode::GetInstance, idx, line);
                Ok(())
            }

            Expression::GlobalVariable { name, position } => {
                let line = Self::pos_line(position);
                let idx = self.identifier_constant(name)?;
                self.emit_op_u8(OpCode::GetGlobal, idx, line);
                Ok(())
            }

            Expression::ClassVariable { name, position } => {
                // Treat class variables like globals for now
                let line = Self::pos_line(position);
                let idx = self.identifier_constant(name)?;
                self.emit_op_u8(OpCode::GetGlobal, idx, line);
                Ok(())
            }

            // ── Binary operations ───────────────────────────────────────
            Expression::BinaryOp {
                op,
                left,
                right,
                position,
            } => {
                let line = Self::pos_line(position);

                // Short-circuit for && and ||
                match op {
                    BinaryOp::And => {
                        self.compile_expression(left)?;
                        let jump = self.emit_jump(OpCode::JumpIfFalse, line);
                        self.emit_op(OpCode::Pop, line);
                        self.compile_expression(right)?;
                        self.patch_jump(jump);
                        return Ok(());
                    }
                    BinaryOp::Or => {
                        self.compile_expression(left)?;
                        let else_jump = self.emit_jump(OpCode::JumpIfFalse, line);
                        let end_jump = self.emit_jump(OpCode::Jump, line);
                        self.patch_jump(else_jump);
                        self.emit_op(OpCode::Pop, line);
                        self.compile_expression(right)?;
                        self.patch_jump(end_jump);
                        return Ok(());
                    }
                    _ => {}
                }

                self.compile_expression(left)?;
                self.compile_expression(right)?;

                match op {
                    BinaryOp::Add => self.emit_op(OpCode::Add, line),
                    BinaryOp::Subtract => self.emit_op(OpCode::Subtract, line),
                    BinaryOp::Multiply => self.emit_op(OpCode::Multiply, line),
                    BinaryOp::Divide => self.emit_op(OpCode::Divide, line),
                    BinaryOp::Modulo => self.emit_op(OpCode::Modulo, line),
                    BinaryOp::Equal | BinaryOp::CaseEqual => self.emit_op(OpCode::Equal, line),
                    BinaryOp::NotEqual => self.emit_op(OpCode::NotEqual, line),
                    BinaryOp::Less => self.emit_op(OpCode::Less, line),
                    BinaryOp::Greater => self.emit_op(OpCode::Greater, line),
                    BinaryOp::LessEqual => self.emit_op(OpCode::LessEqual, line),
                    BinaryOp::GreaterEqual => self.emit_op(OpCode::GreaterEqual, line),
                    BinaryOp::Spaceship
                    | BinaryOp::Xor
                    | BinaryOp::Power
                    | BinaryOp::BitwiseAnd
                    | BinaryOp::BitwiseOr => {
                        return Err(MetorexError::runtime_error(
                            format!("{} operator not yet supported in bytecode compiler", op),
                            SourceLocation::new(line, 0, 0),
                        ));
                    }
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                    BinaryOp::Assign
                    | BinaryOp::AddAssign
                    | BinaryOp::SubtractAssign
                    | BinaryOp::MultiplyAssign
                    | BinaryOp::DivideAssign => {
                        return Err(MetorexError::runtime_error(
                            "Assignment operators should be handled as statements",
                            SourceLocation::new(line, 0, 0),
                        ));
                    }
                }
                Ok(())
            }

            // ── Unary operations ────────────────────────────────────────
            Expression::UnaryOp {
                op,
                operand,
                position,
            } => {
                let line = Self::pos_line(position);
                self.compile_expression(operand)?;
                match op {
                    UnaryOp::Minus => self.emit_op(OpCode::Negate, line),
                    UnaryOp::Not => self.emit_op(OpCode::Not, line),
                    UnaryOp::Plus => {} // no-op
                }
                Ok(())
            }

            // ── Grouping ────────────────────────────────────────────────
            Expression::Grouped { expression, .. } => self.compile_expression(expression),

            // ── Array literal ───────────────────────────────────────────
            Expression::Array { elements, position } => {
                let line = Self::pos_line(position);
                for elem in elements {
                    self.compile_expression(elem)?;
                }
                let count = elements.len().min(255) as u8;
                self.emit_op_u8(OpCode::Array, count, line);
                Ok(())
            }

            // ── Dictionary literal ──────────────────────────────────────
            Expression::Dictionary { entries, position } => {
                let line = Self::pos_line(position);
                for (key, value) in entries {
                    self.compile_expression(key)?;
                    self.compile_expression(value)?;
                }
                let count = entries.len().min(255) as u8;
                self.emit_op_u8(OpCode::Hash, count, line);
                Ok(())
            }

            // ── Index access ────────────────────────────────────────────
            Expression::Index {
                array,
                index,
                position,
            } => {
                let line = Self::pos_line(position);
                self.compile_expression(array)?;
                self.compile_expression(index)?;
                self.emit_op(OpCode::IndexGet, line);
                Ok(())
            }

            // ── Method call (receiver.method(args)) ─────────────────────
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                position,
                ..
            } => {
                let line = Self::pos_line(position);
                self.compile_expression(receiver)?;
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                let name_idx = self.identifier_constant(method)?;
                let arg_count = arguments.len().min(255) as u8;
                // Invoke encodes name_idx in high byte, arg_count in low byte
                let operand = ((name_idx as u16) << 8) | arg_count as u16;
                self.emit_op_u16(OpCode::Invoke, operand, line);
                Ok(())
            }

            // ── Bare function call (callee(args)) ───────────────────────
            Expression::Call {
                callee,
                arguments,
                position,
                ..
            } => {
                let line = Self::pos_line(position);
                self.compile_expression(callee)?;
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                let arg_count = arguments.len().min(255) as u8;
                self.emit_op_u8(OpCode::Call, arg_count, line);
                Ok(())
            }

            // ── Self ────────────────────────────────────────────────────
            Expression::SelfExpr { position } => {
                let line = Self::pos_line(position);
                // "self" is always local slot 0 inside a method
                self.emit_op_u8(OpCode::GetLocal, 0, line);
                Ok(())
            }

            // ── Range ───────────────────────────────────────────────────
            Expression::Range {
                start,
                end,
                exclusive,
                position,
            } => {
                let line = Self::pos_line(position);
                self.compile_expression(start)?;
                self.compile_expression(end)?;
                self.emit_constant(Object::Bool(*exclusive), line)?;
                // Use a call to a built-in range constructor (3 args on stack)
                let name_idx = self.identifier_constant("__build_range")?;
                self.emit_op_u8(OpCode::Call, 3, line);
                // For now just leave the three values — full range support needs VM work
                // This is a placeholder; the bytecode VM will handle ranges
                // Pop the name since we didn't actually use it yet
                let _ = name_idx;
                Ok(())
            }

            // ── Block/lambda compilation (12.8) ───────────────────────
            Expression::Lambda {
                parameters,
                body,
                position,
                ..
            } => {
                let line = Self::pos_line(position);
                let arity = parameters.len() as u8;

                let mut block_compiler = Compiler::new();
                block_compiler.scope_depth = 1;
                block_compiler.enclosing_locals = Some(self.locals.clone());
                block_compiler.enclosing_upvalues = Some(self.upvalues.clone());

                // Add block parameters as locals
                for param in parameters {
                    block_compiler.add_local(param.clone());
                    block_compiler.mark_initialized();
                }

                // Compile the body
                for stmt in body {
                    block_compiler.compile_statement(stmt)?;
                }

                // Emit implicit nil + return
                block_compiler.emit_op(OpCode::Nil, line);
                block_compiler.emit_op(OpCode::Return, line);

                // Copy back capture flags to enclosing locals
                if let Some(ref enclosing) = block_compiler.enclosing_locals {
                    for (i, local) in enclosing.iter().enumerate() {
                        if local.is_captured && i < self.locals.len() {
                            self.locals[i].is_captured = true;
                        }
                    }
                }

                let upvalues = block_compiler.upvalues;

                let mut func = CompiledFunction::new("<block>".to_string(), arity);
                func.chunk = block_compiler.chunk;

                let func_obj = Object::CompiledFunction(Rc::new(func));
                self.emit_constant(func_obj, line)?;

                if !upvalues.is_empty() {
                    self.emit_op_u8(OpCode::Closure, upvalues.len() as u8, line);
                    for uv in &upvalues {
                        self.chunk.write_byte(u8::from(uv.is_local), line);
                        self.chunk.write_byte(uv.index, line);
                    }
                }

                Ok(())
            }

            // ── Not yet compiled ────────────────────────────────────────
            Expression::Super { position, .. }
            | Expression::Case { position, .. }
            | Expression::If { position, .. }
            | Expression::Unless { position, .. }
            | Expression::ScopeResolution { position, .. }
            | Expression::MagicFile { position, .. }
            | Expression::MagicLine { position, .. }
            | Expression::MagicDir { position, .. }
            | Expression::RegexLiteral { position, .. }
            | Expression::Splat { position, .. }
            | Expression::BlockArg { position, .. }
            | Expression::BeginRescue { position, .. }
            | Expression::Defined { position, .. }
            | Expression::Yield { position, .. } => {
                let line = Self::pos_line(position);
                Err(MetorexError::runtime_error(
                    format!(
                        "Compilation not yet implemented for this expression type at line {}",
                        line
                    ),
                    SourceLocation::new(line, 0, 0),
                ))
            }
        }
    }
}
