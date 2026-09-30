//! CLI interface for Running Pseudo Language (RPL).
//! Provides `rpl check`, `rpl build`, and `rpl run` commands.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};
use rpl_codegen_c::generate_c;
use rpl_parser::parse_program;
use rpl_typechecker::check_program;

const RPL_RELEASE: &str = "0.2+2 \"Tohtlane\"";
const VERSION_STRING: &str = concat!(
    "0.2+2 \"Tohtlane\"\nTarget: ",
    env!("RPL_TARGET"),
    " (backends: cranelift-jit, c99-zig)"
);

#[derive(Parser)]
#[command(
    name = "rpl",
    author = "RPL Team",
    version = VERSION_STRING,
    about = "Running Pseudo Language (RPL) compiler & driver",
    long_about = "Running Pseudo Language (RPL) is a compiled, zero-garbage-collector systems programming language."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Type-check and verify RPL source code without code generation
    Check {
        /// Source file to check (.rpl)
        file: PathBuf,
    },

    /// Compile RPL source code to C99 or a native machine executable
    Build {
        /// Source file to build (.rpl)
        file: PathBuf,

        /// Custom output binary or source path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Emit C99 source code instead of compiling to a native binary
        #[arg(long)]
        emit_c: bool,
    },

    /// Compile and run RPL program immediately (default: in-memory Cranelift JIT)
    Run {
        /// Source file to execute (.rpl)
        file: PathBuf,

        /// Execute via C99 compilation instead of in-memory Cranelift JIT
        #[arg(long)]
        via_c: bool,
    },

    /// Start the RPL Language Server Protocol (LSP) daemon
    Lsp,
}

#[derive(Debug, Clone)]
enum HostCCompiler {
    ZigCc,
    Clang,
    Gcc,
    Cc,
    MsvcCl,
}

impl HostCCompiler {
    fn detect() -> Option<Self> {
        if Command::new("clang").arg("--version").output().is_ok() {
            Some(Self::Clang)
        } else if Command::new("gcc").arg("--version").output().is_ok() {
            Some(Self::Gcc)
        } else if Command::new("zig").arg("version").output().is_ok() {
            Some(Self::ZigCc)
        } else if Command::new("cc").arg("--version").output().is_ok() {
            Some(Self::Cc)
        } else if Command::new("cl.exe").output().is_ok() {
            Some(Self::MsvcCl)
        } else {
            None
        }
    }

    fn compile(&self, c_path: &Path, exe_path: &Path) -> Result<(), String> {
        let (cmd, args) = match self {
            Self::ZigCc => (
                "zig",
                vec![
                    "cc".to_string(),
                    "-std=c99".to_string(),
                    "-O2".to_string(),
                    c_path.display().to_string(),
                    "-o".to_string(),
                    exe_path.display().to_string(),
                ],
            ),
            Self::Clang | Self::Gcc | Self::Cc => {
                let bin = match self {
                    Self::Clang => "clang",
                    Self::Gcc => "gcc",
                    _ => "cc",
                };
                (
                    bin,
                    vec![
                        "-std=c99".to_string(),
                        "-O2".to_string(),
                        c_path.display().to_string(),
                        "-o".to_string(),
                        exe_path.display().to_string(),
                    ],
                )
            }
            Self::MsvcCl => (
                "cl.exe",
                vec![
                    "/std:c11".to_string(),
                    "/O2".to_string(),
                    c_path.display().to_string(),
                    format!("/Fe:{}", exe_path.display()),
                ],
            ),
        };

        let output = Command::new(cmd)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to spawn C compiler '{cmd}': {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "C compilation failed using '{cmd}':\n{}{}",
                stderr, stdout
            ));
        }

        Ok(())
    }
}

fn check_rpl_source(file_path: &Path) -> Result<rpl_ast::Program, String> {
    let source = fs::read_to_string(file_path)
        .map_err(|e| format!("Cannot read source file '{}': {e}", file_path.display()))?;

    let program = parse_program(&source)
        .map_err(|e| format!("[- - -] Parser error in '{}':\n  {e}", file_path.display()))?;

    if let Err(errors) = check_program(&program) {
        let mut msg = format!("[+ - -] Type errors in '{}':\n", file_path.display());
        for err in errors {
            msg.push_str(&format!("  - {err}\n"));
        }
        return Err(msg);
    }

    Ok(program)
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lsp => {
            rpl_lsp::run_server().await;
            ExitCode::SUCCESS
        }

        Commands::Check { file } => {
            if !file.exists() {
                eprintln!("Error: Source file does not exist: {}", file.display());
                return ExitCode::FAILURE;
            }

            match check_rpl_source(&file) {
                Ok(_) => {
                    println!("[+ + ?] Check passed: {}", file.display());
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("{err}");
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Build {
            file,
            output,
            emit_c,
        } => {
            if !file.exists() {
                eprintln!("Error: Source file does not exist: {}", file.display());
                return ExitCode::FAILURE;
            }

            let build_start = std::time::Instant::now();
            let program = match check_rpl_source(&file) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("{err}");
                    return ExitCode::FAILURE;
                }
            };

            let c_code = match generate_c(&program) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Codegen error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            let stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            if emit_c {
                let target_c_path = output.unwrap_or_else(|| PathBuf::from(format!("{stem}.c")));
                if let Err(e) = fs::write(&target_c_path, &c_code) {
                    eprintln!("Failed to write C file '{}': {e}", target_c_path.display());
                    return ExitCode::FAILURE;
                }
                let elapsed = build_start.elapsed().as_secs_f64();
                println!(
                    "[+ + +] rpl {RPL_RELEASE}: built '{}' ({:.3}s)",
                    target_c_path.display(),
                    elapsed
                );
                return ExitCode::SUCCESS;
            }

            let host_compiler = match HostCCompiler::detect() {
                Some(c) => c,
                None => {
                    eprintln!("Error: No supported C compiler (clang, gcc, zig cc, cc, cl.exe) found in PATH.");
                    eprintln!("Hint: You can use `rpl build --emit-c <file>` to emit C99 code directly.");
                    return ExitCode::FAILURE;
                }
            };

            let target_exe_path = output.unwrap_or_else(|| {
                let ext = if cfg!(windows) { ".exe" } else { "" };
                PathBuf::from(format!("{stem}{ext}"))
            });

            let temp_c_path = std::env::temp_dir().join(format!("rpl_{stem}_{}.c", std::process::id()));
            if let Err(e) = fs::write(&temp_c_path, &c_code) {
                eprintln!("Failed to write temporary C file: {e}");
                return ExitCode::FAILURE;
            }

            let compile_res = host_compiler.compile(&temp_c_path, &target_exe_path);
            let _ = fs::remove_file(&temp_c_path);

            match compile_res {
                Ok(_) => {
                    let elapsed = build_start.elapsed().as_secs_f64();
                    println!(
                        "[+ + +] rpl {RPL_RELEASE}: built '{}' ({:.3}s)",
                        target_exe_path.display(),
                        elapsed
                    );
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("{err}");
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Run { file, via_c } => {
            if !file.exists() {
                eprintln!("Error: Source file does not exist: {}", file.display());
                return ExitCode::FAILURE;
            }

            let program = match check_rpl_source(&file) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("{err}");
                    return ExitCode::FAILURE;
                }
            };

            if via_c {
                let c_code = match generate_c(&program) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Codegen error: {e}");
                        return ExitCode::FAILURE;
                    }
                };

                let host_compiler = match HostCCompiler::detect() {
                    Some(c) => c,
                    None => {
                        eprintln!("Error: No supported C compiler (clang, gcc, zig cc, cc, cl.exe) found in PATH to execute program.");
                        return ExitCode::FAILURE;
                    }
                };

                let stem = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("run");

                let ext = if cfg!(windows) { ".exe" } else { "" };
                let pid = std::process::id();
                let temp_c_path = std::env::temp_dir().join(format!("rpl_run_{stem}_{pid}.c"));
                let temp_exe_path = std::env::temp_dir().join(format!("rpl_run_{stem}_{pid}{ext}"));

                if let Err(e) = fs::write(&temp_c_path, &c_code) {
                    eprintln!("Failed to write temporary C file: {e}");
                    return ExitCode::FAILURE;
                }

                let compile_res = host_compiler.compile(&temp_c_path, &temp_exe_path);
                let _ = fs::remove_file(&temp_c_path);

                if let Err(err) = compile_res {
                    eprintln!("{err}");
                    return ExitCode::FAILURE;
                }

                let status = Command::new(&temp_exe_path).status();
                let _ = fs::remove_file(&temp_exe_path);

                match status {
                    Ok(s) => {
                        if s.success() {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::from(s.code().unwrap_or(1) as u8)
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to execute compiled binary: {e}");
                        ExitCode::FAILURE
                    }
                }
            } else {
                // In-memory JIT execution via Cranelift
                println!("[+ + +] Running in-memory (Cranelift JIT) ...");
                match rpl_codegen_cranelift::run_program(&program) {
                    Ok(code) => {
                        if code == 0 {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::from((code & 0xFF) as u8)
                        }
                    }
                    Err(err) => {
                        eprintln!("JIT execution error: {err}");
                        ExitCode::FAILURE
                    }
                }
            }
        }
    }
}
