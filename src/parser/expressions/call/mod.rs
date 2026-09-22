// Function and method call parsing.
//
// `chains` reads what follows an expression, and the two argument
// modules read what a call was given, with and without parentheses.

pub(crate) use crate::ast::Expression;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::TokenKind;
pub(crate) use crate::parser::Parser;

mod arguments;
mod bare_arguments;
mod chains;
mod support;

pub(crate) use support::*;
