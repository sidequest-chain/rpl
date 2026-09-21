//! # rpl_lexer — Lexer and Tokenizer for RPL (Running Pseudo Language)
//!
//! This crate tokenizes RPL source code into a stream of `(Token, Span)` pairs.
//! Structural scoping is governed by `:` and `Token::End`. Newlines (`\n`) carry
//! semantic significance as statement terminators outside open parentheses.
//! Semicolons `;` and curly braces `{}` are strictly prohibited and produce errors.

pub mod error;
pub mod interpolation;
pub mod token;

pub use error::LexerError;
pub use interpolation::{split_interpolation, InterpolationPart};
pub use token::Token;

use rpl_ast::Span;
use token::LexerErrorInner;

/// Iterator wrapping `logos::Lexer` that computes 1-based line/column spans
/// and tracks parenthetical depth to filter insignificant newlines.
pub struct RplLexer<'a> {
    source: &'a str,
    lexer: logos::Lexer<'a, Token>,
    line_starts: Vec<usize>,
    paren_depth: usize,
    filter_bracketed_newlines: bool,
}

impl<'a> RplLexer<'a> {
    /// Creates a new `RplLexer` scanning the given source string.
    pub fn new(source: &'a str) -> Self {
        let mut line_starts = vec![0];
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(idx + 1);
            }
        }

        Self {
            source,
            lexer: logos::Logos::lexer(source),
            line_starts,
            paren_depth: 0,
            filter_bracketed_newlines: true,
        }
    }

    /// Sets whether newlines occurring inside open parentheses `()` or brackets `[]`
    /// should be silently skipped (default: `true`).
    pub fn with_bracketed_newline_filtering(mut self, filter: bool) -> Self {
        self.filter_bracketed_newlines = filter;
        self
    }

    /// Converts a byte offset to 1-based (line, column) numbers.
    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.source.len());
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_starts[line_idx];
        let line_slice = &self.source[line_start..offset];
        let col = line_slice.chars().count() + 1;
        (line_idx + 1, col)
    }

    /// Computes a `Span` for a given byte offset range `start..end`.
    pub fn span_from_range(&self, range: std::ops::Range<usize>) -> Span {
        let (start_line, start_col) = self.offset_to_line_col(range.start);
        let (end_line, end_col) = self.offset_to_line_col(range.end);
        Span::new(range.start, range.end, start_line, start_col, end_line, end_col)
    }

    /// Tokenizes the remaining source into a `Vec<(Token, Span)>` or returns the first `LexerError`.
    pub fn tokenize_all(self) -> Result<Vec<(Token, Span)>, LexerError> {
        let mut tokens = Vec::new();
        for item in self {
            tokens.push(item?);
        }
        Ok(tokens)
    }
}

impl<'a> Iterator for RplLexer<'a> {
    type Item = Result<(Token, Span), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_item = self.lexer.next()?;
            let range = self.lexer.span();
            let span = self.span_from_range(range.clone());

            match next_item {
                Ok(Token::UnterminatedStringSentinel) => {
                    return Some(Err(LexerError::UnterminatedString { span }));
                }
                Ok(token) => {
                    match &token {
                        Token::LParen | Token::LBracket => {
                            self.paren_depth += 1;
                        }
                        Token::RParen | Token::RBracket => {
                            self.paren_depth = self.paren_depth.saturating_sub(1);
                        }
                        Token::Newline if self.filter_bracketed_newlines && self.paren_depth > 0 => {
                            continue;
                        }
                        _ => {}
                    }
                    return Some(Ok((token, span)));
                }
                Err(LexerErrorInner::UnterminatedString) => {
                    return Some(Err(LexerError::UnterminatedString { span }));
                }
                Err(LexerErrorInner::InvalidNumber(message)) => {
                    return Some(Err(LexerError::InvalidNumberLiteral { message, span }));
                }
                Err(LexerErrorInner::Unexpected) => {
                    let ch = self.source[range.start..].chars().next().unwrap_or('\0');
                    return Some(Err(LexerError::UnexpectedCharacter { ch, span }));
                }
            }
        }
    }
}
