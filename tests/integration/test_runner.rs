// Test runner for Metorex example files
//
// This test harness executes .mx example files and validates their output.
// Currently, it validates file structure until the interpreter is implemented.

use std::fs;
use std::path::Path;

/// Represents the expected outcome of running a Metorex file
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ExpectedOutcome {
    /// File should parse and run successfully
    Success {
        /// Expected output (when interpreter is ready)
        output: Option<String>,
    },
    /// File should produce a syntax error
    SyntaxError {
        /// Expected error message substring
        message: String,
    },
    /// File should produce a runtime error
    RuntimeError {
        /// Expected error message substring
        message: String,
    },
    /// File should produce a type error
    TypeError {
        /// Expected error message substring
        message: String,
    },
}

/// Test case for a Metorex example file
pub struct TestCase {
    /// Path to the .mx file relative to project root
    pub file_path: &'static str,
    /// Expected outcome when running this file
    #[allow(dead_code)]
    pub expected: ExpectedOutcome,
    /// Description of what this test validates
    pub description: &'static str,
}

impl TestCase {
    /// Run this test case
    ///
    /// Currently validates file existence and structure.
    /// Will execute the file and validate output once interpreter is ready.
    pub fn run(&self) -> Result<(), String> {
        // Validate file exists
        let path = Path::new(self.file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", self.file_path));
        }

        // Read file contents
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", self.file_path, e))?;

        // Validate file has content
        if contents.trim().is_empty() {
            return Err(format!("File is empty: {}", self.file_path));
        }

        // TODO: Once lexer/parser/interpreter are implemented, replace this with:
        // 1. Parse the file
        // 2. Execute it
        // 3. Validate the output matches expected outcome
        //
        // For now, we just validate the file structure is reasonable
        self.validate_file_structure(&contents)?;

        Ok(())
    }

    /// Validate basic file structure (placeholder until interpreter is ready)
    fn validate_file_structure(&self, contents: &str) -> Result<(), String> {
        // Check for balanced 'def'/'end' pairs (basic syntax check)
        let def_count = contents.matches("def ").count() + contents.matches("def\n").count();
        let class_count = contents.matches("class ").count() + contents.matches("class\n").count();
        let _end_count = contents.matches("\nend").count() + contents.matches(" end").count();

        // Note: This is a very rough heuristic and will be replaced by actual parsing
        // For now, we just want to ensure the file has reasonable structure
        if def_count > 0 || class_count > 0 {
            // We expect some 'end' keywords, but not necessarily matching yet
            // since the example file intentionally has incomplete code
            if contents.contains("def ")
                && !contents.contains("end")
                && !contents.contains("# Missing 'end'")
            {
                return Err(format!(
                    "File contains 'def' but no 'end' and no comment about missing end"
                ));
            }
        }

        Ok(())
    }
}

/// Run a test case and panic with detailed message on failure
macro_rules! run_test_case {
    ($test:expr) => {
        match $test.run() {
            Ok(()) => {} // Test passed
            Err(e) => panic!(
                "Test failed: {}\nDescription: {}\nError: {}",
                $test.file_path, $test.description, e
            ),
        }
    };
}

#[test]
#[ignore] // Temporarily disabled until more of the language is up and running
fn test_errors_basic_error() {
    let test = TestCase {
        file_path: "examples/errors/basic_error.mx",
        expected: ExpectedOutcome::Success {
            output: None, // Will add expected output once interpreter runs
        },
        description: "Validates error handling examples including syntax errors, \
                     runtime errors, and type errors with stack traces",
    };

    run_test_case!(test);
}

#[test]
#[ignore] // Temporarily disabled until more of the language is up and running
fn test_example_file_exists() {
    // Ensure our example file exists and is readable
    let path = Path::new("examples/errors/basic_error.mx");
    assert!(
        path.exists(),
        "Example file should exist: examples/errors/basic_error.mx"
    );

    let contents = fs::read_to_string(path).expect("Should be able to read example file");

    assert!(!contents.is_empty(), "Example file should not be empty");

    // Verify it contains error-related content
    assert!(
        contents.contains("Error") || contents.contains("error"),
        "Example file should contain error-related content"
    );
}

#[test]
#[ignore] // Temporarily disabled until more of the language is up and running
fn test_example_file_structure() {
    let path = Path::new("examples/errors/basic_error.mx");
    let contents = fs::read_to_string(path).expect("Should be able to read example file");

    // Verify it demonstrates various error types
    assert!(
        contents.contains("SyntaxError") || contents.contains("Syntax Error"),
        "Should demonstrate syntax errors"
    );

    assert!(
        contents.contains("RuntimeError") || contents.contains("Runtime Error"),
        "Should demonstrate runtime errors"
    );

    assert!(
        contents.contains("TypeError") || contents.contains("Type Error"),
        "Should demonstrate type errors"
    );

    // Verify it mentions stack traces
    assert!(
        contents.contains("stack trace") || contents.contains("Stack trace"),
        "Should mention stack traces"
    );
}

// Runtime instances example test removed - these are placeholders for future work

// Integration test infrastructure for future use
// This will be expanded once we have the interpreter

/// Helper to run a Metorex file and capture output (placeholder)
#[allow(dead_code)]
fn run_metorex_file(_path: &str) -> Result<String, String> {
    // TODO: Implement once interpreter is ready
    // 1. Read file
    // 2. Lex tokens
    // 3. Parse AST
    // 4. Execute/interpret
    // 5. Capture output
    Err("Interpreter not yet implemented".to_string())
}

/// Helper to run a Metorex file and expect an error (placeholder)
#[allow(dead_code)]
fn run_metorex_file_expect_error(_path: &str) -> Result<String, String> {
    // TODO: Implement once interpreter is ready
    // Similar to run_metorex_file but expects an error result
    Err("Interpreter not yet implemented".to_string())
}

#[cfg(test)]
mod future_tests {
    // These tests are commented out and will be enabled once the interpreter is ready

    // #[test]
    // fn test_syntax_error_detection() {
    //     let result = run_metorex_file_expect_error("examples/errors/basic_error.mx");
    //     assert!(result.is_ok());
    //     assert!(result.unwrap().contains("SyntaxError"));
    // }

    // #[test]
    // fn test_runtime_error_with_stack_trace() {
    //     let result = run_metorex_file_expect_error("examples/errors/runtime_error.mx");
    //     assert!(result.is_ok());
    //     let error = result.unwrap();
    //     assert!(error.contains("RuntimeError"));
    //     assert!(error.contains("at level3"));
    //     assert!(error.contains("at level2"));
    //     assert!(error.contains("at level1"));
    // }
}
