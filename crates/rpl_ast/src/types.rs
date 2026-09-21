//! Strongly-typed representations of RPL types.

use std::fmt;

/// RPL type system representations covering primitives, generics, and user-defined types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Integer primitives
    /// Architecture-default 64-bit signed integer (`Int`).
    Int,
    /// Fixed-width 8-bit signed integer (`Int8`).
    Int8,
    /// Fixed-width 16-bit signed integer (`Int16`).
    Int16,
    /// Fixed-width 32-bit signed integer (`Int32`).
    Int32,
    /// Fixed-width 64-bit signed integer (`Int64`).
    Int64,

    // Unsigned integer primitives
    /// Fixed-width 8-bit unsigned integer (`UInt8`).
    UInt8,
    /// Fixed-width 16-bit unsigned integer (`UInt16`).
    UInt16,
    /// Fixed-width 32-bit unsigned integer (`UInt32`).
    UInt32,
    /// Fixed-width 64-bit unsigned integer (`UInt64`).
    UInt64,

    // Floating-point primitives
    /// 64-bit IEEE 754 floating-point number (`Float`).
    Float,
    /// 32-bit IEEE 754 floating-point number (`Float32`).
    Float32,

    // Logical primitives
    /// Standard 2-state boolean (`Bool`).
    Bool,
    /// Kleene 3-state ternary logic (`Trit`).
    Trit,

    // Textual primitive
    /// UTF-8 compliant string (`String`).
    String,

    // Byte primitive
    /// Canonical alias for `UInt8` (`Byte`).
    Byte,

    // Generics / composite types
    /// Homogeneous growable list (`List[T]`).
    List(Box<Type>),
    /// Hash map key-value store (`Map[K, V]`).
    Map(Box<Type>, Box<Type>),
    /// Optional value representation (`Option[T]`).
    Option(Box<Type>),
    /// Value-or-failure representation (`Result[T, E]`).
    Result(Box<Type>, Box<Type>),
    /// Thread-safe lock-free channel (`Channel[T]`).
    Channel(Box<Type>),

    // Custom named types
    /// User-defined struct or enum type name (`Named(String)`).
    Named(String),
}

impl Type {
    /// Returns `true` if the type is an integer (signed or unsigned).
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::Int8
                | Self::Int16
                | Self::Int32
                | Self::Int64
                | Self::UInt8
                | Self::UInt16
                | Self::UInt32
                | Self::UInt64
                | Self::Byte
        )
    }

    /// Returns `true` if the type is a floating-point number.
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float | Self::Float32)
    }

    /// Returns `true` if the type is numeric (integer or float).
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Returns `true` if the type is a primitive scalar type.
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::Int8
                | Self::Int16
                | Self::Int32
                | Self::Int64
                | Self::UInt8
                | Self::UInt16
                | Self::UInt32
                | Self::UInt64
                | Self::Float
                | Self::Float32
                | Self::Bool
                | Self::Trit
                | Self::String
                | Self::Byte
        )
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Int8 => write!(f, "Int8"),
            Self::Int16 => write!(f, "Int16"),
            Self::Int32 => write!(f, "Int32"),
            Self::Int64 => write!(f, "Int64"),
            Self::UInt8 => write!(f, "UInt8"),
            Self::UInt16 => write!(f, "UInt16"),
            Self::UInt32 => write!(f, "UInt32"),
            Self::UInt64 => write!(f, "UInt64"),
            Self::Float => write!(f, "Float"),
            Self::Float32 => write!(f, "Float32"),
            Self::Bool => write!(f, "Bool"),
            Self::Trit => write!(f, "Trit"),
            Self::String => write!(f, "String"),
            Self::Byte => write!(f, "Byte"),
            Self::List(elem) => write!(f, "List[{elem}]"),
            Self::Map(key, val) => write!(f, "Map[{key}, {val}]"),
            Self::Option(elem) => write!(f, "Option[{elem}]"),
            Self::Result(ok, err) => write!(f, "Result[{ok}, {err}]"),
            Self::Channel(elem) => write!(f, "Channel[{elem}]"),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}
