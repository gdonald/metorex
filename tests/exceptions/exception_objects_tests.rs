// Unit tests for Metorex Exception objects
// Tests exception creation, source location, cause chains, and exception hierarchy

use metorex::object::{Exception, Object, SourceLocation};
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Basic Exception Tests
// ============================================================================

#[test]
fn test_exception_new() {
    let exc = Exception::new(
        "RuntimeError".to_string(),
        "Something went wrong".to_string(),
    );

    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, "Something went wrong");
    assert!(exc.backtrace.is_none());
    assert!(exc.location.is_none());
    assert!(exc.cause.is_none());
}

#[test]
fn test_exception_with_backtrace() {
    let backtrace = vec![
        "at main (main.mx:10)".to_string(),
        "at foo (lib.mx:25)".to_string(),
    ];

    let exc = Exception::with_backtrace(
        "RuntimeError".to_string(),
        "Error in foo".to_string(),
        backtrace.clone(),
    );

    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, "Error in foo");
    assert_eq!(exc.backtrace, Some(backtrace));
}

#[test]
fn test_exception_with_location() {
    let location = SourceLocation::new("test.mx".to_string(), 42, 10);
    let exc = Exception::with_location(
        "TypeError".to_string(),
        "Type mismatch".to_string(),
        location.clone(),
    );

    assert_eq!(exc.exception_type, "TypeError");
    assert_eq!(exc.message, "Type mismatch");
    assert_eq!(exc.location, Some(location));
}

#[test]
fn test_exception_with_cause() {
    let cause = Object::exception("ValueError", "Invalid value");
    let exc = Exception::with_cause(
        "RuntimeError".to_string(),
        "Operation failed".to_string(),
        cause.clone(),
    );

    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, "Operation failed");
    assert!(exc.cause.is_some());
}

#[test]
fn test_exception_with_all() {
    let backtrace = vec!["at test (test.mx:1)".to_string()];
    let location = SourceLocation::new("test.mx".to_string(), 1, 1);
    let cause = Object::exception("ValueError", "Root cause");

    let exc = Exception::with_all(
        "RuntimeError".to_string(),
        "Complex error".to_string(),
        Some(backtrace.clone()),
        Some(location.clone()),
        Some(cause),
    );

    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, "Complex error");
    assert_eq!(exc.backtrace, Some(backtrace));
    assert_eq!(exc.location, Some(location));
    assert!(exc.cause.is_some());
}

// ============================================================================
// Source Location Tests
// ============================================================================

#[test]
fn test_source_location_new() {
    let loc = SourceLocation::new("main.mx".to_string(), 10, 5);

    assert_eq!(loc.file, "main.mx");
    assert_eq!(loc.line, 10);
    assert_eq!(loc.column, 5);
}

#[test]
fn test_source_location_equality() {
    let loc1 = SourceLocation::new("test.mx".to_string(), 1, 1);
    let loc2 = SourceLocation::new("test.mx".to_string(), 1, 1);
    let loc3 = SourceLocation::new("test.mx".to_string(), 2, 1);

    assert_eq!(loc1, loc2);
    assert_ne!(loc1, loc3);
}

// ============================================================================
// Exception Chain Tests
// ============================================================================

#[test]
fn test_exception_chain_single() {
    let exc = Exception::new("RuntimeError".to_string(), "Error message".to_string());
    let chain = exc.exception_chain();

    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0], "RuntimeError: Error message");
}

#[test]
fn test_exception_chain_two_levels() {
    let cause = Object::exception("ValueError", "Invalid input");
    let exc = Exception::with_cause(
        "RuntimeError".to_string(),
        "Processing failed".to_string(),
        cause,
    );

    let chain = exc.exception_chain();

    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0], "RuntimeError: Processing failed");
    assert_eq!(chain[1], "ValueError: Invalid input");
}

#[test]
fn test_exception_chain_multiple_levels() {
    let root_cause = Object::exception("IOError", "File not found");

    let mid_cause = Rc::new(RefCell::new(Exception::with_cause(
        "ValueError".to_string(),
        "Invalid file content".to_string(),
        root_cause,
    )));

    let top_exc = Exception::with_cause(
        "RuntimeError".to_string(),
        "Operation failed".to_string(),
        Object::Exception(mid_cause),
    );

    let chain = top_exc.exception_chain();

    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0], "RuntimeError: Operation failed");
    assert_eq!(chain[1], "ValueError: Invalid file content");
    assert_eq!(chain[2], "IOError: File not found");
}

#[test]
fn test_exception_chain_no_cause() {
    let exc = Exception::new("RuntimeError".to_string(), "Simple error".to_string());
    let chain = exc.exception_chain();

    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0], "RuntimeError: Simple error");
}

// ============================================================================
// Exception Object Wrapper Tests
// ============================================================================

#[test]
fn test_exception_object_creation() {
    let obj = Object::exception("RuntimeError", "Test error");

    match obj {
        Object::Exception(exc) => {
            let exception = exc.borrow();
            assert_eq!(exception.exception_type, "RuntimeError");
            assert_eq!(exception.message, "Test error");
        }
        _ => panic!("Expected Exception object"),
    }
}

#[test]
fn test_exception_object_display() {
    let obj = Object::exception("TypeError", "Type mismatch");
    let display = format!("{}", obj);

    assert_eq!(display, "TypeError: Type mismatch");
}

// ============================================================================
// Complex Exception Scenarios
// ============================================================================

#[test]
fn test_exception_with_full_context() {
    let backtrace = vec![
        "at function_c (file.mx:30)".to_string(),
        "at function_b (file.mx:20)".to_string(),
        "at function_a (file.mx:10)".to_string(),
    ];

    let location = SourceLocation::new("file.mx".to_string(), 30, 15);

    let root_cause = Object::exception("ValueError", "Invalid parameter");

    let exc = Exception::with_all(
        "RuntimeError".to_string(),
        "Function call failed".to_string(),
        Some(backtrace.clone()),
        Some(location.clone()),
        Some(root_cause),
    );

    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, "Function call failed");
    assert_eq!(exc.backtrace.as_ref().unwrap().len(), 3);
    assert_eq!(exc.location.as_ref().unwrap().line, 30);

    let chain = exc.exception_chain();
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_exception_different_types() {
    let runtime_err = Exception::new("RuntimeError".to_string(), "Runtime issue".to_string());
    let type_err = Exception::new("TypeError".to_string(), "Type issue".to_string());
    let value_err = Exception::new("ValueError".to_string(), "Value issue".to_string());

    assert_eq!(runtime_err.exception_type, "RuntimeError");
    assert_eq!(type_err.exception_type, "TypeError");
    assert_eq!(value_err.exception_type, "ValueError");
}

#[test]
fn test_exception_empty_message() {
    let exc = Exception::new("RuntimeError".to_string(), String::new());

    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, "");
}

#[test]
fn test_exception_long_message() {
    let long_message = "This is a very long error message that contains a lot of details about what went wrong in the system. ".repeat(10);
    let exc = Exception::new("RuntimeError".to_string(), long_message.clone());

    assert_eq!(exc.exception_type, "RuntimeError");
    assert_eq!(exc.message, long_message);
}

#[test]
fn test_source_location_different_files() {
    let loc1 = SourceLocation::new("file1.mx".to_string(), 10, 5);
    let loc2 = SourceLocation::new("file2.mx".to_string(), 10, 5);

    assert_ne!(loc1.file, loc2.file);
    assert_ne!(loc1, loc2);
}

#[test]
fn test_exception_clone() {
    let exc1 = Exception::new("RuntimeError".to_string(), "Test".to_string());
    let exc2 = exc1.clone();

    assert_eq!(exc1, exc2);
    assert_eq!(exc1.exception_type, exc2.exception_type);
    assert_eq!(exc1.message, exc2.message);
}
