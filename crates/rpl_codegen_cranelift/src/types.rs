//! Type mappings from RPL AST types to Cranelift machine types.

use std::collections::HashMap;
use cranelift::prelude::types;
use rpl_ast::Type;

/// Single field layout inside a struct.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldLayout {
    /// Field identifier.
    pub name: String,
    /// RPL type of the field.
    pub rpl_type: Type,
    /// Cranelift machine type representing the field.
    pub cl_type: types::Type,
    /// Byte offset within the struct allocation.
    pub offset: u32,
    /// Byte size of the field.
    pub size: u32,
}

/// Composite struct memory layout.
#[derive(Debug, Clone, PartialEq)]
pub struct StructLayout {
    /// Struct identifier name.
    pub name: String,
    /// Fields in memory order.
    pub fields: Vec<FieldLayout>,
    /// Total byte size of the struct including trailing alignment padding.
    pub total_size: u32,
    /// Struct alignment in bytes.
    pub alignment: u32,
}

impl StructLayout {
    /// Finds a field layout by name.
    pub fn get_field(&self, name: &str) -> Option<&FieldLayout> {
        self.fields.iter().find(|f| f.name == name)
    }
}

/// Maps an RPL AST type to its corresponding Cranelift primitive or pointer type.
pub fn rpl_to_cl_type(ty: &Type, ptr_type: types::Type) -> types::Type {
    match ty {
        Type::Int | Type::Int64 | Type::UInt64 => types::I64,
        Type::Int32 | Type::UInt32 => types::I32,
        Type::Int16 | Type::UInt16 => types::I16,
        Type::Int8 | Type::UInt8 | Type::Byte => types::I8,
        Type::Float => types::F64,
        Type::Float32 => types::F32,
        Type::Bool => types::I8, // 0 = false, 1 = true
        Type::Trit => types::I8, // -1 = false, 0 = unknown, 1 = true
        Type::String | Type::File => ptr_type,
        Type::Named(_) => ptr_type,
        Type::List(_) | Type::Map(_, _) | Type::Option(_) | Type::Result(_, _) | Type::Channel(_) => {
            ptr_type
        }
    }
}

/// Returns the byte size of an RPL type.
pub fn type_size(
    ty: &Type,
    ptr_size: u32,
    structs: &HashMap<String, StructLayout>,
) -> u32 {
    match ty {
        Type::Int | Type::Int64 | Type::UInt64 | Type::Float => 8,
        Type::Int32 | Type::UInt32 | Type::Float32 => 4,
        Type::Int16 | Type::UInt16 => 2,
        Type::Int8 | Type::UInt8 | Type::Byte | Type::Bool | Type::Trit => 1,
        Type::String | Type::File => ptr_size,
        Type::Named(name) => {
            if let Some(layout) = structs.get(name) {
                layout.total_size
            } else {
                ptr_size
            }
        }
        Type::List(_) | Type::Map(_, _) | Type::Option(_) | Type::Result(_, _) | Type::Channel(_) => {
            ptr_size
        }
    }
}

/// Returns the byte alignment of an RPL type.
pub fn type_align(
    ty: &Type,
    ptr_size: u32,
    structs: &HashMap<String, StructLayout>,
) -> u32 {
    match ty {
        Type::Int | Type::Int64 | Type::UInt64 | Type::Float => 8,
        Type::Int32 | Type::UInt32 | Type::Float32 => 4,
        Type::Int16 | Type::UInt16 => 2,
        Type::Int8 | Type::UInt8 | Type::Byte | Type::Bool | Type::Trit => 1,
        Type::String | Type::File => ptr_size,
        Type::Named(name) => {
            if let Some(layout) = structs.get(name) {
                layout.alignment
            } else {
                ptr_size
            }
        }
        Type::List(_) | Type::Map(_, _) | Type::Option(_) | Type::Result(_, _) | Type::Channel(_) => {
            ptr_size
        }
    }
}

/// Computes the aligned memory layout for a declared struct.
pub fn compute_struct_layout(
    name: &str,
    fields: &[rpl_ast::Field],
    ptr_type: types::Type,
    existing_structs: &HashMap<String, StructLayout>,
) -> StructLayout {
    let ptr_size = ptr_type.bytes();
    let mut current_offset = 0u32;
    let mut max_align = 1u32;
    let mut field_layouts = Vec::with_capacity(fields.len());

    for field in fields {
        let f_size = type_size(&field.field_type, ptr_size, existing_structs);
        let f_align = type_align(&field.field_type, ptr_size, existing_structs).max(1);
        max_align = max_align.max(f_align);

        // Align current_offset up to f_align
        if !current_offset.is_multiple_of(f_align) {
            current_offset += f_align - (current_offset % f_align);
        }

        let cl_type = rpl_to_cl_type(&field.field_type, ptr_type);
        field_layouts.push(FieldLayout {
            name: field.name.clone(),
            rpl_type: field.field_type.clone(),
            cl_type,
            offset: current_offset,
            size: f_size,
        });

        current_offset += f_size;
    }

    // Align total size up to struct max alignment
    if !current_offset.is_multiple_of(max_align) {
        current_offset += max_align - (current_offset % max_align);
    }

    StructLayout {
        name: name.to_string(),
        fields: field_layouts,
        total_size: current_offset.max(1),
        alignment: max_align,
    }
}
