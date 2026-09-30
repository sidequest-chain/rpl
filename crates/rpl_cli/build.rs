//! Build script for rpl_cli to inject compile-time target information.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=RPL_TARGET={target}");
}
