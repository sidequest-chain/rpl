//! Token definitions and Logos lexical rules for RPL.

use std::fmt;
use logos::{Lexer, Logos};

/// Internal error kind produced by Logos callbacks before span attachment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LexerErrorInner {
    /// Generic unexpected character or token failure.
    #[default]
    Unexpected,
    /// An unclosed string literal was found.
    UnterminatedString,
    /// A number literal was malformed or failed parsing.
    InvalidNumber(String),
}

/// Callback that always produces an `UnterminatedString` error.
fn catch_unterminated_string(_lex: &mut Lexer<Token>) -> Result<(), LexerErrorInner> {
    Err(LexerErrorInner::UnterminatedString)
}

/// Parses a decimal integer literal.
fn parse_decimal_int(lex: &mut Lexer<Token>) -> Result<i64, LexerErrorInner> {
    let raw = lex.slice().replace('_', "");
    raw.parse::<i64>()
        .map_err(|e| LexerErrorInner::InvalidNumber(e.to_string()))
}

/// Parses a hexadecimal integer literal (`0x...` or `0X...`).
fn parse_hex_int(lex: &mut Lexer<Token>) -> Result<i64, LexerErrorInner> {
    let slice = lex.slice();
    let digits = slice[2..].replace('_', "");
    if digits.is_empty() {
        return Err(LexerErrorInner::InvalidNumber(
            "Empty hexadecimal integer literal".into(),
        ));
    }
    i64::from_str_radix(&digits, 16)
        .map_err(|e| LexerErrorInner::InvalidNumber(e.to_string()))
}

/// Parses a binary integer literal (`0b...` or `0B...`).
fn parse_binary_int(lex: &mut Lexer<Token>) -> Result<i64, LexerErrorInner> {
    let slice = lex.slice();
    let digits = slice[2..].replace('_', "");
    if digits.is_empty() {
        return Err(LexerErrorInner::InvalidNumber(
            "Empty binary integer literal".into(),
        ));
    }
    i64::from_str_radix(&digits, 2)
        .map_err(|e| LexerErrorInner::InvalidNumber(e.to_string()))
}

/// Parses a floating-point number literal.
fn parse_float(lex: &mut Lexer<Token>) -> Result<f64, LexerErrorInner> {
    let raw = lex.slice().replace('_', "");
    raw.parse::<f64>()
        .map_err(|e| LexerErrorInner::InvalidNumber(e.to_string()))
}

/// Parses a string literal with escape sequence replacement.
fn parse_string(lex: &mut Lexer<Token>) -> Result<String, LexerErrorInner> {
    let slice = lex.slice();
    if slice.len() < 2 || !slice.starts_with('"') || !slice.ends_with('"') {
        return Err(LexerErrorInner::UnterminatedString);
    }
    let inner = &slice[1..slice.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('$') => out.push('$'),
                Some('0') => out.push('\0'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => return Err(LexerErrorInner::UnterminatedString),
            }
        } else {
            out.push(ch);
        }
    }

    Ok(out)
}

/// Tokens recognized in the RPL (Running Pseudo Language) lexical grammar.
///
/// NOTE: In accordance with language specifications, curly braces `{}` and semicolons `;`
/// are strictly forbidden and intentionally omitted from this enum.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(error = LexerErrorInner)]
#[logos(skip r"[ \t\f]+")]
#[logos(skip r"//[^\r\n]*")]
#[logos(skip r"/\*([^*]|\*+[^*/])*\*+/")]
pub enum Token {
    // -------------------------------------------------------------------------
    // 1. Reserved Keywords (28 keywords)
    // -------------------------------------------------------------------------
    #[token("and")]
    And,

    #[token("as")]
    As,

    #[token("break")]
    Break,

    #[token("case")]
    Case,

    #[token("channel")]
    Channel,

    #[token("const")]
    Const,

    #[token("continue")]
    Continue,

    #[token("else")]
    Else,

    #[token("end")]
    End,

    #[token("false")]
    False,

    #[token("fn")]
    Fn,

    #[token("for")]
    For,

    #[token("if")]
    If,

    #[token("in")]
    In,

    #[token("is")]
    Is,

    #[token("let")]
    Let,

    #[token("match")]
    Match,

    #[token("mut")]
    Mut,

    #[token("not")]
    Not,

    #[token("or")]
    Or,

    #[token("parallel")]
    Parallel,

    #[token("return")]
    Return,

    #[token("spawn")]
    Spawn,

    #[token("true")]
    True,

    #[token("type")]
    Type,

    #[token("unknown")]
    Unknown,

    #[token("while")]
    While,

    #[token("yield")]
    Yield,

    // -------------------------------------------------------------------------
    // 2. Operators and Punctuation
    // -------------------------------------------------------------------------
    // Arithmetic operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    // Relational comparison operators
    #[token("==")]
    EqEq,

    #[token("!=")]
    NotEq,

    #[token("<")]
    Lt,

    #[token("<=")]
    LtEq,

    #[token(">")]
    Gt,

    #[token(">=")]
    GtEq,

    // Bitwise / logical operators
    #[token("&")]
    Ampersand,

    #[token("|")]
    Pipe,

    #[token("^")]
    Caret,

    #[token("~")]
    Tilde,

    #[token("<<")]
    Shl,

    #[token(">>")]
    Shr,

    // Pipes and arrows
    #[token("|>")]
    PipeRight,

    #[token("->")]
    Arrow,

    #[token("=>")]
    FatArrow,

    // Assignment
    #[token("=")]
    Assign,

    // Delimiters
    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("..")]
    DotDot,

    #[token(".")]
    Dot,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    // -------------------------------------------------------------------------
    // 3. Literals and Identifiers
    // -------------------------------------------------------------------------
    // Hexadecimal integer literal (e.g. 0x1A2B)
    #[regex(r"0[xX][0-9a-zA-Z_]*", parse_hex_int)]
    HexInt(i64),

    // Binary integer literal (e.g. 0b1010)
    #[regex(r"0[bB][0-9a-zA-Z_]*", parse_binary_int)]
    BinaryInt(i64),

    // Standard floating-point number literal (e.g. 12.34, 0.5, 1e-3)
    #[regex(
        r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9_]+)?|[0-9][0-9_]*[eE][+-]?[0-9_]+",
        parse_float
    )]
    Float(f64),

    // Decimal integer literal (e.g. 1234, 0)
    #[regex(r"[0-9][0-9_]*", parse_decimal_int)]
    Int(i64),

    // String literal with escape sequence support
    #[regex(r#""([^"\\\r\n]|\\.)*""#, parse_string)]
    String(String),

    // Unterminated string catch rule (matches unclosed quote up to newline or EOF)
    #[regex(r#""([^"\\\r\n]|\\.)*"#, catch_unterminated_string)]
    UnterminatedStringSentinel,

    // String interpolation start tokens (when lexing interpolated fragments)
    #[regex(r"\$[a-zA-Z_\u{0080}-\u{10FFFF}][a-zA-Z0-9_\u{0080}-\u{10FFFF}]*", |lex| lex.slice()[1..].to_string())]
    DollarIdent(String),

    #[token("$(")]
    DollarLParen,

    // Identifiers (supports ASCII and UTF-8 graphemes/letters)
    #[regex(r"[\p{XID_Start}_][\p{XID_Continue}]*", |lex| lex.slice().to_string())]
    Ident(String),

    // -------------------------------------------------------------------------
    // 4. Line terminators
    // -------------------------------------------------------------------------
    #[regex(r"\r?\n")]
    Newline,
}

impl Token {
    /// Normalizes integer literal variants (`Int`, `HexInt`, `BinaryInt`) into their integer value.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) | Self::HexInt(v) | Self::BinaryInt(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns `true` if this token is any integer literal.
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_) | Self::HexInt(_) | Self::BinaryInt(_))
    }

    /// Returns `true` if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::And
                | Self::As
                | Self::Break
                | Self::Case
                | Self::Channel
                | Self::Const
                | Self::Continue
                | Self::Else
                | Self::End
                | Self::False
                | Self::Fn
                | Self::For
                | Self::If
                | Self::In
                | Self::Is
                | Self::Let
                | Self::Match
                | Self::Mut
                | Self::Not
                | Self::Or
                | Self::Parallel
                | Self::Return
                | Self::Spawn
                | Self::True
                | Self::Type
                | Self::Unknown
                | Self::While
                | Self::Yield
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::And => write!(f, "and"),
            Self::As => write!(f, "as"),
            Self::Break => write!(f, "break"),
            Self::Case => write!(f, "case"),
            Self::Channel => write!(f, "channel"),
            Self::Const => write!(f, "const"),
            Self::Continue => write!(f, "continue"),
            Self::Else => write!(f, "else"),
            Self::End => write!(f, "end"),
            Self::False => write!(f, "false"),
            Self::Fn => write!(f, "fn"),
            Self::For => write!(f, "for"),
            Self::If => write!(f, "if"),
            Self::In => write!(f, "in"),
            Self::Is => write!(f, "is"),
            Self::Let => write!(f, "let"),
            Self::Match => write!(f, "match"),
            Self::Mut => write!(f, "mut"),
            Self::Not => write!(f, "not"),
            Self::Or => write!(f, "or"),
            Self::Parallel => write!(f, "parallel"),
            Self::Return => write!(f, "return"),
            Self::Spawn => write!(f, "spawn"),
            Self::True => write!(f, "true"),
            Self::Type => write!(f, "type"),
            Self::Unknown => write!(f, "unknown"),
            Self::While => write!(f, "while"),
            Self::Yield => write!(f, "yield"),

            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),

            Self::EqEq => write!(f, "=="),
            Self::NotEq => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::LtEq => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::GtEq => write!(f, ">="),

            Self::Ampersand => write!(f, "&"),
            Self::Pipe => write!(f, "|"),
            Self::Caret => write!(f, "^"),
            Self::Tilde => write!(f, "~"),
            Self::Shl => write!(f, "<<"),
            Self::Shr => write!(f, ">>"),

            Self::PipeRight => write!(f, "|>"),
            Self::Arrow => write!(f, "->"),
            Self::FatArrow => write!(f, "=>"),
            Self::Assign => write!(f, "="),

            Self::Colon => write!(f, ":"),
            Self::Comma => write!(f, ","),
            Self::DotDot => write!(f, ".."),
            Self::Dot => write!(f, "."),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),

            Self::Int(n) | Self::HexInt(n) | Self::BinaryInt(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::UnterminatedStringSentinel => write!(f, "<unterminated string>"),
            Self::DollarIdent(id) => write!(f, "${id}"),
            Self::DollarLParen => write!(f, "$("),
            Self::Ident(id) => write!(f, "{id}"),
            Self::Newline => write!(f, "<newline>"),
        }
    }
}
