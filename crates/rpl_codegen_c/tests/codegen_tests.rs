use std::fs;
use std::process::Command;
use rpl_codegen_c::generate_c;
use rpl_parser::parse_program;

#[test]
fn test_basic_arithmetic_and_variables_codegen() {
    let source = r#"
let a = 10
let mut b = 20
b = a + b * 2
println("Result: $b")
"#;

    let program = parse_program(source).expect("Failed to parse program");
    let c_code = generate_c(&program).expect("Failed to generate C");

    assert!(c_code.contains("const int64_t a = 10LL;"));
    assert!(c_code.contains("int64_t b = 20LL;"));
    assert!(c_code.contains("b = (a + (b * 2LL));"));
    assert!(c_code.contains("rpl_int_to_str(b)"));
    assert!(c_code.contains("int main(int argc, char** argv)"));
}

#[test]
fn test_trit_ternary_logic_and_match_codegen() {
    let source = r#"
let t1: Trit = true
let t2: Trit = unknown
let result: Trit = t1 and t2

match result:
    case true:
        println("Is true")
    case false:
        println("Is false")
    case unknown:
        println("Is unknown")
end
"#;

    let program = parse_program(source).expect("Failed to parse program");
    let c_code = generate_c(&program).expect("Failed to generate C");

    assert!(c_code.contains("const rpl_trit_t t1 = RPL_TRIT_TRUE;"));
    assert!(c_code.contains("const rpl_trit_t t2 = RPL_TRIT_UNKNOWN;"));
    assert!(c_code.contains("rpl_trit_and(t1, t2)"));
    assert!(c_code.contains("switch (result) {"));
    assert!(c_code.contains("case RPL_TRIT_TRUE:"));
    assert!(c_code.contains("case RPL_TRIT_FALSE:"));
    assert!(c_code.contains("case RPL_TRIT_UNKNOWN:"));
}

#[test]
fn test_functions_and_pipe_operator_codegen() {
    let source = r#"
fn double_val(x: Int) -> Int:
    return x * 2
end

fn add_one(x: Int) -> Int:
    return x + 1
end

let val = 5 |> double_val() |> add_one()
println("Final: $val")
"#;

    let program = parse_program(source).expect("Failed to parse program");
    let c_code = generate_c(&program).expect("Failed to generate C");

    assert!(c_code.contains("int64_t double_val(int64_t x);"));
    assert!(c_code.contains("int64_t add_one(int64_t x);"));
    assert!(c_code.contains("int64_t double_val(int64_t x) {"));
    assert!(c_code.contains("return (x * 2LL);"));
    assert!(c_code.contains("add_one(double_val(5LL))"));
}

#[test]
fn test_type_declaration_and_struct_instantiation() {
    let source = r#"
type SensorReport:
    device_id: String
    temperature_nominal: Trit
    voltage_stable: Trit
end

let report = SensorReport(
    device_id: "TURBINE-04",
    temperature_nominal: true,
    voltage_stable: unknown
)

let status: Trit = report.temperature_nominal and report.voltage_stable
println("Status evaluated")
"#;

    let program = parse_program(source).expect("Failed to parse program");
    let c_code = generate_c(&program).expect("Failed to generate C");

    assert!(c_code.contains("typedef struct SensorReport {"));
    assert!(c_code.contains("const char* device_id;"));
    assert!(c_code.contains("rpl_trit_t temperature_nominal;"));
    assert!(c_code.contains("rpl_trit_t voltage_stable;"));
    assert!(c_code.contains("(SensorReport){"));
    assert!(c_code.contains(".device_id = \"TURBINE-04\""));
    assert!(c_code.contains(".temperature_nominal = RPL_TRIT_TRUE"));
    assert!(c_code.contains(".voltage_stable = RPL_TRIT_UNKNOWN"));
    assert!(c_code.contains("rpl_trit_and(report.temperature_nominal, report.voltage_stable)"));
}

#[test]
fn test_for_loop_range_codegen() {
    let source = r#"
for i in 1..5:
    println("Count: $i")
end
"#;

    let program = parse_program(source).expect("Failed to parse program");
    let c_code = generate_c(&program).expect("Failed to generate C");

    assert!(c_code.contains("for (int64_t i = 1LL; i <= 5LL; ++i) {"));
}

#[test]
fn test_real_c_compilation_with_host_compiler() {
    let source = r#"
fn calculate(x: Int, y: Int) -> Int:
    let temp = x * 2
    return temp + y
end

let res = calculate(21, 0)
println("Magic number: $res")
"#;

    let program = parse_program(source).expect("Failed to parse program");
    let c_code = generate_c(&program).expect("Failed to generate C");

    let temp_dir = std::env::temp_dir();
    let c_path = temp_dir.join("test_rpl_generated.c");
    let exe_path = temp_dir.join(if cfg!(windows) { "test_rpl_generated.exe" } else { "test_rpl_generated" });

    fs::write(&c_path, &c_code).expect("Failed to write temporary C file");

    // Try finding a C compiler (zig cc, clang, gcc, cl, cc)
    let compiler_cmd = if Command::new("zig").arg("version").output().is_ok() {
        Some(("zig", vec!["cc", "-std=c99", c_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()]))
    } else if Command::new("clang").arg("--version").output().is_ok() {
        Some(("clang", vec!["-std=c99", c_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()]))
    } else if Command::new("gcc").arg("--version").output().is_ok() {
        Some(("gcc", vec!["-std=c99", c_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()]))
    } else if Command::new("cc").arg("--version").output().is_ok() {
        Some(("cc", vec!["-std=c99", c_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()]))
    } else {
        None
    };

    if let Some((prog, args)) = compiler_cmd {
        let compile_output = Command::new(prog)
            .args(&args)
            .output()
            .expect("Failed to execute C compiler");

        assert!(
            compile_output.status.success(),
            "C compilation failed: {}",
            String::from_utf8_lossy(&compile_output.stderr)
        );

        // Run compiled executable and check stdout
        let run_output = Command::new(&exe_path)
            .output()
            .expect("Failed to execute generated binary");

        let stdout = String::from_utf8_lossy(&run_output.stdout);
        assert!(stdout.contains("Magic number: 42"));

        let _ = fs::remove_file(c_path);
        let _ = fs::remove_file(exe_path);
    }
}

#[test]
fn test_struct_block_init_and_trit_interpolation_codegen() {
    let source = r#"
type Sensor:
    id: String
    active: Trit
end Sensor

let s = Sensor:
    id: "S-1"
    active: unknown
end Sensor

print "Sensor $s.id state: $s.active"
"#;
    let program = parse_program(source).expect("Failed to parse sensor test");
    let c_code = generate_c(&program).expect("Failed to generate C code");

    assert!(c_code.contains("(Sensor){ .id = \"S-1\", .active = RPL_TRIT_UNKNOWN }"));
    assert!(c_code.contains("rpl_trit_to_str(s.active)"));
}

#[test]
fn test_io_codegen_and_execution() {
    let source = r#"
let test_file = "temp_c_io_test.txt"
write_file(test_file, "Line A\nLine B\n")
let content = read_file(test_file)
println("Content: $content")
"#;
    let program = parse_program(source).expect("Failed to parse I/O source");
    let c_code = generate_c(&program).expect("Failed to generate C code");

    assert!(c_code.contains("rpl_write_file"));
    assert!(c_code.contains("rpl_read_file"));

    let temp_dir = std::env::temp_dir();
    let c_path = temp_dir.join("test_rpl_io_generated.c");
    let exe_path = temp_dir.join(if cfg!(windows) { "test_rpl_io_generated.exe" } else { "test_rpl_io_generated" });

    std::fs::write(&c_path, &c_code).expect("Failed to write temporary C file");

    let compiler_cmd = if std::process::Command::new("zig").arg("version").output().is_ok() {
        Some(("zig", vec!["cc", "-std=c99", c_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()]))
    } else if std::process::Command::new("clang").arg("--version").output().is_ok() {
        Some(("clang", vec!["-std=c99", c_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()]))
    } else if std::process::Command::new("gcc").arg("--version").output().is_ok() {
        Some(("gcc", vec!["-std=c99", c_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()]))
    } else {
        None
    };

    if let Some((prog, args)) = compiler_cmd {
        let compile_output = std::process::Command::new(prog)
            .args(&args)
            .output()
            .expect("Failed to compile generated C code");

        assert!(
            compile_output.status.success(),
            "C compilation of I/O test failed: {}",
            String::from_utf8_lossy(&compile_output.stderr)
        );

        let run_output = std::process::Command::new(&exe_path)
            .current_dir(&temp_dir)
            .output()
            .expect("Failed to run compiled C binary");

        let stdout = String::from_utf8_lossy(&run_output.stdout);
        assert!(stdout.replace("\r\n", "\n").contains("Content: Line A\nLine B\n"), "Actual stdout was: {stdout:?}, stderr: {:?}", String::from_utf8_lossy(&run_output.stderr));

        let _ = std::fs::remove_file(c_path);
        let _ = std::fs::remove_file(exe_path);
        let _ = std::fs::remove_file(temp_dir.join("temp_c_io_test.txt"));
    }
}


