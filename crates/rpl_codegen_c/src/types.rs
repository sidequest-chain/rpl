//! Type conversion from RPL AST types to C99 native types.

use rpl_ast::Type;

/// Converts an RPL AST `Type` to its corresponding C99 type signature.
pub fn to_c_type(ty: &Type) -> String {
    match ty {
        Type::Int | Type::Int64 => "int64_t".to_string(),
        Type::Int8 => "int8_t".to_string(),
        Type::Int16 => "int16_t".to_string(),
        Type::Int32 => "int32_t".to_string(),
        Type::UInt8 | Type::Byte => "uint8_t".to_string(),
        Type::UInt16 => "uint16_t".to_string(),
        Type::UInt32 => "uint32_t".to_string(),
        Type::UInt64 => "uint64_t".to_string(),
        Type::Float => "double".to_string(),
        Type::Float32 => "float".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Trit => "rpl_trit_t".to_string(),
        Type::String => "const char*".to_string(),
        Type::Named(name) => name.clone(),
        Type::List(elem) => format!("{}*", to_c_type(elem)),
        Type::Map(_, _) => "void*".to_string(),
        Type::Option(elem) => format!("{}*", to_c_type(elem)),
        Type::Result(ok, _) => to_c_type(ok),
        Type::Channel(_) => "void*".to_string(),
    }
}

/// Converts an optional RPL return type into a C99 return type (defaults to `void`).
pub fn to_c_return_type(ret_ty: Option<&Type>) -> String {
    match ret_ty {
        Some(ty) => to_c_type(ty),
        None => "void".to_string(),
    }
}
