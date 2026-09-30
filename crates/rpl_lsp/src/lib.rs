//! # rpl_lsp — Language Server Protocol (LSP) for Running Pseudo Language (RPL)
//!
//! Provides real-time syntax checking, Kleene ternary logic type diagnostics,
//! and hover documentation for IDE integrations (VS Code, Antigravity IDE, Zed).

use std::collections::HashMap;
use std::sync::RwLock;

use rpl_ast::Span;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

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

/// The RPL Language Server state and backend.
pub struct Backend {
    client: Client,
    documents: RwLock<HashMap<Url, String>>,
}

impl Backend {
    /// Creates a new `Backend` instance bound to the provided LSP client.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    async fn validate_document(&self, uri: Url, text: &str) {
        let diagnostics = compute_diagnostics(text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "rpl-lsp".to_string(),
                version: Some(
                    include_str!("../../../VERSION")
                        .split_whitespace()
                        .next()
                        .unwrap_or("0.2+2")
                        .to_string(),
                ),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "RPL Language Server initialized.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        if let Ok(mut docs) = self.documents.write() {
            docs.insert(uri.clone(), text.clone());
        }

        self.validate_document(uri, &text).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.pop() {
            let text = change.text;

            if let Ok(mut docs) = self.documents.write() {
                docs.insert(uri.clone(), text.clone());
            }

            self.validate_document(uri, &text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = self
            .documents
            .read()
            .ok()
            .and_then(|docs| docs.get(&uri).cloned());

        if let Some(content) = text {
            self.validate_document(uri, &content).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let doc_content = self
            .documents
            .read()
            .ok()
            .and_then(|docs| docs.get(&uri).cloned());

        if let Some(content) = doc_content {
            if let Some(word) = get_word_at_position(&content, pos) {
                if let Some(doc) = get_hover_for_word(&word, &content) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: doc,
                        }),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }
}

/// Runs the RPL Language Server over standard input and standard output.
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
