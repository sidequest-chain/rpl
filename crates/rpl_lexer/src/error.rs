//! Lexical analysis errors with source spans.

use rpl_ast::Span;
use thiserror::Error;

/// Lexical error kinds occurring during RPL tokenization.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    /// An unexpected or disallowed character was encountered.
    #[error("Unexpected character '{ch}' at {span}")]
    UnexpectedCharacter {
        /// The unexpected character.
        ch: char,
        /// Source code coordinates where the character was encountered.
        span: Span,
    },

    /// A string literal was not closed before the end of the line or file.
    #[error("Unterminated string literal starting at {span}")]
    UnterminatedString {
        /// Source span of the unclosed string literal.
        span: Span,
    },

    /// A numeric literal was malformed or could not be parsed into a numeric type.
    #[error("Invalid number literal at {span}: {message}")]
    InvalidNumberLiteral {
        /// Diagnostic description of the parsing failure.
        message: String,
        /// Source code span of the malformed number literal.
        span: Span,
    },
}

impl LexerError {
    /// Returns the source code span of this lexer error.
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedCharacter { span, .. } => *span,
            Self::UnterminatedString { span } => *span,
            Self::InvalidNumberLiteral { span, .. } => *span,
        }
    }
}
