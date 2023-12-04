// Tests for Metorex error handling foundation

use metorex_core::error::{MetorexError, SourceLocation, StackFrame, reporting};

#[test]
fn test_source_location_display() {
    let loc = SourceLocation::new(10, 5, 120);
    assert_eq!(loc.to_string(), "10:5");

    let loc_with_file = SourceLocation::with_filename(10, 5, 120, "test.mx".to_string());
    assert_eq!(loc_with_file.to_string(), "test.mx:10:5");
}

#[test]
fn test_stack_frame_display() {
    let loc = SourceLocation::new(10, 5, 120);
    let frame = StackFrame::new("main".to_string(), loc);
    assert_eq!(frame.to_string(), "  at main (10:5)");
}

#[test]
fn test_syntax_error_creation() {
    let loc = SourceLocation::new(5, 10, 50);
    let err = MetorexError::syntax_error("Unexpected token", loc.clone());

    match err {
        MetorexError::SyntaxError { message, location } => {
            assert_eq!(message, "Unexpected token");
            assert_eq!(location, loc);
        }
        _ => panic!("Expected SyntaxError"),
    }
}

#[test]
fn test_runtime_error_creation() {
    let loc = SourceLocation::new(15, 3, 200);
    let err = MetorexError::runtime_error("Division by zero", loc.clone());

    match err {
        MetorexError::RuntimeError {
            message,
            location,
            stack_trace,
        } => {
            assert_eq!(message, "Division by zero");
            assert_eq!(location, loc);
            assert!(stack_trace.is_empty());
        }
        _ => panic!("Expected RuntimeError"),
    }
}

#[test]
fn test_runtime_error_with_stack_trace() {
    let loc = SourceLocation::new(20, 8, 300);
    let frame1 = StackFrame::new("foo".to_string(), SourceLocation::new(10, 5, 100));
    let frame2 = StackFrame::new("bar".to_string(), SourceLocation::new(5, 2, 50));

    let err = MetorexError::runtime_error_with_trace(
        "Null pointer",
        loc.clone(),
        vec![frame1.clone(), frame2.clone()],
    );

    match err {
        MetorexError::RuntimeError {
            message,
            location,
            stack_trace,
        } => {
            assert_eq!(message, "Null pointer");
            assert_eq!(location, loc);
            assert_eq!(stack_trace.len(), 2);
            assert_eq!(stack_trace[0], frame1);
            assert_eq!(stack_trace[1], frame2);
        }
        _ => panic!("Expected RuntimeError"),
    }
}

#[test]
fn test_type_error_creation() {
    let loc = SourceLocation::new(8, 12, 150);
    let err = MetorexError::type_error("Type mismatch", loc.clone());

    match err {
        MetorexError::TypeError {
            message,
            location,
            expected,
            found,
        } => {
            assert_eq!(message, "Type mismatch");
            assert_eq!(location, loc);
            assert!(expected.is_none());
            assert!(found.is_none());
        }
        _ => panic!("Expected TypeError"),
    }
}

#[test]
fn test_type_error_with_types() {
    let loc = SourceLocation::new(8, 12, 150);
    let err = MetorexError::type_error_with_types(
        "Cannot add String and Int",
        loc.clone(),
        "Int",
        "String",
    );

    match err {
        MetorexError::TypeError {
            message,
            location,
            expected,
            found,
        } => {
            assert_eq!(message, "Cannot add String and Int");
            assert_eq!(location, loc);
            assert_eq!(expected, Some("Int".to_string()));
            assert_eq!(found, Some("String".to_string()));
        }
        _ => panic!("Expected TypeError"),
    }
}

#[test]
fn test_with_stack_frame() {
    let loc = SourceLocation::new(20, 8, 300);
    let err = MetorexError::runtime_error("Error", loc);

    let frame = StackFrame::new("test".to_string(), SourceLocation::new(10, 5, 100));
    let err_with_frame = err.with_stack_frame(frame.clone());

    match err_with_frame {
        MetorexError::RuntimeError { stack_trace, .. } => {
            assert_eq!(stack_trace.len(), 1);
            assert_eq!(stack_trace[0], frame);
        }
        _ => panic!("Expected RuntimeError"),
    }
}

#[test]
fn test_error_location() {
    let loc = SourceLocation::new(5, 10, 50);
    let err = MetorexError::syntax_error("Test", loc.clone());
    assert_eq!(err.location(), Some(&loc));

    let internal_err = MetorexError::internal_error("Test");
    assert_eq!(internal_err.location(), None);
}

#[test]
fn test_format_error_with_source() {
    let source = "line 1\nline 2\nline 3 with error\nline 4";
    let loc = SourceLocation::new(3, 8, 20);
    let err = MetorexError::syntax_error("Unexpected token", loc);

    let formatted = reporting::format_error_with_source(&err, source);
    assert!(formatted.contains("Syntax error"));
    assert!(formatted.contains("line 3 with error"));
    assert!(formatted.contains("^"));
}

#[test]
fn test_format_error_with_stack_trace() {
    let source = "line 1\nline 2\nline 3";
    let loc = SourceLocation::new(2, 5, 10);
    let frame = StackFrame::new("main".to_string(), SourceLocation::new(1, 1, 0));
    let err = MetorexError::runtime_error_with_trace("Error", loc, vec![frame]);

    let formatted = reporting::format_error_with_source(&err, source);
    assert!(formatted.contains("Runtime error"));
    assert!(formatted.contains("Stack trace"));
    assert!(formatted.contains("at main"));
}

#[test]
fn test_format_error_compact() {
    let loc = SourceLocation::new(10, 5, 100);
    let err = MetorexError::syntax_error("Test error", loc);
    let formatted = reporting::format_error_compact(&err);
    assert!(formatted.contains("Syntax error"));
    assert!(formatted.contains("Test error"));
}
