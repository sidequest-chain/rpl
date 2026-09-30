//! # rpl_parser — Recursive Descent & Pratt Parser for RPL
//!
//! Parses token streams emitted by `rpl_lexer` into strongly-typed AST nodes (`rpl_ast`).
//! Blocks are strictly delimited by `:` and `Token::End`. Curly braces `{}` and semicolons
//! `;` are prohibited. Expressions are parsed using a precedence climbing Pratt parser.

#![allow(clippy::result_large_err)]

pub mod error;
pub mod expr;
pub mod stmt;

pub use error::ParserError;

use rpl_ast::{Expr, Program, Span};
use rpl_lexer::{RplLexer, Token};

/// Represents an open syntactic block frame being tracked by the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockFrame {
    /// Primary block label (e.g. "fn", "if", "for", "match", "type", "spawn", or a struct name).
    pub label: String,
    /// Optional secondary label (e.g. function name for "fn", type name for "type").
    pub alt_label: Option<String>,
    /// Source code span where the block was opened (`:`).
    pub opened_at: Span,
    /// Nesting depth (1-indexed: root blocks are depth 1).
    pub depth: usize,
}

/// Recursive descent and Pratt parser state.
pub struct Parser<'a> {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    eof_span: Span,
    block_stack: Vec<BlockFrame>,
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
            block_stack: Vec::new(),
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

    /// Pushes a new block frame onto the parser block stack.
    pub fn push_block(&mut self, label: &str, alt_label: Option<&str>, opened_at: Span) {
        let depth = self.block_stack.len() + 1;
        self.block_stack.push(BlockFrame {
            label: label.to_string(),
            alt_label: alt_label.map(|s| s.to_string()),
            opened_at,
            depth,
        });
    }

    /// Expects a closing `end` token, validates optional block end label and nesting depth.
    pub fn expect_block_end(&mut self) -> Result<Span, ParserError> {
        let frame = match self.block_stack.pop() {
            Some(f) => f,
            None => {
                return self.expect_token(&Token::End, "'end'");
            }
        };

        if self.is_at_end() {
            return Err(ParserError::UnclosedBlock {
                started_at: frame.opened_at,
                span: self.eof_span,
            });
        }

        let end_span = self.expect_token(&Token::End, "'end'")?;

        // Check if an explicit label follows `end` on the same line
        let mut label_info = None;
        if let Some((tok, span)) = self.peek_with_span() {
            if span.start_line == end_span.end_line {
                match tok {
                    Token::Ident(name) => label_info = Some((name.clone(), span)),
                    Token::If => label_info = Some(("if".to_string(), span)),
                    Token::For => label_info = Some(("for".to_string(), span)),
                    Token::Match => label_info = Some(("match".to_string(), span)),
                    Token::Fn => label_info = Some(("fn".to_string(), span)),
                    Token::While => label_info = Some(("while".to_string(), span)),
                    Token::Type => label_info = Some(("type".to_string(), span)),
                    Token::Spawn => label_info = Some(("spawn".to_string(), span)),
                    _ => {}
                }
            }
        }

        if let Some((found, label_span)) = label_info {
            self.advance(); // consume the label token
            let matches = found == frame.label
                || frame.alt_label.as_deref() == Some(&found);
            if !matches {
                return Err(ParserError::MismatchedBlockEnd {
                    expected: frame.label,
                    found,
                    opened_at: frame.opened_at,
                    span: label_span,
                });
            }
            Ok(end_span.combine(label_span))
        } else {
            // Unlabeled `end`
            if frame.depth >= 3 {
                return Err(ParserError::AmbiguousBlockEnd {
                    depth: frame.depth,
                    expected: frame.label,
                    opened_at: frame.opened_at,
                    span: end_span,
                });
            }
            Ok(end_span)
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
