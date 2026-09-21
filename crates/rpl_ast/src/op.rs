//! Binary and unary operators supported in RPL syntax.

use std::fmt;

/// Binary operators in RPL expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryOp {
    // Arithmetic operators
    /// Addition (`+`).
    Add,
    /// Subtraction (`-`).
    Sub,
    /// Multiplication (`*`).
    Mul,
    /// Division (`/`).
    Div,
    /// Modulo / remainder (`%`).
    Mod,

    // Relational comparison operators
    /// Equal (`==`).
    Eq,
    /// Not equal (`!=`).
    NotEq,
    /// Less than (`<`).
    Lt,
    /// Less than or equal (`<=`).
    LtEq,
    /// Greater than (`>`).
    Gt,
    /// Greater than or equal (`>=`).
    GtEq,

    // Logical operators
    /// Logical AND (`and`).
    And,
    /// Logical OR (`or`).
    Or,

    // Bitwise operators
    /// Bitwise AND (`&`).
    BitAnd,
    /// Bitwise OR (`|`).
    BitOr,
    /// Bitwise XOR (`^`).
    BitXor,
    /// Bitwise left shift (`<<`).
    Shl,
    /// Bitwise right shift (`>>`).
    Shr,
}

impl BinaryOp {
    /// Returns `true` if this operator is an arithmetic operator (`+`, `-`, `*`, `/`, `%`).
    pub const fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod
        )
    }

    /// Returns `true` if this operator is a comparison operator.
    pub const fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Eq | Self::NotEq | Self::Lt | Self::LtEq | Self::Gt | Self::GtEq
        )
    }

    /// Returns `true` if this operator is a logical operator (`and`, `or`).
    pub const fn is_logical(self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    /// Returns `true` if this operator is a bitwise operator.
    pub const fn is_bitwise(self) -> bool {
        matches!(
            self,
            Self::BitAnd | Self::BitOr | Self::BitXor | Self::Shl | Self::Shr
        )
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Eq => write!(f, "=="),
            Self::NotEq => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::LtEq => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::GtEq => write!(f, ">="),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::BitAnd => write!(f, "&"),
            Self::BitOr => write!(f, "|"),
            Self::BitXor => write!(f, "^"),
            Self::Shl => write!(f, "<<"),
            Self::Shr => write!(f, ">>"),
        }
    }
}

/// Unary operators in RPL expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryOp {
    /// Logical negation (`not`).
    Not,
    /// Arithmetic negation (`-`).
    Neg,
    /// Bitwise NOT / complement (`~`).
    BitNot,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Not => write!(f, "not "),
            Self::Neg => write!(f, "-"),
            Self::BitNot => write!(f, "~"),
        }
    }
}
