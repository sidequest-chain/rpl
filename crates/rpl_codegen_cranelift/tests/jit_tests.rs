//! Integration and execution tests for the Cranelift JIT backend.

use rpl_codegen_cranelift::run_program;
use rpl_parser::parse_program;

#[test]
fn test_jit_arithmetic() {
    let source = "2 + 3 * 4";
    let program = parse_program(source).expect("Failed to parse arithmetic expression");
    let result = run_program(&program).expect("Failed to execute JIT");
    assert_eq!(result, 14);
}

#[test]
fn test_jit_trit_logic() {
    // Kleene AND: true and unknown => unknown (0)
    let src1 = "true and unknown";
    let prog1 = parse_program(src1).expect("Parse error");
    let res1 = run_program(&prog1).expect("JIT error");
    assert_eq!(res1, 0, "true and unknown must equal unknown (0)");

    // Kleene AND: false and unknown => false (-1)
    // -1 as i8 sign-extended or cast to i64 is -1
    let src2 = "false and unknown";
    let prog2 = parse_program(src2).expect("Parse error");
    let res2 = run_program(&prog2).expect("JIT error");
    assert_eq!(res2 as i8, -1, "false and unknown must equal false (-1)");

    // Kleene NOT: not unknown => unknown (0)
    let src3 = "not unknown";
    let prog3 = parse_program(src3).expect("Parse error");
    let res3 = run_program(&prog3).expect("JIT error");
    assert_eq!(res3, 0, "not unknown must equal unknown (0)");

    // Kleene NOT: not false => true (1)
    let src4 = "not false";
    let prog4 = parse_program(src4).expect("Parse error");
    let res4 = run_program(&prog4).expect("JIT error");
    assert_eq!(res4, 1, "not false must equal true (1)");

    // Kleene OR: false or unknown => unknown (0)
    let src5 = "false or unknown";
    let prog5 = parse_program(src5).expect("Parse error");
    let res5 = run_program(&prog5).expect("JIT error");
    assert_eq!(res5, 0, "false or unknown must equal unknown (0)");

    // Kleene OR: true or unknown => true (1)
    let src6 = "true or unknown";
    let prog6 = parse_program(src6).expect("Parse error");
    let res6 = run_program(&prog6).expect("JIT error");
    assert_eq!(res6, 1, "true or unknown must equal true (1)");
}

#[test]
fn test_jit_struct_init_and_field_access() {
    let source = r#"
type Point:
    x: Int
    y: Int
end Point

fn get_y(p: Point) -> Int:
    return p.y
end fn

let pt = Point:
    x: 10
    y: 25
end Point

get_y(pt)
"#;
    let program = parse_program(source).expect("Failed to parse struct source");
    let result = run_program(&program).expect("Failed to execute JIT");
    assert_eq!(result, 25);
}

#[test]
fn test_jit_range_for_loop() {
    let source = r#"
let mut sum = 0
for i in 1..4:
    sum = sum + i
end for
sum
"#;
    let program = parse_program(source).expect("Failed to parse loop source");
    let result = run_program(&program).expect("Failed to execute JIT");
    assert_eq!(result, 10); // 1 + 2 + 3 + 4 = 10
}

#[test]
fn test_jit_run_reaktor_example() {
    let file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("reaktor.rpl");

    let source = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Could not read {}: {e}", file_path.display()));

    let program = parse_program(&source).expect("Failed to parse reaktor.rpl");
    let exit_code = run_program(&program).expect("Failed to execute reaktor.rpl in Cranelift JIT");
    assert_eq!(exit_code, 0);
}
