//! Runtime C-ABI functions and helpers exposed to JIT-compiled RPL code.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CStr;
use std::io::{self, Write};
use cranelift_jit::JITBuilder;

/// Prints a signed 64-bit integer to stdout followed by a newline.
#[no_mangle]
pub extern "C" fn rpl_jit_print_int(v: i64) {
    println!("{v}");
    let _ = io::stdout().flush();
}

/// Prints a 64-bit floating point value to stdout followed by a newline.
#[no_mangle]
pub extern "C" fn rpl_jit_print_float(v: f64) {
    if v.fract() == 0.0 {
        println!("{v:.1}");
    } else {
        println!("{v}");
    }
    let _ = io::stdout().flush();
}

/// Prints an 8-bit boolean (0 = false, 1 = true) to stdout followed by a newline.
#[no_mangle]
pub extern "C" fn rpl_jit_print_bool(v: i8) {
    println!("{}", if v != 0 { "true" } else { "false" });
    let _ = io::stdout().flush();
}

/// Prints a Trit value (-1 = false, 0 = unknown, 1 = true) to stdout followed by a newline.
#[no_mangle]
pub extern "C" fn rpl_jit_print_trit(v: i8) {
    match v {
        1 => println!("true"),
        -1 => println!("false"),
        _ => println!("unknown"),
    }
    let _ = io::stdout().flush();
}

/// Prints a UTF-8 string slice to stdout followed by a newline.
/// If `len == usize::MAX`, treats `ptr` as a null-terminated C string.
#[no_mangle]
pub extern "C" fn rpl_jit_print_str(ptr: *const u8, len: usize) {
    if ptr.is_null() {
        println!();
        let _ = io::stdout().flush();
        return;
    }

    let s = if len == usize::MAX {
        unsafe { CStr::from_ptr(ptr as *const std::ffi::c_char).to_string_lossy() }
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        String::from_utf8_lossy(bytes)
    };

    println!("{s}");
    let _ = io::stdout().flush();
}

/// Concatenates two string slices or null-terminated strings and returns a null-terminated heap buffer.
#[no_mangle]
pub extern "C" fn rpl_jit_str_concat(
    s1: *const u8,
    len1: usize,
    s2: *const u8,
    len2: usize,
) -> *mut u8 {
    let slice1 = if s1.is_null() {
        &[]
    } else if len1 == usize::MAX {
        unsafe { CStr::from_ptr(s1 as *const _).to_bytes() }
    } else {
        unsafe { std::slice::from_raw_parts(s1, len1) }
    };

    let slice2 = if s2.is_null() {
        &[]
    } else if len2 == usize::MAX {
        unsafe { CStr::from_ptr(s2 as *const _).to_bytes() }
    } else {
        unsafe { std::slice::from_raw_parts(s2, len2) }
    };

    let mut buf = Vec::with_capacity(slice1.len() + slice2.len() + 1);
    buf.extend_from_slice(slice1);
    buf.extend_from_slice(slice2);
    buf.push(0); // Null terminator

    Box::into_raw(buf.into_boxed_slice()) as *mut u8
}

/// Converts an i64 to a heap-allocated null-terminated string.
#[no_mangle]
pub extern "C" fn rpl_jit_int_to_str(v: i64) -> *mut u8 {
    let mut s = v.to_string().into_bytes();
    s.push(0);
    Box::into_raw(s.into_boxed_slice()) as *mut u8
}

/// Converts an f64 to a heap-allocated null-terminated string.
#[no_mangle]
pub extern "C" fn rpl_jit_float_to_str(v: f64) -> *mut u8 {
    let formatted = if v.fract() == 0.0 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    };
    let mut s = formatted.into_bytes();
    s.push(0);
    Box::into_raw(s.into_boxed_slice()) as *mut u8
}

/// Converts a bool (i8) to a static null-terminated string pointer.
#[no_mangle]
pub extern "C" fn rpl_jit_bool_to_str(v: i8) -> *const u8 {
    if v != 0 {
        c"true".as_ptr().cast()
    } else {
        c"false".as_ptr().cast()
    }
}

/// Converts a Trit (i8) to a static null-terminated string pointer.
#[no_mangle]
pub extern "C" fn rpl_jit_trit_to_str(v: i8) -> *const u8 {
    match v {
        1 => c"true".as_ptr().cast(),
        -1 => c"false".as_ptr().cast(),
        _ => c"unknown".as_ptr().cast(),
    }
}

/// Calculates the byte length of a null-terminated string.
#[no_mangle]
pub extern "C" fn rpl_jit_str_len(s: *const u8) -> usize {
    if s.is_null() {
        return 0;
    }
    unsafe { CStr::from_ptr(s as *const _).to_bytes().len() }
}

/// Registers all standard runtime functions with a `JITBuilder`.
pub fn register_runtime_symbols(builder: &mut JITBuilder) {
    builder.symbol("rpl_jit_print_int", rpl_jit_print_int as *const u8);
    builder.symbol("rpl_jit_print_float", rpl_jit_print_float as *const u8);
    builder.symbol("rpl_jit_print_bool", rpl_jit_print_bool as *const u8);
    builder.symbol("rpl_jit_print_trit", rpl_jit_print_trit as *const u8);
    builder.symbol("rpl_jit_print_str", rpl_jit_print_str as *const u8);
    builder.symbol("rpl_jit_str_concat", rpl_jit_str_concat as *const u8);
    builder.symbol("rpl_jit_int_to_str", rpl_jit_int_to_str as *const u8);
    builder.symbol("rpl_jit_float_to_str", rpl_jit_float_to_str as *const u8);
    builder.symbol("rpl_jit_bool_to_str", rpl_jit_bool_to_str as *const u8);
    builder.symbol("rpl_jit_trit_to_str", rpl_jit_trit_to_str as *const u8);
    builder.symbol("rpl_jit_str_len", rpl_jit_str_len as *const u8);
}
