//! Recursive descent parser for RPL statements, declarations, blocks, and types.

use rpl_ast::{
    Block, Expr, Field, Literal, MatchCase, Param, Pattern, Program, Span, Stmt, TritValue, Type,
};
use rpl_lexer::Token;

use crate::error::ParserError;
use crate::Parser;

impl<'a> Parser<'a> {
    /// Parses a complete RPL program consisting of top-level statements.
    pub fn parse_program_ast(&mut self) -> Result<Program, ParserError> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() {
            let stmt = self.parse_stmt()?;
            statements.push(stmt);
            self.skip_newlines();
        }

        let span = if let (Some(first), Some(last)) = (statements.first(), statements.last()) {
            first.span().combine(last.span())
        } else {
            self.eof_span
        };

        Ok(Program::new(statements, span))
    }

    /// Parses a sequence of statements delimited by an opening `:` and terminated by `Token::End`.
    pub fn parse_block(&mut self, start_span: Span) -> Result<Block, ParserError> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while !self.is_at_end()
            && !matches!(
                self.peek(),
                Some(Token::End | Token::Else | Token::Case)
            )
        {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.skip_newlines();
        }

        let end_span = self.peek_span();
        let span = if let (Some(first), Some(last)) = (stmts.first(), stmts.last()) {
            first.span().combine(last.span())
        } else {
            start_span.combine(end_span)
        };

        Ok(Block::new(stmts, span))
    }

    /// Parses a single RPL statement or declaration.
    pub fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.skip_newlines();

        let (token, _span) = match self.peek_with_span() {
            Some((t, s)) => (t.clone(), s),
            None => {
                return Err(ParserError::UnexpectedEof {
                    expected: "statement".into(),
                    span: self.eof_span,
                });
            }
        };

        match token {
            Token::Let => self.parse_let_stmt(),
            Token::Fn => self.parse_fn_decl(),
            Token::Type => self.parse_type_decl(),
            Token::If => self.parse_if_stmt(),
            Token::Match => self.parse_match_stmt(),
            Token::For => self.parse_for_stmt(false),
            Token::Parallel => {
                self.advance();
                self.expect_token(&Token::For, "'for'")?;
                self.parse_for_stmt(true)
            }
            Token::Spawn => self.parse_spawn_stmt(),
            Token::Return => self.parse_return_stmt(),
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    /// Parses `let name [: Type] = expr` or `let mut name [: Type] = expr`.
    fn parse_let_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start_span = self.expect_token(&Token::Let, "'let'")?;
        let is_mut = self.match_token(&Token::Mut);

        let (name, _name_span) = self.expect_ident("variable name")?;
        let type_annot = if self.match_token(&Token::Colon) {
            let (ty, _) = self.parse_type_with_span()?;
            Some(ty)
        } else {
            None
        };

        self.expect_token(&Token::Assign, "'='")?;
        let value = self.parse_expr()?;
        let span = start_span.combine(value.span());

        if is_mut {
            Ok(Stmt::MutLet {
                name,
                type_annot,
                value,
                span,
            })
        } else {
            Ok(Stmt::Let {
                name,
                type_annot,
                value,
                span,
            })
        }
    }

    /// Parses `fn name(param1: T1, ...) [-> RetType]: body end`.
    fn parse_fn_decl(&mut self) -> Result<Stmt, ParserError> {
        let start_span = self.expect_token(&Token::Fn, "'fn'")?;
        let (name, _) = self.expect_ident("function name")?;

        self.expect_token(&Token::LParen, "'('")?;
        let mut params = Vec::new();
        self.skip_newlines();

        while !self.check(&Token::RParen) && !self.is_at_end() {
            let (param_name, param_span) = self.expect_ident("parameter name")?;
            self.expect_token(&Token::Colon, "':'")?;
            let (param_type, type_span) = self.parse_type_with_span()?;
            let span = param_span.combine(type_span);
            params.push(Param::new(param_name, param_type, span));
            self.skip_newlines();

            if self.match_token(&Token::Comma) {
                self.skip_newlines();
                continue;
            }
            break;
        }

        self.expect_token(&Token::RParen, "')'")?;

        let return_type = if self.match_token(&Token::Arrow) {
            let (ty, _) = self.parse_type_with_span()?;
            Some(ty)
        } else {
            None
        };

        let colon_span = self.expect_token(&Token::Colon, "':'")?;
        self.push_block("fn", Some(&name), colon_span);
        let body = self.parse_block(colon_span)?;

        let end_span = self.expect_block_end()?;
        Ok(Stmt::FnDecl {
            name,
            params,
            return_type,
            body,
            span: start_span.combine(end_span),
        })
    }

    /// Parses `type Name: field1: Type ... end`.
    fn parse_type_decl(&mut self) -> Result<Stmt, ParserError> {
        let start_span = self.expect_token(&Token::Type, "'type'")?;
        let (name, _) = self.expect_ident("type name")?;
        let colon_span = self.expect_token(&Token::Colon, "':'")?;
        self.push_block("type", Some(&name), colon_span);

        let mut fields = Vec::new();
        self.skip_newlines();

        while !self.check(&Token::End) && !self.is_at_end() {
            let (field_name, field_span) = self.expect_ident("field name")?;
            self.expect_token(&Token::Colon, "':'")?;
            let (field_type, type_span) = self.parse_type_with_span()?;
            let span = field_span.combine(type_span);
            fields.push(Field::new(field_name, field_type, span));
            self.skip_newlines();
        }

        let end_span = self.expect_block_end()?;
        Ok(Stmt::TypeDecl {
            name,
            fields,
            span: start_span.combine(end_span),
        })
    }

    /// Parses `if condition: then_branch [else if condition: ...] [else: else_branch] end`.
    fn parse_if_stmt(&mut self) -> Result<Stmt, ParserError> {
        self.parse_if_helper(false)
    }

    fn parse_if_helper(&mut self, is_chained: bool) -> Result<Stmt, ParserError> {
        let start_span = self.expect_token(&Token::If, "'if'")?;
        let condition = self.parse_expr()?;
        let colon_span = self.expect_token(&Token::Colon, "':'")?;

        if !is_chained {
            self.push_block("if", None, colon_span);
        }

        let then_branch = self.parse_block(colon_span)?;

        let (else_branch, end_span) = if self.match_token(&Token::Else) {
            if self.check(&Token::If) {
                // Chained `else if`: parse nested if statement
                let nested_if = self.parse_if_helper(true)?;
                let end = if !is_chained {
                    self.expect_block_end()?
                } else {
                    nested_if.span()
                };
                let block = Block::new(vec![nested_if], start_span.combine(end));
                (Some(block), end)
            } else {
                let else_colon_span = self.expect_token(&Token::Colon, "':'")?;
                let else_branch = self.parse_block(else_colon_span)?;
                let end = if !is_chained {
                    self.expect_block_end()?
                } else {
                    else_branch.span
                };
                (Some(else_branch), end)
            }
        } else {
            let end = if !is_chained {
                self.expect_block_end()?
            } else {
                then_branch.span
            };
            (None, end)
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span: start_span.combine(end_span),
        })
    }

    /// Parses `match subject: case pattern [if guard]: body ... end`.
    fn parse_match_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start_span = self.expect_token(&Token::Match, "'match'")?;
        let subject = self.parse_expr()?;
        let colon_span = self.expect_token(&Token::Colon, "':'")?;
        self.push_block("match", None, colon_span);

        let mut cases = Vec::new();
        self.skip_newlines();

        while self.match_token(&Token::Case) {
            let case_start = self.peek_span();
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(&Token::If) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            let case_colon_span = self.expect_token(&Token::Colon, "':'")?;
            let body = self.parse_block(case_colon_span)?;
            let span = case_start.combine(body.span);

            cases.push(MatchCase {
                pattern,
                guard,
                body,
                span,
            });
            self.skip_newlines();
        }

        let end_span = self.expect_block_end()?;
        Ok(Stmt::Match {
            subject,
            cases,
            span: start_span.combine(end_span),
        })
    }

    /// Parses `for item in iterator: body end` or `parallel for item in iterator: body end`.
    fn parse_for_stmt(&mut self, is_parallel: bool) -> Result<Stmt, ParserError> {
        let start_span = self.peek_span();
        if !is_parallel {
            self.expect_token(&Token::For, "'for'")?;
        }

        // Support single or multiple loop variables (`for word in tokens` or `for word, count in frequency_map`)
        let (first_var, _) = self.expect_ident("loop variable name")?;
        let mut item_name = first_var;
        while self.match_token(&Token::Comma) {
            let (next_var, _) = self.expect_ident("loop variable name")?;
            item_name.push_str(", ");
            item_name.push_str(&next_var);
        }

        self.expect_token(&Token::In, "'in'")?;
        let iterator = self.parse_expr()?;
        let colon_span = self.expect_token(&Token::Colon, "':'")?;
        let alt = if is_parallel { Some("parallel") } else { None };
        self.push_block("for", alt, colon_span);
        let body = self.parse_block(colon_span)?;

        let end_span = self.expect_block_end()?;
        let span = start_span.combine(end_span);

        if is_parallel {
            Ok(Stmt::ParallelFor {
                item_name,
                iterator,
                body,
                span,
            })
        } else {
            Ok(Stmt::For {
                item_name,
                iterator,
                body,
                span,
            })
        }
    }

    /// Parses `spawn: body end`.
    fn parse_spawn_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start_span = self.expect_token(&Token::Spawn, "'spawn'")?;
        let colon_span = self.expect_token(&Token::Colon, "':'")?;
        self.push_block("spawn", None, colon_span);
        let body = self.parse_block(colon_span)?;

        let end_span = self.expect_block_end()?;
        Ok(Stmt::Spawn {
            body,
            span: start_span.combine(end_span),
        })
    }

    /// Parses `return [expr]`.
    fn parse_return_stmt(&mut self) -> Result<Stmt, ParserError> {
        let start_span = self.expect_token(&Token::Return, "'return'")?;

        // If next token is newline, block delimiter, or EOF, this is an empty return
        if self.is_at_end()
            || matches!(
                self.peek(),
                Some(Token::Newline | Token::End | Token::Else | Token::Case)
            )
        {
            return Ok(Stmt::Return(None, start_span));
        }

        let value = self.parse_expr()?;
        let span = start_span.combine(value.span());
        Ok(Stmt::Return(Some(value), span))
    }

    /// Parses either an assignment `target = value` or a standalone expression statement.
    /// Also handles command-style invocations like `print "..."`.
    fn parse_expr_or_assign_stmt(&mut self) -> Result<Stmt, ParserError> {
        // Special case: command-like invocation e.g. `print "message"` or `print value`
        if let Some(Token::Ident(name)) = self.peek() {
            if let Some(next) = self.peek_next() {
                // If next is an expression starter on the same line (and not `=`, `:`, `.`, `(`, `[`)
                let is_callee_followed_by_arg = matches!(
                    next,
                    Token::String(_)
                        | Token::Int(_)
                        | Token::Float(_)
                        | Token::True
                        | Token::False
                        | Token::Unknown
                );

                if is_callee_followed_by_arg {
                    let callee_name = name.clone();
                    let callee_span = self.advance().1;
                    let callee = Expr::Identifier(callee_name, callee_span);
                    let arg = self.parse_expr()?;
                    let span = callee_span.combine(arg.span());
                    return Ok(Stmt::Expr(Expr::Call {
                        callee: Box::new(callee),
                        args: vec![arg],
                        span,
                    }));
                }
            }
        }

        let expr = self.parse_expr()?;

        if self.match_token(&Token::Assign) {
            // Verify assignment target
            match &expr {
                Expr::Identifier(..) | Expr::MemberAccess { .. } | Expr::Index { .. } => {}
                _ => {
                    return Err(ParserError::InvalidAssignmentTarget { span: expr.span() });
                }
            }

            let value = self.parse_expr()?;
            let span = expr.span().combine(value.span());
            return Ok(Stmt::Assign {
                target: expr,
                value,
                span,
            });
        }

        Ok(Stmt::Expr(expr))
    }

    /// Parses pattern matching patterns in `match` statements.
    fn parse_pattern(&mut self) -> Result<Pattern, ParserError> {
        let (token, span) = self.advance();

        match token {
            Token::True => Ok(Pattern::Literal(Literal::Bool(true), span)),
            Token::False => Ok(Pattern::Literal(Literal::Bool(false), span)),
            Token::Unknown => Ok(Pattern::Literal(Literal::Trit(TritValue::Unknown), span)),
            Token::Int(n) | Token::HexInt(n) | Token::BinaryInt(n) => {
                Ok(Pattern::Literal(Literal::Int(n), span))
            }
            Token::Float(f) => Ok(Pattern::Literal(Literal::Float(f), span)),
            Token::String(s) => Ok(Pattern::Literal(Literal::String(s), span)),
            Token::Ident(name) => {
                if name == "_" {
                    return Ok(Pattern::Wildcard(span));
                }

                // Constructor pattern: `Ok(val)` or `Error(err)`
                if self.match_token(&Token::LParen) {
                    let mut args = Vec::new();
                    self.skip_newlines();

                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        let arg = self.parse_pattern()?;
                        args.push(arg);
                        self.skip_newlines();

                        if self.match_token(&Token::Comma) {
                            self.skip_newlines();
                            continue;
                        }
                        break;
                    }

                    let rparen_span = self.expect_token(&Token::RParen, "')'")?;
                    return Ok(Pattern::Constructor {
                        name,
                        args,
                        span: span.combine(rparen_span),
                    });
                }

                Ok(Pattern::Identifier(name, span))
            }
            other => Err(ParserError::UnexpectedToken {
                expected: "pattern (literal, identifier, constructor, or wildcard)".into(),
                found: format!("{other}"),
                span,
            }),
        }
    }

    /// Parses a type annotation returning the `Type` and its `Span`.
    pub fn parse_type_with_span(&mut self) -> Result<(Type, Span), ParserError> {
        let (name, span) = self.expect_ident("type name")?;

        if self.match_token(&Token::LBracket) {
            let mut inner = Vec::new();
            self.skip_newlines();

            while !self.check(&Token::RBracket) && !self.is_at_end() {
                let (inner_ty, _) = self.parse_type_with_span()?;
                inner.push(inner_ty);
                self.skip_newlines();

                if self.match_token(&Token::Comma) {
                    self.skip_newlines();
                    continue;
                }
                break;
            }

            let rbracket_span = self.expect_token(&Token::RBracket, "']'")?;
            let total_span = span.combine(rbracket_span);

            let ty = match name.as_str() {
                "List" if inner.len() == 1 => Type::List(Box::new(inner.remove(0))),
                "Option" if inner.len() == 1 => Type::Option(Box::new(inner.remove(0))),
                "Channel" if inner.len() == 1 => Type::Channel(Box::new(inner.remove(0))),
                "Map" if inner.len() == 2 => {
                    let v = inner.remove(1);
                    let k = inner.remove(0);
                    Type::Map(Box::new(k), Box::new(v))
                }
                "Result" if inner.len() == 2 => {
                    let e = inner.remove(1);
                    let t = inner.remove(0);
                    Type::Result(Box::new(t), Box::new(e))
                }
                _ => {
                    let formatted = format!(
                        "{name}[{}]",
                        inner
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    Type::Named(formatted)
                }
            };

            Ok((ty, total_span))
        } else {
            let ty = match name.as_str() {
                "Int" => Type::Int,
                "Int8" => Type::Int8,
                "Int16" => Type::Int16,
                "Int32" => Type::Int32,
                "Int64" => Type::Int64,
                "UInt8" => Type::UInt8,
                "UInt16" => Type::UInt16,
                "UInt32" => Type::UInt32,
                "UInt64" => Type::UInt64,
                "Float" => Type::Float,
                "Float32" => Type::Float32,
                "Bool" => Type::Bool,
                "Trit" => Type::Trit,
                "String" => Type::String,
                "Byte" => Type::Byte,
                "File" => Type::File,
                _ => Type::Named(name),
            };

            Ok((ty, span))
        }
    }

    /// Convenience wrapper to parse a type annotation.
    pub fn parse_type(&mut self) -> Result<Type, ParserError> {
        self.parse_type_with_span().map(|(ty, _)| ty)
    }
}
