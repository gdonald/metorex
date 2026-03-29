// Test Discovery Module
//
// Discovers and runs Metorex test files matching naming conventions:
// - *_test.rb
// - test_*.rb
// - *_spec.rb
//
// Walks directories recursively, executes each test file through the VM,
// and aggregates results with colored output.

use crate::error::MetorexError;
use crate::file_loader::parse_file;
use crate::vm::VirtualMachine;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Result of running a single test file.
#[derive(Debug)]
pub struct TestFileResult {
    pub path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u128,
}

/// Aggregated results from running all discovered test files.
#[derive(Debug)]
pub struct TestRunResult {
    pub results: Vec<TestFileResult>,
    pub total_duration_ms: u128,
}

impl TestRunResult {
    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }

    pub fn failed_count(&self) -> usize {
        self.results.iter().filter(|r| !r.success).count()
    }

    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.success)
    }
}

/// Check if a filename matches test file naming conventions.
pub fn is_test_file(filename: &str) -> bool {
    let name = filename.to_lowercase();
    if !name.ends_with(".rb") {
        return false;
    }
    let stem = &name[..name.len() - 3];
    stem.ends_with("_test") || stem.starts_with("test_") || stem.ends_with("_spec")
}

/// Recursively discover test files in a directory.
pub fn discover_test_files(dir: &Path) -> Result<Vec<PathBuf>, MetorexError> {
    let mut test_files = Vec::new();
    discover_recursive(dir, &mut test_files)?;
    test_files.sort();
    Ok(test_files)
}

fn discover_recursive(dir: &Path, results: &mut Vec<PathBuf>) -> Result<(), MetorexError> {
    if !dir.is_dir() {
        return Err(MetorexError::runtime_error(
            format!("Not a directory: '{}'", dir.display()),
            crate::error::SourceLocation::new(0, 0, 0),
        ));
    }

    let entries = fs::read_dir(dir).map_err(|e| {
        MetorexError::runtime_error(
            format!("Cannot read directory '{}': {}", dir.display(), e),
            crate::error::SourceLocation::new(0, 0, 0),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            MetorexError::runtime_error(
                format!("Error reading directory entry: {}", e),
                crate::error::SourceLocation::new(0, 0, 0),
            )
        })?;
        let path = entry.path();

        if path.is_dir() {
            discover_recursive(&path, results)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && is_test_file(name)
        {
            results.push(path);
        }
    }

    Ok(())
}

/// Run a single test file through the VM.
fn run_test_file(path: &Path) -> TestFileResult {
    let start = Instant::now();

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return TestFileResult {
                path: path.to_path_buf(),
                success: false,
                error: Some(format!("Cannot read file: {}", e)),
                duration_ms: start.elapsed().as_millis(),
            };
        }
    };

    let filename = path.display().to_string();
    let program = match parse_file(&source, &filename) {
        Ok(p) => p,
        Err(e) => {
            return TestFileResult {
                path: path.to_path_buf(),
                success: false,
                error: Some(format!("Parse error: {}", e)),
                duration_ms: start.elapsed().as_millis(),
            };
        }
    };

    let mut vm = VirtualMachine::new();
    let absolute_path = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    vm.set_current_file(absolute_path.clone());
    vm.mark_file_loaded(absolute_path);

    match vm.execute_program(&program) {
        Ok(_) => TestFileResult {
            path: path.to_path_buf(),
            success: true,
            error: None,
            duration_ms: start.elapsed().as_millis(),
        },
        Err(e) => TestFileResult {
            path: path.to_path_buf(),
            success: false,
            error: Some(format!("{}", e)),
            duration_ms: start.elapsed().as_millis(),
        },
    }
}

/// Discover and run all test files in the given directory.
/// Prints results as each file completes, then prints a summary.
pub fn run_test_discovery(dir: &Path) -> Result<TestRunResult, MetorexError> {
    let test_files = discover_test_files(dir)?;

    if test_files.is_empty() {
        println!("No test files found in '{}'", dir.display());
        println!("Test files must match: *_test.rb, test_*.rb, or *_spec.rb");
        return Ok(TestRunResult {
            results: vec![],
            total_duration_ms: 0,
        });
    }

    println!(
        "Discovered {} test file(s) in '{}'\n",
        test_files.len(),
        dir.display()
    );

    let total_start = Instant::now();
    let mut results = Vec::new();

    for file in &test_files {
        let display_path = file.strip_prefix(dir).unwrap_or(file);
        print!("  {} ... ", display_path.display());

        let result = run_test_file(file);

        if result.success {
            println!("\x1b[32mok\x1b[0m ({}ms)", result.duration_ms);
        } else {
            println!("\x1b[31mFAIL\x1b[0m ({}ms)", result.duration_ms);
            if let Some(ref err) = result.error {
                println!("    {}", err);
            }
        }

        results.push(result);
    }

    let total_duration_ms = total_start.elapsed().as_millis();
    let run_result = TestRunResult {
        results,
        total_duration_ms,
    };

    println!();
    print_summary(&run_result);

    Ok(run_result)
}

fn print_summary(result: &TestRunResult) {
    let passed = result.passed_count();
    let failed = result.failed_count();
    let total = result.results.len();

    println!("========================================");
    if failed == 0 {
        println!(
            "\x1b[32m{} passed\x1b[0m, {} failed ({} total) in {}ms",
            passed, failed, total, result.total_duration_ms
        );
    } else {
        println!(
            "{} passed, \x1b[31m{} failed\x1b[0m ({} total) in {}ms",
            passed, failed, total, result.total_duration_ms
        );
        println!("\nFailed files:");
        for r in &result.results {
            if !r.success {
                println!("  \x1b[31m{}\x1b[0m", r.path.display());
                if let Some(ref err) = r.error {
                    println!("    {}", err);
                }
            }
        }
    }
}
