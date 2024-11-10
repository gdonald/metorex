// Example runner tests for Metorex
// These tests execute example files using the Metorex CLI and verify their output

use std::process::Command;

/// Test greeting_line example with execution
#[test]
fn test_basics_greeting_line_execution() {
    let output = Command::new("./target/debug/metorex")
        .arg("examples/basics/greeting_line.mx")
        .output()
        .expect("Failed to execute metorex");

    assert!(
        output.status.success(),
        "metorex exited with error: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        "Hello, Ada!",
        "Expected output 'Hello, Ada!' but got '{}'",
        stdout.trim()
    );
}
