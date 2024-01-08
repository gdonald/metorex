// Token types for the Metorex lexer

use std::fmt;

/// Represents a part of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Text(String),
    Expression(String), // The expression inside {}
}

/// Represents the position of a token in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

/// The different kinds of tokens in Metorex
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Def,
    Class,
    If,
    Else,
    While,
    End,
    Do,

    // Literals
    Int(i64),
    Float(f64),
    String(String),
    InterpolatedString(Vec<InterpolationPart>), // String with embedded expressions
    True,
    False,
    Nil,

    // Identifiers
    Ident(String),
    InstanceVar(String), // @variable
    ClassVar(String),    // @@variable

    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Equal,        // =
    EqualEqual,   // ==
    BangEqual,    // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    // Delimiters
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Comma,    // ,
    Dot,      // .
    Colon,    // :
    Arrow,    // ->

    // Special tokens
    Newline,
    Semicolon, // ;
    Comment(String),
    EOF,
}

/// A token with its kind and position in the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, position: Position) -> Self {
        Self { kind, position }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Keywords
            TokenKind::Def => write!(f, "def"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::End => write!(f, "end"),
            TokenKind::Do => write!(f, "do"),

            // Literals
            TokenKind::Int(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::InterpolatedString(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        InterpolationPart::Text(s) => write!(f, "{}", s)?,
                        InterpolationPart::Expression(e) => write!(f, "{{{}}}", e)?,
                    }
                }
                write!(f, "\"")
            }
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Nil => write!(f, "nil"),

            // Identifiers
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::InstanceVar(s) => write!(f, "@{}", s),
            TokenKind::ClassVar(s) => write!(f, "@@{}", s),

            // Operators
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::GreaterEqual => write!(f, ">="),

            // Delimiters
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),

            // Special tokens
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Comment(s) => write!(f, "# {}", s),
            TokenKind::EOF => write!(f, "EOF"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.kind, self.position.line, self.position.column
        )
    }
}
