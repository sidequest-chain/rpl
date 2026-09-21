//! # rpl_parser — Recursive Descent & Pratt Parser for RPL
//!
//! Parses token streams emitted by `rpl_lexer` into strongly-typed AST nodes (`rpl_ast`).
//! Blocks are strictly delimited by `:` and `Token::End`. Curly braces `{}` and semicolons
//! `;` are prohibited. Expressions are parsed using a precedence climbing Pratt parser.

pub mod error;
pub mod expr;
pub mod stmt;

pub use error::ParserError;

use rpl_ast::{Expr, Program, Span};
use rpl_lexer::{RplLexer, Token};

/// Recursive descent and Pratt parser state.
pub struct Parser<'a> {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    eof_span: Span,
    _phantom: std::marker::PhantomData<&'a str>,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` by tokenizing the provided RPL source string.
    pub fn from_source(source: &'a str) -> Result<Self, ParserError> {
        let lexer = RplLexer::new(source);
        let tokens = lexer.tokenize_all()?;
        let eof_span = tokens
            .last()
            .map(|(_, s)| Span::new(s.end, s.end, s.end_line, s.end_col, s.end_line, s.end_col))
            .unwrap_or_else(Span::dummy);

        Ok(Self {
            tokens,
            pos: 0,
            eof_span,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Checks if the parser has consumed all tokens.
    pub fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    /// Returns a reference to the current lookahead token without advancing.
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    /// Returns the source span of the current lookahead token or EOF span.
    pub fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|(_, s)| *s)
            .unwrap_or(self.eof_span)
    }

    /// Returns the current lookahead token and its span.
    pub fn peek_with_span(&self) -> Option<(&Token, Span)> {
        self.tokens.get(self.pos).map(|(t, s)| (t, *s))
    }

    /// Returns a reference to the token one position ahead of current.
    pub fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1).map(|(t, _)| t)
    }

    /// Consumes and returns the current token and its span.
    pub fn advance(&mut self) -> (Token, Span) {
        if self.pos < self.tokens.len() {
            let item = self.tokens[self.pos].clone();
            self.pos += 1;
            item
        } else {
            (Token::Newline, self.eof_span)
        }
    }

    /// Returns `true` if the current token matches `token`.
    pub fn check(&self, token: &Token) -> bool {
        self.peek() == Some(token)
    }

    /// Consumes the current token if it matches `token`, returning `true`.
    pub fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consumes the current token if it matches `expected`, or returns a `ParserError::UnexpectedToken`.
    pub fn expect_token(&mut self, expected: &Token, desc: &str) -> Result<Span, ParserError> {
        if self.check(expected) {
            Ok(self.advance().1)
        } else if let Some((found, span)) = self.tokens.get(self.pos) {
            Err(ParserError::UnexpectedToken {
                expected: desc.to_string(),
                found: format!("{found}"),
                span: *span,
            })
        } else {
            Err(ParserError::UnexpectedEof {
                expected: desc.to_string(),
                span: self.eof_span,
            })
        }
    }

    /// Consumes the current token expecting an identifier, returning its name and span.
    pub fn expect_ident(&mut self, desc: &str) -> Result<(String, Span), ParserError> {
        if let Some((Token::Ident(name), span)) = self.tokens.get(self.pos) {
            let name = name.clone();
            let span = *span;
            self.pos += 1;
            Ok((name, span))
        } else if let Some((found, span)) = self.tokens.get(self.pos) {
            Err(ParserError::UnexpectedToken {
                expected: desc.to_string(),
                found: format!("{found}"),
                span: *span,
            })
        } else {
            Err(ParserError::UnexpectedEof {
                expected: desc.to_string(),
                span: self.eof_span,
            })
        }
    }

    /// Skips any consecutive newline tokens.
    pub fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(Token::Newline)) {
            self.advance();
        }
    }
}

/// Parses a full RPL source string into a top-level `Program` AST.
pub fn parse_program(source: &str) -> Result<Program, ParserError> {
    let mut parser = Parser::from_source(source)?;
    parser.parse_program_ast()
}

/// Parses an isolated RPL expression string into an `Expr` AST.
pub fn parse_expression(source: &str) -> Result<Expr, ParserError> {
    let mut parser = Parser::from_source(source)?;
    parser.parse_expr()
}
