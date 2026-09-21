//! Literal values and first-class ternary logic in RPL.

use std::fmt;
use std::ops::{BitAnd, BitOr, Not};

/// First-class ternary logic state (Kleene 3-state logic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TritValue {
    /// Confirmed positive (+1).
    True,
    /// Confirmed negative (-1).
    False,
    /// Undetermined, pending, or missing state (0).
    #[default]
    Unknown,
}

impl TritValue {
    /// Converts a standard boolean into a `TritValue`.
    pub const fn from_bool(val: bool) -> Self {
        if val {
            Self::True
        } else {
            Self::False
        }
    }

    /// Returns `Some(bool)` if definite (`True` or `False`), or `None` if `Unknown`.
    pub const fn to_bool(self) -> Option<bool> {
        match self {
            Self::True => Some(true),
            Self::False => Some(false),
            Self::Unknown => None,
        }
    }

    /// Returns `true` if this value is `True`.
    pub const fn is_true(self) -> bool {
        matches!(self, Self::True)
    }

    /// Returns `true` if this value is `False`.
    pub const fn is_false(self) -> bool {
        matches!(self, Self::False)
    }

    /// Returns `true` if this value is `Unknown`.
    pub const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl fmt::Display for TritValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl Not for TritValue {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

impl BitAnd for TritValue {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::True, Self::True) => Self::True,
            _ => Self::Unknown,
        }
    }
}

impl BitOr for TritValue {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::False, Self::False) => Self::False,
            _ => Self::Unknown,
        }
    }
}

/// Primitive literal constants in RPL source code.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// 64-bit signed integer literal.
    Int(i64),
    /// 64-bit IEEE 754 floating-point literal.
    Float(f64),
    /// UTF-8 string literal.
    String(String),
    /// 2-state boolean literal.
    Bool(bool),
    /// 3-state ternary logic literal (`true`, `false`, `unknown`).
    Trit(TritValue),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => {
                if v.fract() == 0.0 {
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Self::String(v) => write!(f, "\"{v}\""),
            Self::Bool(v) => write!(f, "{v}"),
            Self::Trit(v) => write!(f, "{v}"),
        }
    }
}
