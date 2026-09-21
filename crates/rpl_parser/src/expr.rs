//! Pratt expression parser for RPL expressions and operators.

use rpl_ast::{BinaryOp, Expr, InterpolationFragment, Literal, Span, TritValue, UnaryOp};
use rpl_lexer::{split_interpolation, InterpolationPart, Token};

use crate::error::ParserError;
use crate::Parser;

/// Binding power levels for Pratt parsing.
const PRECEDENCE_RANGE: u8 = 2;
const PRECEDENCE_PIPE: u8 = 4;
const PRECEDENCE_OR: u8 = 6;
const PRECEDENCE_AND: u8 = 8;
const PRECEDENCE_RELATIONAL: u8 = 10;
const PRECEDENCE_BIT_OR: u8 = 12;
const PRECEDENCE_BIT_XOR: u8 = 14;
const PRECEDENCE_BIT_AND: u8 = 16;
const PRECEDENCE_SHIFT: u8 = 18;
const PRECEDENCE_ADD: u8 = 20;
const PRECEDENCE_MUL: u8 = 22;
const PRECEDENCE_UNARY: u8 = 24;
const PRECEDENCE_POSTFIX: u8 = 26;

impl<'a> Parser<'a> {
    /// Parses an expression with minimum binding power 0.
    pub fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        self.parse_expr_bp(0)
    }

    /// Checks if the next non-newline token continues the expression (e.g. `|>`, `|`, `+`, etc.).
    fn peek_is_continuation(&self) -> bool {
        let mut idx = self.pos;
        while let Some((Token::Newline, _)) = self.tokens.get(idx) {
            idx += 1;
        }
        matches!(
            self.tokens.get(idx),
            Some((
                Token::PipeRight
                    | Token::Pipe
                    | Token::Plus
                    | Token::Minus
                    | Token::Star
                    | Token::Slash
                    | Token::Percent
                    | Token::Ampersand
                    | Token::Caret
                    | Token::Shl
                    | Token::Shr
                    | Token::And
                    | Token::Or
                    | Token::DotDot,
                _
            ))
        )
    }

    /// Parses an expression using Pratt binding power / precedence climbing.
    pub fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParserError> {
        let mut left = self.parse_primary_or_prefix()?;

        loop {
            // Check for multiline continuation
            if self.check(&Token::Newline) {
                if self.peek_is_continuation() {
                    self.skip_newlines();
                } else {
                    break;
                }
            }

            // Check for postfix operators: calls `()`, field access `.`, index/generic `[]`
            if let Some(token) = self.peek() {
                match token {
                    Token::LParen if PRECEDENCE_POSTFIX >= min_bp => {
                        self.advance();
                        let args = self.parse_call_arguments()?;
                        let rparen_span = self.expect_token(&Token::RParen, "')'")?;
                        let span = left.span().combine(rparen_span);
                        left = Expr::Call {
                            callee: Box::new(left),
                            args,
                            span,
                        };
                        continue;
                    }
                    Token::Dot if PRECEDENCE_POSTFIX >= min_bp => {
                        self.advance();
                        let (field, field_span) = self.expect_ident("field name")?;
                        let span = left.span().combine(field_span);
                        left = Expr::MemberAccess {
                            target: Box::new(left),
                            field,
                            span,
                        };
                        continue;
                    }
                    Token::LBracket if PRECEDENCE_POSTFIX >= min_bp => {
                        let lbracket_span = self.advance().1;
                        self.skip_newlines();
                        let mut index_elems = Vec::new();

                        while !self.check(&Token::RBracket) && !self.is_at_end() {
                            let elem = self.parse_expr()?;
                            index_elems.push(elem);
                            self.skip_newlines();

                            if self.match_token(&Token::Comma) {
                                self.skip_newlines();
                                continue;
                            }
                            break;
                        }

                        let rbracket_span = self.expect_token(&Token::RBracket, "']'")?;
                        let index = if index_elems.len() == 1 {
                            index_elems.remove(0)
                        } else {
                            Expr::List {
                                elements: index_elems,
                                span: lbracket_span.combine(rbracket_span),
                            }
                        };

                        let span = left.span().combine(rbracket_span);
                        left = Expr::Index {
                            target: Box::new(left),
                            index: Box::new(index),
                            span,
                        };
                        continue;
                    }
                    _ => {}
                }
            }

            // Check for infix binary operators
            let (lbp, rbp, op_kind) = match self.peek() {
                Some(Token::PipeRight) => (PRECEDENCE_PIPE, PRECEDENCE_PIPE + 1, InfixKind::Pipe),
                Some(Token::Or) => (PRECEDENCE_OR, PRECEDENCE_OR + 1, InfixKind::Binary(BinaryOp::Or)),
                Some(Token::And) => (PRECEDENCE_AND, PRECEDENCE_AND + 1, InfixKind::Binary(BinaryOp::And)),
                Some(Token::EqEq) => (PRECEDENCE_RELATIONAL, PRECEDENCE_RELATIONAL + 1, InfixKind::Binary(BinaryOp::Eq)),
                Some(Token::NotEq) => (PRECEDENCE_RELATIONAL, PRECEDENCE_RELATIONAL + 1, InfixKind::Binary(BinaryOp::NotEq)),
                Some(Token::Lt) => (PRECEDENCE_RELATIONAL, PRECEDENCE_RELATIONAL + 1, InfixKind::Binary(BinaryOp::Lt)),
                Some(Token::LtEq) => (PRECEDENCE_RELATIONAL, PRECEDENCE_RELATIONAL + 1, InfixKind::Binary(BinaryOp::LtEq)),
                Some(Token::Gt) => (PRECEDENCE_RELATIONAL, PRECEDENCE_RELATIONAL + 1, InfixKind::Binary(BinaryOp::Gt)),
                Some(Token::GtEq) => (PRECEDENCE_RELATIONAL, PRECEDENCE_RELATIONAL + 1, InfixKind::Binary(BinaryOp::GtEq)),
                Some(Token::Pipe) => (PRECEDENCE_BIT_OR, PRECEDENCE_BIT_OR + 1, InfixKind::Binary(BinaryOp::BitOr)),
                Some(Token::Caret) => (PRECEDENCE_BIT_XOR, PRECEDENCE_BIT_XOR + 1, InfixKind::Binary(BinaryOp::BitXor)),
                Some(Token::Ampersand) => (PRECEDENCE_BIT_AND, PRECEDENCE_BIT_AND + 1, InfixKind::Binary(BinaryOp::BitAnd)),
                Some(Token::Shl) => (PRECEDENCE_SHIFT, PRECEDENCE_SHIFT + 1, InfixKind::Binary(BinaryOp::Shl)),
                Some(Token::Shr) => (PRECEDENCE_SHIFT, PRECEDENCE_SHIFT + 1, InfixKind::Binary(BinaryOp::Shr)),
                Some(Token::Plus) => (PRECEDENCE_ADD, PRECEDENCE_ADD + 1, InfixKind::Binary(BinaryOp::Add)),
                Some(Token::Minus) => (PRECEDENCE_ADD, PRECEDENCE_ADD + 1, InfixKind::Binary(BinaryOp::Sub)),
                Some(Token::Star) => (PRECEDENCE_MUL, PRECEDENCE_MUL + 1, InfixKind::Binary(BinaryOp::Mul)),
                Some(Token::Slash) => (PRECEDENCE_MUL, PRECEDENCE_MUL + 1, InfixKind::Binary(BinaryOp::Div)),
                Some(Token::Percent) => (PRECEDENCE_MUL, PRECEDENCE_MUL + 1, InfixKind::Binary(BinaryOp::Mod)),
                Some(Token::DotDot) => (PRECEDENCE_RANGE, PRECEDENCE_RANGE + 1, InfixKind::Range),
                _ => break,
            };

            if lbp < min_bp {
                break;
            }

            self.advance();
            self.skip_newlines();
            let right = self.parse_expr_bp(rbp)?;
            let span = left.span().combine(right.span());

            left = match op_kind {
                InfixKind::Pipe => Expr::Pipe {
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                },
                InfixKind::Binary(bin_op) => Expr::Binary {
                    left: Box::new(left),
                    op: bin_op,
                    right: Box::new(right),
                    span,
                },
                InfixKind::Range => Expr::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                    span,
                },
            };
        }

        Ok(left)
    }

    /// Parses primary expressions, literals, parenthesized expressions, and prefix unary operations.
    fn parse_primary_or_prefix(&mut self) -> Result<Expr, ParserError> {
        let (token, span) = self.advance();

        match token {
            // Numeric literals
            Token::Int(n) | Token::HexInt(n) | Token::BinaryInt(n) => {
                Ok(Expr::Literal(Literal::Int(n), span))
            }
            Token::Float(f) => Ok(Expr::Literal(Literal::Float(f), span)),

            // Boolean and Trit literals
            Token::True => Ok(Expr::Literal(Literal::Bool(true), span)),
            Token::False => Ok(Expr::Literal(Literal::Bool(false), span)),
            Token::Unknown => Ok(Expr::Literal(Literal::Trit(TritValue::Unknown), span)),

            // String literals and interpolation
            Token::String(s) => self.parse_string_literal(s, span),

            // Direct string interpolation fragments
            Token::DollarIdent(id) => Ok(Expr::StringInterpolation {
                fragments: vec![InterpolationFragment::Expr(Expr::Identifier(id, span))],
                span,
            }),
            Token::DollarLParen => {
                let inner = self.parse_expr()?;
                let rparen_span = self.expect_token(&Token::RParen, "')'")?;
                Ok(Expr::StringInterpolation {
                    fragments: vec![InterpolationFragment::Expr(inner)],
                    span: span.combine(rparen_span),
                })
            }

            // Identifiers and lambdas (`param => body`)
            Token::Ident(name) => {
                // Check if this identifier is followed by `:` in named arguments (`sku: "..."`)
                if self.check(&Token::Colon) && self.peek_next() != Some(&Token::Newline) {
                    self.advance(); // consume ':'
                    let value = self.parse_expr()?;
                    let arg_span = span.combine(value.span());
                    return Ok(Expr::NamedArg {
                        name,
                        value: Box::new(value),
                        span: arg_span,
                    });
                }

                // Check for single parameter closure (`token => token.length > 0`)
                if self.match_token(&Token::FatArrow) {
                    let body = self.parse_expr()?;
                    let lambda_span = span.combine(body.span());
                    return Ok(Expr::Lambda {
                        params: vec![name],
                        body: Box::new(body),
                        span: lambda_span,
                    });
                }

                Ok(Expr::Identifier(name, span))
            }

            // Parenthesized expression `(expr)` or parameter closure `(a, b) => expr`
            Token::LParen => {
                self.skip_newlines();
                if self.match_token(&Token::RParen) {
                    // Empty parens followed by => is a zero-arg lambda
                    if self.match_token(&Token::FatArrow) {
                        let body = self.parse_expr()?;
                        let lambda_span = span.combine(body.span());
                        return Ok(Expr::Lambda {
                            params: Vec::new(),
                            body: Box::new(body),
                            span: lambda_span,
                        });
                    }
                    return Err(ParserError::UnexpectedToken {
                        expected: "expression inside parentheses".into(),
                        found: "')'".into(),
                        span,
                    });
                }

                let inner = self.parse_expr()?;
                self.skip_newlines();
                let _rparen_span = self.expect_token(&Token::RParen, "')'")?;

                if self.match_token(&Token::FatArrow) {
                    let body = self.parse_expr()?;
                    let params = match inner {
                        Expr::Identifier(name, _) => vec![name],
                        _ => Vec::new(),
                    };
                    let lambda_span = span.combine(body.span());
                    return Ok(Expr::Lambda {
                        params,
                        body: Box::new(body),
                        span: lambda_span,
                    });
                }

                Ok(inner)
            }

            // List/Array literal: `[item1, item2, ...]`
            Token::LBracket => {
                let mut elements = Vec::new();
                self.skip_newlines();

                while !self.check(&Token::RBracket) && !self.is_at_end() {
                    let elem = self.parse_expr()?;
                    elements.push(elem);
                    self.skip_newlines();

                    if self.match_token(&Token::Comma) {
                        self.skip_newlines();
                        continue;
                    }
                    break;
                }

                let rbracket_span = self.expect_token(&Token::RBracket, "']'")?;
                Ok(Expr::List {
                    elements,
                    span: span.combine(rbracket_span),
                })
            }

            // Unary prefix operators: `not`, `-`, `~`
            Token::Not => {
                let expr = self.parse_expr_bp(PRECEDENCE_UNARY)?;
                let total_span = span.combine(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span: total_span,
                })
            }
            Token::Minus => {
                let expr = self.parse_expr_bp(PRECEDENCE_UNARY)?;
                let total_span = span.combine(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                    span: total_span,
                })
            }
            Token::Tilde => {
                let expr = self.parse_expr_bp(PRECEDENCE_UNARY)?;
                let total_span = span.combine(expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
                    expr: Box::new(expr),
                    span: total_span,
                })
            }

            other => Err(ParserError::UnexpectedToken {
                expected: "expression (literal, identifier, unary op, or parenthesis)".into(),
                found: format!("{other}"),
                span,
            }),
        }
    }

    /// Parses positional and named arguments within function/constructor calls `callee(arg1, arg2, name: val)`.
    fn parse_call_arguments(&mut self) -> Result<Vec<Expr>, ParserError> {
        let mut args = Vec::new();
        self.skip_newlines();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            let arg = self.parse_expr()?;
            args.push(arg);
            self.skip_newlines();

            if self.match_token(&Token::Comma) {
                self.skip_newlines();
                continue;
            }
            break;
        }

        Ok(args)
    }

    /// Converts a string literal into either a plain `Expr::Literal` or an `Expr::StringInterpolation`.
    fn parse_string_literal(&self, s: String, span: Span) -> Result<Expr, ParserError> {
        let parts = split_interpolation(&s);
        let has_interpolation = parts.iter().any(|p| !matches!(p, InterpolationPart::Literal(_)));

        if !has_interpolation {
            return Ok(Expr::Literal(Literal::String(s), span));
        }

        let mut fragments = Vec::new();
        for part in parts {
            match part {
                InterpolationPart::Literal(text) => {
                    fragments.push(InterpolationFragment::Literal(text));
                }
                InterpolationPart::Variable(var) => {
                    // Check for nested member access in $var.field
                    if var.contains('.') {
                        let mut segments = var.split('.');
                        let base_name = segments.next().unwrap_or("").to_string();
                        let mut current_expr = Expr::Identifier(base_name, span);
                        for field in segments {
                            current_expr = Expr::MemberAccess {
                                target: Box::new(current_expr),
                                field: field.to_string(),
                                span,
                            };
                        }
                        fragments.push(InterpolationFragment::Expr(current_expr));
                    } else {
                        fragments.push(InterpolationFragment::Expr(Expr::Identifier(var, span)));
                    }
                }
                InterpolationPart::Expression(expr_str) => {
                    let sub_expr = match crate::parse_expression(&expr_str) {
                        Ok(e) => e,
                        Err(_) => Expr::Identifier(expr_str, span),
                    };
                    fragments.push(InterpolationFragment::Expr(sub_expr));
                }
            }
        }

        Ok(Expr::StringInterpolation { fragments, span })
    }
}

/// Helper for categorizing infix operators.
enum InfixKind {
    Pipe,
    Binary(BinaryOp),
    Range,
}
