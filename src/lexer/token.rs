// Token types for the Metorex lexer

use std::fmt;

/// Represents a part of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Text(String),
    /// The expression inside `#{}`, with the source line its `#{` was
    /// written on. Zero where the line is not known, which falls back to the
    /// line the literal itself starts on.
    Expression(String, usize),
}

/// Represents the position of a token in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    /// Whether this came from the core library metorex loads at startup.
    /// Ruby never traces its own C code, and the prelude stands in for it.
    pub prelude: bool,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
            prelude: false,
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
    Elsif,
    Else,
    Unless,
    While,
    Until,
    For,
    In,
    End,
    Do,
    Begin,
    Rescue,
    Ensure,
    Raise,
    Break,
    Continue,
    Redo,
    Retry,
    Return,
    Lambda,
    Super,
    Yield,
    Defined,
    Case,
    When,
    Then,
    AttrReader,
    AttrWriter,
    AttrAccessor,
    Module,
    Include,
    Extend,
    Alias,
    MagicFile,
    MagicLine,
    MagicDir,
    /// `__ENCODING__`, carrying the encoding the source it is written in
    /// says it is written in.
    SourceEncoding(String),

    // Literals
    Int(i64),
    /// An integer literal too large for an i64, kept as its digits.
    BigInt(String),
    Float(f64),
    /// The `r` literal suffix: numerator and denominator as base-ten digits,
    /// which a literal past an i64 needs.
    Rational(String, String),
    /// The `i` literal suffix, as in `1.3i`: the number as it was written,
    /// and whether it was written as a float.
    Imaginary(String, bool),
    String(String),
    InterpolatedString(Vec<InterpolationPart>), // String with embedded expressions
    /// A backtick command literal, `` `echo hi` ``, whose parts interpolate
    /// the same way a double-quoted string's do.
    CommandString(Vec<InterpolationPart>),
    /// `:` followed by a backtick, which names the command method.
    CommandSymbol,
    /// `%s{name}`, a symbol written the way a `%q` string is.
    PercentSymbol(String),
    Regex(String, String),  // pattern, flags
    PercentW(String, bool), // %w[...] words, true when %W fills in `#{}`
    PercentI(String, bool), // %i[...] symbols, true when %I fills in `#{}`
    True,
    False,
    Nil,

    // Identifiers
    Ident(String),
    InstanceVar(String), // @variable
    ClassVar(String),    // @@variable
    GlobalVar(String),   // $variable

    // Operators
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Percent,         // %
    Caret,           // ^
    Equal,           // =
    Bang,            // !
    EqualEqual,      // ==
    TripleEqual,     // ===
    BangEqual,       // !=
    Match,           // =~
    NotMatch,        // !~
    StarStar,        // ** (double splat / exponent)
    Less,            // <
    Greater,         // >
    LessEqual,       // <=
    Spaceship,       // <=>
    Shovel,          // <<
    Question,        // ? (ternary operator)
    GreaterEqual,    // >=
    PlusEqual,       // +=
    MinusEqual,      // -=
    StarEqual,       // *=
    SlashEqual,      // /=
    PercentEqual,    // %=
    StarStarEqual,   // **=
    PipeEqual,       // |=
    AmpersandEqual,  // &=
    CaretEqual,      // ^=
    ShovelEqual,     // <<=
    RightShiftEqual, // >>=

    // Delimiters
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Dot,        // .
    DotDot,     // ..
    DotDotDot,  // ...
    Colon,      // :
    Arrow,      // ->
    FatArrow,   // =>
    Pipe,       // |
    Ampersand,  // &
    SafeDot,    // &. — a call that answers nil for a nil receiver
    NotKeyword, // the word `not`, which takes a parenthesized operand
    LogicalAnd, // &&
    LogicalOr,  // ||
    // `and` and `or` are the same test as `&&` and `||`, but they bind more
    // loosely than everything else, so `take x and y` calls `take x` first.
    /// A string literal whose escapes named bytes that spell no text, so the
    /// characters stand for those bytes rather than for what they read as.
    ByteString(String),
    /// A backtick command whose escapes named bytes rather than characters.
    ByteCommandString(String),
    /// A literal in a source written in bytes, which stands for those bytes
    /// rather than for the characters they spell.
    BinaryString(String),
    /// A literal in a source that asked for frozen literals, which stands for
    /// the one frozen string every place writing it shares.
    FrozenString(String),
    /// A literal in a source that asked outright for literals that change,
    /// which carries no notice that a later release will freeze it.
    MutableString(String),
    KeywordAnd,       // and
    KeywordOr,        // or
    LogicalOrAssign,  // ||=
    LogicalAndAssign, // &&=
    ColonColon,       // ::
    Tilde,            // ~ (bitwise complement)
    RightShift,       // >> (right shift)

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
    /// True when at least one whitespace character was skipped between the
    /// previous token and this one. Used to disambiguate `m[...]` (indexing)
    /// from `m [...]` (paren-less array argument).
    pub had_leading_space: bool,
}

impl Token {
    pub fn new(kind: TokenKind, position: Position) -> Self {
        Self {
            kind,
            position,
            had_leading_space: false,
        }
    }

    pub fn with_leading_space(
        kind: TokenKind,
        position: Position,
        had_leading_space: bool,
    ) -> Self {
        Self {
            kind,
            position,
            had_leading_space,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Keywords
            TokenKind::Def => write!(f, "def"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Elsif => write!(f, "elsif"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Unless => write!(f, "unless"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Until => write!(f, "until"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::End => write!(f, "end"),
            TokenKind::Do => write!(f, "do"),
            TokenKind::Begin => write!(f, "begin"),
            TokenKind::Rescue => write!(f, "rescue"),
            TokenKind::Ensure => write!(f, "ensure"),
            TokenKind::Raise => write!(f, "raise"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Redo => write!(f, "redo"),
            TokenKind::Retry => write!(f, "retry"),
            TokenKind::Continue => write!(f, "next"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Lambda => write!(f, "lambda"),
            TokenKind::Super => write!(f, "super"),
            TokenKind::Yield => write!(f, "yield"),
            TokenKind::Defined => write!(f, "defined?"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::When => write!(f, "when"),
            TokenKind::Then => write!(f, "then"),
            TokenKind::AttrReader => write!(f, "attr_reader"),
            TokenKind::AttrWriter => write!(f, "attr_writer"),
            TokenKind::AttrAccessor => write!(f, "attr_accessor"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Include => write!(f, "include"),
            TokenKind::Extend => write!(f, "extend"),
            TokenKind::Alias => write!(f, "alias"),
            TokenKind::MagicFile => write!(f, "__FILE__"),
            TokenKind::MagicLine => write!(f, "__LINE__"),
            TokenKind::MagicDir => write!(f, "__dir__"),
            TokenKind::SourceEncoding(_) => write!(f, "__ENCODING__"),

            // Literals
            TokenKind::Int(n) => write!(f, "{}", n),
            TokenKind::BigInt(digits) => write!(f, "{}", digits),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::PercentW(s, _) => write!(f, "%w[{}]", s),
            TokenKind::PercentI(s, _) => write!(f, "%i[{}]", s),
            TokenKind::CommandSymbol => write!(f, ":`"),
            TokenKind::PercentSymbol(name) => write!(f, "%s{{{}}}", name),
            TokenKind::CommandString(parts) => {
                write!(f, "`")?;
                for part in parts {
                    match part {
                        InterpolationPart::Text(s) => write!(f, "{}", s)?,
                        InterpolationPart::Expression(e, _) => write!(f, "#{{{}}}", e)?,
                    }
                }
                write!(f, "`")
            }
            TokenKind::InterpolatedString(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        InterpolationPart::Text(s) => write!(f, "{}", s)?,
                        InterpolationPart::Expression(e, _) => write!(f, "{{{}}}", e)?,
                    }
                }
                write!(f, "\"")
            }
            TokenKind::Regex(pat, flags) => write!(f, "/{}/{}", pat, flags),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Nil => write!(f, "nil"),

            // Identifiers
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::InstanceVar(s) => write!(f, "@{}", s),
            TokenKind::ClassVar(s) => write!(f, "@@{}", s),
            TokenKind::GlobalVar(s) => write!(f, "${}", s),

            // Operators
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::TripleEqual => write!(f, "==="),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::Match => write!(f, "=~"),
            TokenKind::NotMatch => write!(f, "!~"),
            TokenKind::Rational(numerator, denominator) => {
                write!(f, "{}/{}r", numerator, denominator)
            }
            TokenKind::Imaginary(value, _) => write!(f, "{value}i"),
            TokenKind::StarStar => write!(f, "**"),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Spaceship => write!(f, "<=>"),
            TokenKind::Shovel => write!(f, "<<"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::PlusEqual => write!(f, "+="),
            TokenKind::MinusEqual => write!(f, "-="),
            TokenKind::StarEqual => write!(f, "*="),
            TokenKind::SlashEqual => write!(f, "/="),
            TokenKind::PercentEqual => write!(f, "%="),
            TokenKind::StarStarEqual => write!(f, "**="),
            TokenKind::PipeEqual => write!(f, "|="),
            TokenKind::AmpersandEqual => write!(f, "&="),
            TokenKind::CaretEqual => write!(f, "^="),
            TokenKind::ShovelEqual => write!(f, "<<="),
            TokenKind::RightShiftEqual => write!(f, ">>="),

            // Delimiters
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotDot => write!(f, "..."),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::SafeDot => write!(f, "&."),
            TokenKind::NotKeyword => write!(f, "not"),
            TokenKind::LogicalAnd => write!(f, "&&"),
            TokenKind::ByteString(text) => write!(f, "{}", text),
            TokenKind::MutableString(text) => write!(f, "{}", text),
            TokenKind::ByteCommandString(text) => write!(f, "`{}`", text),
            TokenKind::BinaryString(text) => write!(f, "{}", text),
            TokenKind::FrozenString(text) => write!(f, "{}", text),
            TokenKind::KeywordAnd => write!(f, "and"),
            TokenKind::KeywordOr => write!(f, "or"),
            TokenKind::LogicalOr => write!(f, "||"),
            TokenKind::LogicalOrAssign => write!(f, "||="),
            TokenKind::LogicalAndAssign => write!(f, "&&="),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::RightShift => write!(f, ">>"),

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
