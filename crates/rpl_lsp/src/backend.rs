//! # rpl_lsp::backend — LSP Backend implementation for RPL
//!
//! Handles JSON-RPC communication, document lifecycle, hover requests,
//! diagnostics computation, and semantic token delta encoding.

use std::collections::HashMap;
use std::sync::RwLock;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::{
    compute_diagnostics, compute_semantic_tokens, get_hover_for_word, get_word_at_position,
    SUPPORTED_TOKEN_TYPES,
};

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
                        .unwrap_or("0.2+3")
                        .to_string(),
                ),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: WorkDoneProgressOptions::default(),
                            legend: SemanticTokensLegend {
                                token_types: SUPPORTED_TOKEN_TYPES.to_vec(),
                                token_modifiers: vec![],
                            },
                            range: None,
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
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

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let doc_content = self
            .documents
            .read()
            .ok()
            .and_then(|docs| docs.get(&uri).cloned());

        if let Some(content) = doc_content {
            let tokens = compute_semantic_tokens(&content);
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens,
            })));
        }

        Ok(None)
    }
}
