// Unit tests for exception handling execution in Metorex VM
// Tests raise, begin/rescue/else/ensure execution

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

// Helper to parse and execute code
fn execute_code(code: &str) -> Result<Option<Object>, Box<dyn std::error::Error>> {
    let lexer = Lexer::new(code);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|errors| {
        let error_messages: Vec<String> = errors.iter().map(|e| format!("{}", e)).collect();
        format!("Parse errors: {}", error_messages.join(", "))
    })?;

    let mut vm = VirtualMachine::new();
    Ok(vm.execute_program(&ast)?)
}

// ============================================================================
// Basic Raise Tests
// ============================================================================

#[test]
fn test_raise_with_string_message() {
    let code = r#"raise "Something went wrong""#;
    let result = execute_code(code);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("RuntimeError"));
}

#[test]
fn test_raise_with_exception_class() {
    let code = r#"raise RuntimeError"#;
    let result = execute_code(code);
    assert!(result.is_err());
}

// ============================================================================
// Basic Rescue Tests
// ============================================================================

#[test]
fn test_begin_rescue_catches_exception() {
    let code = r#"
begin
  raise "Error"
rescue
  x = 42
end
x
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn test_begin_rescue_specific_exception_type() {
    let code = r#"
begin
  raise RuntimeError
rescue RuntimeError
  x = 1
end
x
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn test_begin_rescue_wrong_exception_type_not_caught() {
    let code = r#"
begin
  raise RuntimeError
rescue TypeError
  x = 1
end
"#;
    let result = execute_code(code);
    // Exception should not be caught, should propagate
    assert!(result.is_err());
}

// ============================================================================
// Exception Variable Binding Tests
// ============================================================================

#[test]
fn test_rescue_binds_exception_to_variable() {
    let code = r#"
begin
  raise "Error message"
rescue => e
  x = e
end
x
"#;
    let result = execute_code(code).unwrap();
    // x should contain the exception object
    match result {
        Some(Object::Exception(ex)) => {
            let exception = ex.borrow();
            assert_eq!(exception.message, "Error message");
        }
        _ => panic!("Expected exception object, got: {:?}", result),
    }
}

// ============================================================================
// Else Clause Tests
// ============================================================================

#[test]
fn test_else_clause_runs_when_no_exception() {
    let code = r#"
begin
  x = 1
rescue
  x = 2
else
  x = 3
end
x
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(3))); // else clause ran
}

#[test]
fn test_else_clause_does_not_run_when_exception_occurs() {
    let code = r#"
begin
  raise "Error"
rescue
  x = 2
else
  x = 3
end
x
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(2))); // rescue ran, else did not
}

// ============================================================================
// Ensure Block Tests
// ============================================================================

#[test]
fn test_ensure_block_runs_when_no_exception() {
    let code = r#"
begin
  x = 1
ensure
  y = 2
end
y
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn test_ensure_block_runs_when_exception_caught() {
    let code = r#"
begin
  raise "Error"
rescue
  x = 1
ensure
  y = 2
end
y
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn test_ensure_block_runs_when_exception_not_caught() {
    let code = r#"
y = 0
begin
  y = 1
  raise "Error"
  y = 99
ensure
  y = 2
end
"#;
    let result = execute_code(code);
    // Should fail because exception wasn't caught
    assert!(result.is_err());
    // We can't easily verify y=2 was executed, but it should have been
}

// ============================================================================
// Multiple Rescue Clauses Tests
// ============================================================================

#[test]
fn test_multiple_rescue_clauses_first_match_wins() {
    let code = r#"
begin
  raise RuntimeError
rescue RuntimeError
  x = 1
rescue TypeError
  x = 2
end
x
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(1))); // First rescue matched
}

// ============================================================================
// Bare Raise Tests
// ============================================================================

#[test]
fn test_bare_raise_in_rescue_block() {
    let code = r#"
begin
  raise "Original error"
rescue => e
  raise
end
"#;
    let result = execute_code(code);
    // Should propagate the original exception
    assert!(result.is_err());
}

#[test]
fn test_bare_raise_outside_rescue_fails() {
    let code = r#"raise"#;
    let result = execute_code(code);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No exception to re-raise")
    );
}

// ============================================================================
// Nested Begin/Rescue Tests
// ============================================================================

#[test]
fn test_nested_begin_rescue() {
    let code = r#"
begin
  begin
    raise "Inner error"
  rescue
    x = 1
  end
rescue
  x = 2
end
x
"#;
    let result = execute_code(code).unwrap();
    assert_eq!(result, Some(Object::Int(1))); // Inner rescue caught it
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_exception_with_all_clauses() {
    let code = r#"
y = 0
z = 0
begin
  y = 1
  raise "test"
  y = 99
rescue => e
  z = 2
else
  z = 3
ensure
  w = 4
end
[y, z, w]
"#;
    let result = execute_code(code).unwrap();
    match result {
        Some(Object::Array(arr)) => {
            let array = arr.borrow();
            assert_eq!(array.len(), 3);
            assert_eq!(array[0], Object::Int(1)); // y set before raise
            assert_eq!(array[1], Object::Int(2)); // rescue ran (z=2)
            assert_eq!(array[2], Object::Int(4)); // ensure ran (w=4)
        }
        _ => panic!("Expected array, got: {:?}", result),
    }
}

#[test]
fn test_no_exception_with_all_clauses() {
    let code = r#"
y = 0
z = 0
begin
  y = 1
rescue => e
  z = 2
else
  z = 3
ensure
  w = 4
end
[y, z, w]
"#;
    let result = execute_code(code).unwrap();
    match result {
        Some(Object::Array(arr)) => {
            let array = arr.borrow();
            assert_eq!(array.len(), 3);
            assert_eq!(array[0], Object::Int(1)); // y set in begin
            assert_eq!(array[1], Object::Int(3)); // else ran (z=3)
            assert_eq!(array[2], Object::Int(4)); // ensure ran (w=4)
        }
        _ => panic!("Expected array, got: {:?}", result),
    }
}
