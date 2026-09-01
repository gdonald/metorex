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
                    literals::int_literal(numerator, position),
                    literals::int_literal(denominator, position),
                ],
                trailing_block: None,
                position,
            }),
            // `1.3i` is spelled out as the `Complex(0, 1.3)` it stands for.
            TokenKind::Imaginary(value) => Ok(Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Complex".to_string(),
                    position,
                }),
                arguments: vec![
                    literals::int_literal(0, position),
                    literals::float_literal(value, position),
                ],
                trailing_block: None,
                position,
            }),
            TokenKind::String(value) => Ok(literals::string_literal(value, position)),
            TokenKind::Regex(pattern, flags) => self.regex_expression(pattern, flags, position),
            TokenKind::PercentW(value) => Ok(self.primary_percent_w(value, position)),
            TokenKind::PercentI(value) => Ok(self.primary_percent_i(value, position)),
            TokenKind::InterpolatedString(parts) => {
                self.primary_interpolated_string(parts, position)
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

            // ── Control-flow expressions ────────────────────────────────────
            TokenKind::Case => self.parse_case_expression(position),
            TokenKind::If => self.parse_if_expression(position),
            TokenKind::Unless => self.parse_unless_expression(position),

            other => Err(self.error_at_previous(&format!("Unexpected token: {:?}", other))),
        }
    }
}
