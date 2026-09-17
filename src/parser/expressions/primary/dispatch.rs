// Primary expression dispatch: `parse_primary` is a small `match` over the
// leading token that delegates to per-category helpers in sibling modules.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

use super::literals;

impl Parser {
    /// Parse primary expressions (literals, identifiers, groups).
    pub(crate) fn parse_primary(&mut self) -> Result<Expression, MetorexError> {
        // A definition already read stands where the next primary would, so
        // the operators and calls written after it apply to what it answered.
        if let Some(seed) = self.seeded_primary.take() {
            return self.parse_postfix_calls(seed);
        }
        // Nothing is left to read, and advancing past the end would hand back
        // the token already read, which would be read again without end.
        if self.is_at_end() {
            return Err(self.error_at_previous("Unexpected end of input"));
        }
        let token = self.advance();
        let position = token.position;

        match token.kind {
            // ── Literals ────────────────────────────────────────────────────
            TokenKind::Int(value) => Ok(literals::int_literal(value, position)),
            TokenKind::BigInt(digits) => Ok(Expression::BigIntLiteral { digits, position }),
            TokenKind::Float(value) => Ok(literals::float_literal(value, position)),
            // `5r` is spelled out as the `Rational(5, 1)` it stands for.
            TokenKind::Rational(numerator, denominator) => Ok(Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Rational".to_string(),
                    position,
                }),
                arguments: vec![
                    literals::whole_number(&numerator, position),
                    literals::whole_number(&denominator, position),
                ],
                trailing_block: None,
                position,
            }),
            // `1.3i` is spelled out as the `Complex(0, 1.3)` it stands for.
            TokenKind::Imaginary(value, is_float) => Ok(Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Complex".to_string(),
                    position,
                }),
                arguments: vec![
                    literals::int_literal(0, position),
                    if is_float {
                        literals::float_literal(value.parse().unwrap_or(0.0), position)
                    } else {
                        literals::whole_number(&value, position)
                    },
                ],
                trailing_block: None,
                position,
            }),
            // Two string literals written next to each other are one string,
            // which is how a long one is split across lines.
            TokenKind::String(value) => {
                let mut spelled = value;
                while let TokenKind::String(next) = &self.peek().kind {
                    spelled.push_str(next);
                    self.advance();
                }
                Ok(literals::string_literal(spelled, position))
            }
            TokenKind::ByteString(value) => Ok(literals::byte_string_literal(value, position)),
            // A literal in a source that asked outright for literals that
            // change carries no notice that it will be frozen.
            TokenKind::MutableString(value) => Ok(Expression::MethodCall {
                receiver: Box::new(Expression::StringLiteral { value, position }),
                method: "__mutable_literal__".to_string(),
                arguments: Vec::new(),
                trailing_block: None,
                position,
            }),
            TokenKind::BinaryString(value) => Ok(literals::binary_string_literal(value, position)),
            // A literal in a source that asked for frozen literals stands for
            // the one frozen string every place writing it shares.
            TokenKind::FrozenString(value) => Ok(Expression::MethodCall {
                receiver: Box::new(Expression::StringLiteral { value, position }),
                method: "__frozen_literal__".to_string(),
                arguments: Vec::new(),
                trailing_block: None,
                position,
            }),
            TokenKind::Regex(pattern, flags) => self.regex_expression(pattern, flags, position),
            TokenKind::PercentW(value, filled) => {
                Ok(self.primary_percent_w(value, filled, position))
            }
            TokenKind::PercentI(value, filled) => {
                Ok(self.primary_percent_i(value, filled, position))
            }
            TokenKind::InterpolatedString(parts) => {
                self.primary_interpolated_string(parts, position)
            }
            TokenKind::PercentSymbol(name) => Ok(Expression::Symbol {
                value: name,
                position,
            }),
            TokenKind::CommandSymbol => Ok(Expression::Symbol {
                value: "`".to_string(),
                position,
            }),
            // A backtick literal is a call to Kernel#` with the command it
            // spells out, interpolation and all.
            // A command written with byte escapes runs those bytes.
            TokenKind::ByteCommandString(text) => Ok(Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "`".to_string(),
                    position,
                }),
                arguments: vec![Expression::MethodCall {
                    receiver: Box::new(literals::byte_string_literal(text, position)),
                    method: "freeze".to_string(),
                    arguments: Vec::new(),
                    trailing_block: None,
                    position,
                }],
                trailing_block: None,
                position,
            }),
            TokenKind::CommandString(parts) => {
                let written = parts.len() == 1
                    && matches!(parts[0], crate::lexer::InterpolationPart::Text(_));
                let command = self.primary_interpolated_string(parts, position)?;
                // A command written out in full is handed over frozen, the
                // way Ruby hands one to a `` ` `` of the program's own.
                let command = if written {
                    Expression::MethodCall {
                        receiver: Box::new(command),
                        method: "freeze".to_string(),
                        arguments: Vec::new(),
                        trailing_block: None,
                        position,
                    }
                } else {
                    command
                };
                Ok(Expression::Call {
                    callee: Box::new(Expression::Identifier {
                        name: "`".to_string(),
                        position,
                    }),
                    arguments: vec![command],
                    trailing_block: None,
                    position,
                })
            }
            TokenKind::True => Ok(literals::bool_literal(true, position)),
            TokenKind::False => Ok(literals::bool_literal(false, position)),
            TokenKind::Nil => Ok(literals::nil_literal(position)),

            // ── Identifiers and variables ───────────────────────────────────
            TokenKind::Ident(name) => Ok(literals::identifier(name, position)),
            // `include`/`extend` as a method call in expression context
            // (e.g. `should include(Foo)`). Statement-level `include Foo`
            // is dispatched before reaching primary parsing.
            TokenKind::Include => Ok(literals::identifier("include".to_string(), position)),
            TokenKind::Extend => Ok(literals::identifier("extend".to_string(), position)),
            // attr_reader/writer/accessor in expression context
            // (e.g. `(attr_accessor :foo).should ==` from the specs).
            TokenKind::AttrReader => Ok(literals::identifier("attr_reader".to_string(), position)),
            TokenKind::AttrWriter => Ok(literals::identifier("attr_writer".to_string(), position)),
            TokenKind::AttrAccessor => {
                Ok(literals::identifier("attr_accessor".to_string(), position))
            }
            TokenKind::InstanceVar(name) => Ok(literals::instance_variable(name, position)),
            TokenKind::ClassVar(name) => Ok(literals::class_variable(name, position)),
            TokenKind::GlobalVar(name) => Ok(literals::global_variable(name, position)),
            TokenKind::MagicFile => Ok(literals::magic_file(position)),
            // `__ENCODING__` names the encoding of the source it is written
            // in, which is settled where it is written rather than where the
            // code around it is run.
            TokenKind::SourceEncoding(named) if !self.check(&[TokenKind::Equal]) => {
                Ok(Expression::MethodCall {
                    receiver: Box::new(Expression::Identifier {
                        name: "Encoding".to_string(),
                        position,
                    }),
                    method: "find".to_string(),
                    arguments: vec![Expression::StringLiteral {
                        value: named,
                        position,
                    }],
                    trailing_block: None,
                    position,
                })
            }
            // Ruby refuses an assignment to `__ENCODING__` while it reads the
            // source, rather than while it runs it.
            TokenKind::SourceEncoding(_) => Err(MetorexError::syntax_error(
                "Can't set variable __ENCODING__".to_string(),
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            )),
            TokenKind::MagicLine => Ok(literals::magic_line(position)),
            TokenKind::MagicDir => Ok(Expression::MagicDir { position }),

            // ── Symbol literal: `:name`, `:@ivar`, `:[]`, `:+`, `:"..."` ────
            TokenKind::Colon => self.parse_symbol_literal(position),

            // ── Leading `::Name` top-level constant ─────────────────────────
            TokenKind::ColonColon => self.parse_leading_coloncolon(position),

            // ── Groups: `(...)`, `[...]`, `{...}` ───────────────────────────
            TokenKind::LParen => self.parse_paren_group(position),
            TokenKind::LBracket => self.parse_array_literal(position),
            TokenKind::LBrace => self.parse_dictionary_literal(position),

            // ── Block / lambda literals ─────────────────────────────────────
            TokenKind::Lambda => self.parse_lambda_literal(position),
            TokenKind::Do => self.parse_do_block(position),
            TokenKind::Arrow => self.parse_stabby_lambda(position),

            // ── Keyword-led expressions ─────────────────────────────────────
            // `a ||= raise "..."` puts a raise where a value goes, which
            // reads as a call to `raise` with what follows it.
            TokenKind::Raise => {
                let arguments = if self.check(&[TokenKind::LParen]) {
                    self.advance();
                    self.parse_arguments()?
                } else if self.check(&[
                    TokenKind::Newline,
                    TokenKind::Semicolon,
                    TokenKind::EOF,
                    TokenKind::End,
                    TokenKind::RParen,
                    TokenKind::RBrace,
                    // A modifier after `raise` says when to raise rather than
                    // what to raise, so the re-raise takes no arguments.
                    TokenKind::If,
                    TokenKind::Unless,
                    TokenKind::While,
                    TokenKind::Until,
                    TokenKind::Rescue,
                ]) {
                    Vec::new()
                } else {
                    self.parse_arguments_without_parens()?
                };
                Ok(Expression::MethodCall {
                    receiver: Box::new(Expression::SelfExpr { position }),
                    method: "raise".to_string(),
                    arguments,
                    trailing_block: None,
                    position,
                })
            }
            TokenKind::Super => self.parse_super_call(position),
            TokenKind::Defined => self.parse_defined_expression(position),
            TokenKind::Yield => self.parse_yield_expression(position),
            TokenKind::Begin => self.parse_begin_expression(position),
            TokenKind::Class => {
                self.skip_whitespace();
                if !self.check(&[TokenKind::Shovel]) {
                    return Err(self.error_at_previous(
                        "`class` as an expression is only valid in the `class << ...` form",
                    ));
                }
                self.advance();
                self.parse_singleton_class_after_shovel(position)
            }

            // `name = def held; end` reads the definition where a value goes,
            // and what it answers is the name it defined.
            TokenKind::Def => {
                self.stream
                    .restore_position(self.stream.current_position() - 1);
                let defined = self.parse_statement()?;
                Ok(Expression::BeginRescue {
                    body: vec![defined],
                    rescue_clauses: Vec::new(),
                    else_clause: None,
                    ensure_block: None,
                    position,
                })
            }

            // ── Jumps where a value goes ────────────────────────────────────
            // `found or next`, `(break 123 while true)`: Ruby reads a jump as
            // an expression, which stands for the one-statement `begin` that
            // holds it.
            TokenKind::Break | TokenKind::Continue | TokenKind::Redo | TokenKind::Return => {
                self.stream
                    .restore_position(self.stream.current_position() - 1);
                let jump = self.parse_statement()?;
                Ok(Expression::BeginRescue {
                    body: vec![jump],
                    rescue_clauses: Vec::new(),
                    else_clause: None,
                    ensure_block: None,
                    position,
                })
            }

            // ── Control-flow expressions ────────────────────────────────────
            TokenKind::Case => self.parse_case_expression(position),
            TokenKind::If => self.parse_if_expression(position),
            TokenKind::Unless => self.parse_unless_expression(position),
            TokenKind::While | TokenKind::Until => self.parse_loop_expression(position),
            TokenKind::For => self.parse_for_expression(position),

            other => Err(self.error_at_previous(&format!("Unexpected token: {:?}", other))),
        }
    }
}
