//! # rpl_ast — Abstract Syntax Tree for RPL (Running Pseudo Language)
//!
//! This crate provides strongly-typed, immutable AST definitions for the RPL
//! compiler pipeline. Structural curly braces `{}` and semicolons `;` are strictly
//! absent from the language grammar, and block scoping is governed by `:` and `end`.
//!
//! ## Modules
//! - [`span`]: Byte offset and line/column diagnostics tracking.
//! - [`literal`]: Scalar constants and first-class 3-state logic (`TritValue`).
//! - [`types`]: Primitive, generic, and user-defined type definitions.
//! - [`op`]: Binary and unary operators.
//! - [`expr`]: Recursive expression trees and string interpolation fragments.
//! - [`stmt`]: Statement and declaration AST nodes, pattern matching, and blocks.
//! - [`error`]: AST-related error types using `thiserror`.

pub mod error;
pub mod expr;
pub mod literal;
pub mod op;
pub mod span;
pub mod stmt;
pub mod types;

// Re-export core AST types at crate root for ergonomic access
pub use error::AstError;
pub use expr::{Expr, InterpolationFragment};
pub use literal::{Literal, TritValue};
pub use op::{BinaryOp, UnaryOp};
pub use span::Span;
pub use stmt::{Block, Field, MatchCase, Param, Pattern, Program, Stmt};
pub use types::Type;
