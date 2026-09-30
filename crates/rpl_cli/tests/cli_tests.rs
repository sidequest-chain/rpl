use std::fs;
use std::process::Command;

fn get_rpl_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("Cannot get current_exe");
    path.pop(); // Remove test exe name
    if path.ends_with("deps") {
        path.pop(); // Remove 'deps'
    }
    path.push(if cfg!(windows) { "rpl.exe" } else { "rpl" });
    path
}

#[test]
fn test_cli_check_valid_and_invalid_files() {
    let rpl_bin = get_rpl_bin();
    let temp_dir = std::env::temp_dir();

    // 1. Valid file
    let valid_file = temp_dir.join("test_valid.rpl");
    fs::write(
        &valid_file,
        r#"
fn add(x: Int, y: Int) -> Int:
    return x + y
end

let val = add(10, 20)
println("Val: $val")
"#,
    )
    .unwrap();

    let output_valid = Command::new(&rpl_bin)
        .args(["check", valid_file.to_str().unwrap()])
        .output()
        .expect("Failed to run rpl check on valid file");

    assert!(output_valid.status.success());
    let stdout = String::from_utf8_lossy(&output_valid.stdout);
    assert!(stdout.contains("Check passed"));

    // 2. Invalid file (Type mismatch)
    let invalid_file = temp_dir.join("test_invalid.rpl");
    fs::write(
        &invalid_file,
        r#"
let x: Int = "not an int"
"#,
    )
    .unwrap();

    let output_invalid = Command::new(&rpl_bin)
        .args(["check", invalid_file.to_str().unwrap()])
        .output()
        .expect("Failed to run rpl check on invalid file");

    assert!(!output_invalid.status.success());
    let stderr = String::from_utf8_lossy(&output_invalid.stderr);
    assert!(stderr.contains("Type mismatch") || stderr.contains("Type errors"));

    let _ = fs::remove_file(valid_file);
    let _ = fs::remove_file(invalid_file);
}

#[test]
fn test_cli_build_emit_c() {
    let rpl_bin = get_rpl_bin();
    let temp_dir = std::env::temp_dir();
    let rpl_file = temp_dir.join("test_emit.rpl");
    let c_file = temp_dir.join("test_emit.c");

    fs::write(
        &rpl_file,
        r#"
let answer = 42
println("The answer is $answer")
"#,
    )
    .unwrap();

    let output = Command::new(&rpl_bin)
        .args([
            "build",
            rpl_file.to_str().unwrap(),
            "--emit-c",
            "-o",
            c_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run rpl build --emit-c");

    assert!(output.status.success());
    assert!(c_file.exists());

    let c_content = fs::read_to_string(&c_file).unwrap();
    assert!(c_content.contains("const int64_t answer = 42LL;"));
    assert!(c_content.contains("RPL_RUNTIME_H"));

    let _ = fs::remove_file(rpl_file);
    let _ = fs::remove_file(c_file);
}

#[test]
fn test_cli_run() {
    let rpl_bin = get_rpl_bin();
    let temp_dir = std::env::temp_dir();
    let rpl_file = temp_dir.join("test_run.rpl");

    fs::write(
        &rpl_file,
        r#"
let status: Trit = true and unknown
match status:
    case true:
        println("VERIFIED")
    case false:
        println("FAILED")
    case unknown:
        println("INDETERMINATE_OK")
end
"#,
    )
    .unwrap();

    let output = Command::new(&rpl_bin)
        .args(["run", rpl_file.to_str().unwrap()])
        .output()
        .expect("Failed to run rpl run");

    assert!(
        output.status.success(),
        "rpl run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("INDETERMINATE_OK"));

    let _ = fs::remove_file(rpl_file);
}

#[test]
fn test_cli_version() {
    let rpl_bin = get_rpl_bin();

    let output = Command::new(&rpl_bin)
        .arg("--version")
        .output()
        .expect("Failed to run rpl --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rpl 0.2+2 \"Tohtlane\""),
        "stdout was: {}",
        stdout
    );
    assert!(
        stdout.contains("Target:"),
        "stdout was: {}",
        stdout
    );
    assert!(
        stdout.contains("(backends: cranelift-jit, c99-zig)"),
        "stdout was: {}",
        stdout
    );
}

#[test]
fn test_cli_lsp_help() {
    let rpl_bin = get_rpl_bin();

    let output = Command::new(&rpl_bin)
        .args(["lsp", "--help"])
        .output()
        .expect("Failed to run rpl lsp --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Start the RPL Language Server Protocol (LSP) daemon"),
        "stdout was: {}",
        stdout
    );
}

