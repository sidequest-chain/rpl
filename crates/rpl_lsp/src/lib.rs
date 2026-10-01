//! # rpl_lsp — Language Server Protocol (LSP) for Running Pseudo Language (RPL)
//!
//! Provides real-time syntax checking, Kleene ternary logic type diagnostics,
//! and hover documentation for IDE integrations (VS Code, Antigravity IDE, Zed).

use rpl_ast::Span;
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, Server};

/// Converts an RPL 1-indexed `Span` into a 0-indexed LSP `Range`.
pub fn span_to_range(span: &Span) -> Range {
    let start_line = span.start_line.saturating_sub(1) as u32;
    let start_col = span.start_col.saturating_sub(1) as u32;
    let end_line = span.end_line.saturating_sub(1) as u32;
    let mut end_col = span.end_col.saturating_sub(1) as u32;

    // Ensure the range encloses at least 1 character so IDEs display the diagnostic underline
    if start_line == end_line && start_col >= end_col {
        end_col = start_col + 1;
    }

    Range {
        start: Position {
            line: start_line,
            character: start_col,
        },
        end: Position {
            line: end_line,
            character: end_col,
        },
    }
}

/// Analyzes RPL source text and generates LSP diagnostics.
///
/// Returns:
/// - Parsing errors prefixed with `[- - -]`
/// - Typechecking errors prefixed with `[+ - -]`
/// - An empty vector if the program is completely valid
pub fn compute_diagnostics(source: &str) -> Vec<Diagnostic> {
    match rpl_parser::parse_program(source) {
        Err(parser_err) => {
            let range = span_to_range(&parser_err.span());
            let message = format!("[- - -] {}", parser_err);
            vec![Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("rpl".to_string()),
                message,
                related_information: None,
                tags: None,
                data: None,
            }]
        }
        Ok(program) => match rpl_typechecker::check_program(&program) {
            Err(type_errors) => type_errors
                .into_iter()
                .map(|type_err| {
                    let range = span_to_range(&type_err.span());
                    let message = format!("[+ - -] {}", type_err);
                    Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("rpl".to_string()),
                        message,
                        related_information: None,
                        tags: None,
                        data: None,
                    }
                })
                .collect(),
            Ok(()) => Vec::new(),
        },
    }
}

/// Helper function to extract a word/token from text at a specific 0-based position.
pub fn get_word_at_position(text: &str, pos: Position) -> Option<String> {
    let line = text.lines().nth(pos.line as usize)?;
    let col = pos.character as usize;

    if col > line.len() {
        return None;
    }

    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';

    let start = line[..col]
        .rfind(|c: char| !is_ident_char(c))
        .map(|idx| idx + 1)
        .unwrap_or(0);

    let end = line[col..]
        .find(|c: char| !is_ident_char(c))
        .map(|idx| col + idx)
        .unwrap_or(line.len());

    if start < end {
        Some(line[start..end].to_string())
    } else {
        None
    }
}

/// Helper to generate hover documentation for a keyword, type, or identifier.
pub fn get_hover_for_word(word: &str, source: &str) -> Option<String> {
    // 1. Built-in keywords and primitive types
    let doc = match word {
        "Trit" => "**`type Trit`**\n\nFirst-class Kleene 3-valued logic type supporting `true`, `false`, and `unknown` with compile-time exhaustiveness checking.",
        "unknown" => "**`unknown`** (Trit literal)\n\nRepresents an indeterminate or uncertain state in Kleene ternary logic algebra.",
        "true" => "**`true`** (Boolean / Trit literal)\n\nRepresents positive truth.",
        "false" => "**`false`** (Boolean / Trit literal)\n\nRepresents negative truth.",
        "Int" | "Int64" => "**`type Int`**\n\n64-bit signed systems integer (`i64`).",
        "Float" => "**`type Float`**\n\n64-bit IEEE-754 double precision floating-point (`f64`).",
        "String" => "**`type String`**\n\nHeap-allocated UTF-8 string with deterministic move semantics.",
        "Bool" => "**`type Bool`**\n\nStandard binary boolean type (`true` or `false`).",
        "Byte" => "**`type Byte`**\n\n8-bit unsigned raw byte value (`u8`).",
        "fn" => "**`fn`** (Keyword)\n\nDeclares a function. Scopes open with `:` and close exclusively with `end` or `end fn`.",
        "let" => "**`let`** (Keyword)\n\nBinds a variable. Variables are immutable by default.",
        "mut" => "**`mut`** (Keyword)\n\nDeclares a mutable variable binding: `let mut name: Type = value`.",
        "match" => "**`match`** (Keyword)\n\nPattern matching construct. Exhaustiveness is strictly enforced for ternary `Trit` types.",
        "type" => "**`type`** (Keyword)\n\nDeclares a composite data structure or struct record.",
        "for" => "**`for`** (Keyword)\n\nIterates over a range: `for i in 0..10: ... end`.",
        "in" => "**`in`** (Keyword)\n\nRange iteration operator used within `for` loops.",
        "spawn" => "**`spawn`** (Keyword)\n\nAsynchronous concurrency block for parallel tasks.",
        "return" => "**`return`** (Keyword)\n\nReturns an expression value from the current function.",
        "if" => "**`if`** (Keyword)\n\nConditional branching block.",
        "else" => "**`else`** (Keyword)\n\nAlternative branch for conditional or match constructs.",
        "end" => "**`end`** (Keyword)\n\nUniversal block terminator in RPL (replaces closing curly braces `}`).",
        "println" => "**`fn println(text: String)`**\n\nStandard runtime function to print text to stdout followed by a newline.",
        "print" => "**`fn print(text: String)`**\n\nStandard runtime function to print text to stdout.",
        "File" => "**`type File`**\n\nOpaque systems file handle backed by a standard stream (`FILE*`).",
        "input" => "**`fn input() -> String`**\n\nReads a line of text from standard input (stdin) with trailing newline stripped.",
        "read_file" => "**`fn read_file(path: String) -> String`**\n\nConvenience helper that reads the entire file into memory and immediately closes it. Returns empty string on error.",
        "write_file" => "**`fn write_file(path: String, content: String) -> Bool`**\n\nConvenience helper that overwrites file with given content and immediately closes it. Returns true on success.",
        "append_file" => "**`fn append_file(path: String, content: String) -> Bool`**\n\nConvenience helper that appends content to file and immediately closes it. Returns true on success.",
        "open_file" => "**`fn open_file(path: String, mode: String) -> File`**\n\nOpens a file stream in specified mode (\"r\", \"w\", \"a\"). Returns a persistent `File` handle.",
        "read_line" => "**`fn read_line(file: File) -> String`**\n\nReads a single line from an open file stream without closing the handle.",
        "write_line" => "**`fn write_line(file: File, line: String) -> Bool`**\n\nWrites a line of text followed by newline to an open file stream and flushes without closing the handle.",
        "close_file" => "**`fn close_file(file: File)`**\n\nCloses the open file stream. Consumes ownership of the file handle.",
        _ => {
            // 2. Search declarations in parsed AST
            if let Ok(prog) = rpl_parser::parse_program(source) {
                for stmt in &prog.statements {
                    match stmt {
                        rpl_ast::Stmt::FnDecl {
                            name,
                            params,
                            return_type,
                            ..
                        } if name == word => {
                            let params_str = params
                                .iter()
                                .map(|p| format!("{}: {}", p.name, p.param_type))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let ret_str = return_type
                                .as_ref()
                                .map(|t| format!(" -> {}", t))
                                .unwrap_or_default();
                            return Some(format!(
                                "```rpl\nfn {}({}){}\n```\nUser-defined RPL function.",
                                name, params_str, ret_str
                            ));
                        }
                        rpl_ast::Stmt::TypeDecl { name, fields, .. } if name == word => {
                            let fields_str = fields
                                .iter()
                                .map(|f| format!("    {}: {}", f.name, f.field_type))
                                .collect::<Vec<_>>()
                                .join("\n");
                            return Some(format!(
                                "```rpl\ntype {}:\n{}\nend\n```\nUser-defined RPL struct.",
                                name, fields_str
                            ));
                        }
                        _ => {}
                    }
                }
            }
            return None;
        }
    };

    Some(doc.to_string())
}

/// Supported Semantic Token Types advertised to LSP clients (Zed, VS Code, Helix, Neovim).
///
/// Order conforms strictly to LSP 3.17 legend:
/// - 0: `keyword` (fn, end, let, mut, if, else, match, for, in, return, spawn, type)
/// - 1: `type` (Int, Int64, Float, String, Bool, Trit, Byte, custom record types)
/// - 2: `variable` (variable names, dollar identifiers)
/// - 3: `function` (function names)
/// - 4: `string` (text literals)
/// - 5: `number` (numeric literals)
/// - 6: `operator` (trit literals +, -, ?, arithmetic and comparison ops)
/// - 7: `comment` (// and /* */)
pub const SUPPORTED_TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,  // 0
    SemanticTokenType::TYPE,     // 1
    SemanticTokenType::VARIABLE, // 2
    SemanticTokenType::FUNCTION, // 3
    SemanticTokenType::STRING,   // 4
    SemanticTokenType::NUMBER,   // 5
    SemanticTokenType::OPERATOR, // 6
    SemanticTokenType::COMMENT,  // 7
];

#[derive(Debug, Clone)]
struct RawSemanticToken {
    line: u32,
    col: u32,
    length: u32,
    token_type: u32,
}

fn classify_token(
    tok: &rpl_lexer::Token,
    user_types: &std::collections::HashSet<String>,
    user_fns: &std::collections::HashSet<String>,
) -> Option<u32> {
    match tok {
        // 0: keyword (fn, end, let, mut, if, else, match, for, in, return, spawn, type, etc.)
        rpl_lexer::Token::Fn
        | rpl_lexer::Token::End
        | rpl_lexer::Token::Type
        | rpl_lexer::Token::Let
        | rpl_lexer::Token::Mut
        | rpl_lexer::Token::Match
        | rpl_lexer::Token::Case
        | rpl_lexer::Token::Spawn
        | rpl_lexer::Token::Return
        | rpl_lexer::Token::If
        | rpl_lexer::Token::Else
        | rpl_lexer::Token::For
        | rpl_lexer::Token::In
        | rpl_lexer::Token::While
        | rpl_lexer::Token::Break
        | rpl_lexer::Token::Continue
        | rpl_lexer::Token::As
        | rpl_lexer::Token::Channel
        | rpl_lexer::Token::Const
        | rpl_lexer::Token::Is
        | rpl_lexer::Token::Parallel
        | rpl_lexer::Token::Yield => Some(0), // KEYWORD

        // 4: string (text literals)
        rpl_lexer::Token::String(_) => Some(4), // STRING

        // 5: number (numeric literals)
        rpl_lexer::Token::Int(_)
        | rpl_lexer::Token::HexInt(_)
        | rpl_lexer::Token::BinaryInt(_)
        | rpl_lexer::Token::Float(_) => Some(5), // NUMBER

        // 6: operator (trit literals +, -, ?, arithmetic and comparison ops)
        rpl_lexer::Token::True
        | rpl_lexer::Token::False
        | rpl_lexer::Token::Unknown
        | rpl_lexer::Token::Plus
        | rpl_lexer::Token::Minus
        | rpl_lexer::Token::Star
        | rpl_lexer::Token::Slash
        | rpl_lexer::Token::Percent
        | rpl_lexer::Token::EqEq
        | rpl_lexer::Token::NotEq
        | rpl_lexer::Token::Lt
        | rpl_lexer::Token::LtEq
        | rpl_lexer::Token::Gt
        | rpl_lexer::Token::GtEq
        | rpl_lexer::Token::Ampersand
        | rpl_lexer::Token::Pipe
        | rpl_lexer::Token::Caret
        | rpl_lexer::Token::Tilde
        | rpl_lexer::Token::Shl
        | rpl_lexer::Token::Shr
        | rpl_lexer::Token::PipeRight
        | rpl_lexer::Token::Arrow
        | rpl_lexer::Token::FatArrow
        | rpl_lexer::Token::Assign
        | rpl_lexer::Token::DotDot
        | rpl_lexer::Token::And
        | rpl_lexer::Token::Or
        | rpl_lexer::Token::Not => Some(6), // OPERATOR

        // 2: variable (dollar identifier string interpolation)
        rpl_lexer::Token::DollarIdent(_) => Some(2), // VARIABLE

        // Identifiers
        rpl_lexer::Token::Ident(name) => match name.as_str() {
            "Trit" | "Int" | "Int64" | "Float" | "String" | "Bool" | "Byte" | "File" => Some(1), // TYPE
            "print" | "println" | "input" | "read_file" | "write_file" | "append_file"
            | "open_file" | "read_line" | "write_line" | "close_file" => Some(3), // FUNCTION
            _ if user_types.contains(name) => Some(1),                        // TYPE
            _ if user_fns.contains(name) => Some(3),                          // FUNCTION
            _ => Some(2),                                                     // VARIABLE
        },

        _ => None,
    }
}

fn extract_comments(source: &str) -> Vec<RawSemanticToken> {
    let mut comments = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut in_block = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_u32 = line_idx as u32;
        let bytes = line.as_bytes();
        let mut idx = 0;

        while idx < bytes.len() {
            if in_block {
                if idx + 1 < bytes.len() && bytes[idx] == b'*' && bytes[idx + 1] == b'/' {
                    let len = (idx + 2) as u32;
                    comments.push(RawSemanticToken {
                        line: line_u32,
                        col: 0,
                        length: len,
                        token_type: 7, // COMMENT
                    });
                    in_block = false;
                    idx += 2;
                } else {
                    idx += 1;
                }
            } else if idx + 1 < bytes.len() && bytes[idx] == b'/' {
                if bytes[idx + 1] == b'/' {
                    let col = idx as u32;
                    let len = (bytes.len() - idx) as u32;
                    comments.push(RawSemanticToken {
                        line: line_u32,
                        col,
                        length: len,
                        token_type: 7, // COMMENT
                    });
                    break;
                } else if bytes[idx + 1] == b'*' {
                    let start_col = idx as u32;
                    idx += 2;
                    let mut closed_on_same_line = false;
                    while idx + 1 < bytes.len() {
                        if bytes[idx] == b'*' && bytes[idx + 1] == b'/' {
                            let len = (idx + 2 - (start_col as usize)) as u32;
                            comments.push(RawSemanticToken {
                                line: line_u32,
                                col: start_col,
                                length: len,
                                token_type: 7, // COMMENT
                            });
                            closed_on_same_line = true;
                            idx += 2;
                            break;
                        }
                        idx += 1;
                    }
                    if !closed_on_same_line {
                        let len = (bytes.len() - (start_col as usize)) as u32;
                        comments.push(RawSemanticToken {
                            line: line_u32,
                            col: start_col,
                            length: len,
                            token_type: 7, // COMMENT
                        });
                        in_block = true;
                        break;
                    }
                } else {
                    idx += 1;
                }
            } else {
                idx += 1;
            }
        }

        if in_block && !comments.iter().any(|c| c.line == line_u32) {
            comments.push(RawSemanticToken {
                line: line_u32,
                col: 0,
                length: bytes.len() as u32,
                token_type: 7, // COMMENT
            });
        }
    }

    comments
}

/// Computes semantic tokens for an RPL source document.
///
/// Encodes token delta coordinates in accordance with the LSP 3.17 specification:
/// `[delta_line, delta_start_col, length, token_type, token_modifiers]`.
pub fn compute_semantic_tokens(source: &str) -> Vec<SemanticToken> {
    let mut raw_tokens = extract_comments(source);

    let mut user_types = std::collections::HashSet::new();
    let mut user_fns = std::collections::HashSet::new();
    if let Ok(prog) = rpl_parser::parse_program(source) {
        for stmt in &prog.statements {
            match stmt {
                rpl_ast::Stmt::TypeDecl { name, .. } => {
                    user_types.insert(name.clone());
                }
                rpl_ast::Stmt::FnDecl { name, .. } => {
                    user_fns.insert(name.clone());
                }
                _ => {}
            }
        }
    }

    let lexer = rpl_lexer::RplLexer::new(source).with_bracketed_newline_filtering(false);
    for (tok, span) in lexer.flatten() {
        if let Some(token_type) = classify_token(&tok, &user_types, &user_fns) {
            let line = span.start_line.saturating_sub(1) as u32;
            let col = span.start_col.saturating_sub(1) as u32;
            let length = if span.start_line == span.end_line {
                span.end_col.saturating_sub(span.start_col) as u32
            } else {
                let first_line_len = source
                    .lines()
                    .nth(line as usize)
                    .map(|l| l.len())
                    .unwrap_or(0);
                first_line_len.saturating_sub(col as usize) as u32
            };

            if length > 0 {
                raw_tokens.push(RawSemanticToken {
                    line,
                    col,
                    length,
                    token_type,
                });
            }
        }
    }

    raw_tokens.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.col.cmp(&b.col)));
    raw_tokens.dedup_by(|a, b| a.line == b.line && a.col == b.col);

    let mut semantic_tokens = Vec::with_capacity(raw_tokens.len());
    let mut prev_line = 0;
    let mut prev_col = 0;

    for tok in raw_tokens {
        let delta_line = tok.line.saturating_sub(prev_line);
        let delta_start = if delta_line == 0 {
            tok.col.saturating_sub(prev_col)
        } else {
            tok.col
        };

        semantic_tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: tok.length,
            token_type: tok.token_type,
            token_modifiers_bitset: 0,
        });

        prev_line = tok.line;
        prev_col = tok.col;
    }

    semantic_tokens
}

pub mod backend;
pub use backend::Backend;

/// Runs the RPL Language Server over standard input and standard output.
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
