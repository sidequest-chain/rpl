//! Parser diagnostic errors and span locations.

use rpl_ast::Span;
use rpl_lexer::LexerError;
use thiserror::Error;

/// Syntactic errors produced during RPL parsing.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    /// An unexpected token was encountered when a specific construct was expected.
    #[error("Expected {expected}, but found {found} at {span}")]
    UnexpectedToken {
        /// Expected token or syntactic construct.
        expected: String,
        /// Description of the token actually found.
        found: String,
        /// Source span where the unexpected token occurred.
        span: Span,
    },

    /// Reached unexpected end of input while expecting a token or construct.
    #[error("Unexpected end of file while expecting {expected} at {span}")]
    UnexpectedEof {
        /// Description of what was expected before EOF.
        expected: String,
        /// Source span at EOF.
        span: Span,
    },

    /// A block opened with `:` was never closed with `end`.
    #[error("Unclosed block started at {started_at}, unclosed at {span}")]
    UnclosedBlock {
        /// Span where the block was opened (`:`).
        started_at: Span,
        /// Span where the unclosed block was detected.
        span: Span,
    },

    /// The target of an assignment expression is invalid.
    #[error("Invalid assignment target at {span}")]
    InvalidAssignmentTarget {
        /// Source span of the invalid target.
        span: Span,
    },

    /// Lexical tokenization error encountered during parsing.
    #[error(transparent)]
    LexerError(#[from] LexerError),
}

impl ParserError {
    /// Returns the source code span of this parser error.
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedToken { span, .. } => *span,
            Self::UnexpectedEof { span, .. } => *span,
            Self::UnclosedBlock { span, .. } => *span,
            Self::InvalidAssignmentTarget { span } => *span,
            Self::LexerError(err) => err.span(),
        }
    }
}
