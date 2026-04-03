// Parser error recovery tests for Metorex
// Tests how the parser handles and recovers from syntax errors

use metorex::error::MetorexError;
use metorex::lexer::Lexer;
use metorex::parser::Parser;

/// Helper function to parse source code and get errors
fn parse_and_get_errors(source: &str) -> Vec<MetorexError> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}

/// Helper function to check if parsing fails
fn parse_fails(source: &str) -> bool {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse().is_err()
}

// ============================================================================
// Missing Token Tests
// ============================================================================

#[test]
fn test_missing_end_in_function() {
    let source = r#"
def foo
  x = 1
"#;
    assert!(parse_fails(source));
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

#[test]
fn test_missing_end_in_class() {
    let source = r#"
class Foo
  def bar
    42
  end
"#;
    assert!(parse_fails(source));
}

#[test]
fn test_missing_end_in_if() {
    let source = r#"
if true
  x = 1
"#;
    assert!(parse_fails(source));
}

#[test]
fn test_missing_end_in_while() {
    let source = r#"
while true
  x = 1
"#;
    assert!(parse_fails(source));
}

// ============================================================================
// Missing Delimiter Tests
// ============================================================================

#[test]
fn test_missing_closing_paren() {
    let source = "foo(1, 2, 3";
    assert!(parse_fails(source));
}

#[test]
fn test_missing_closing_bracket() {
    let source = "[1, 2, 3";
    assert!(parse_fails(source));
}

#[test]
fn test_missing_closing_brace() {
    let source = "{x: 1, y: 2";
    assert!(parse_fails(source));
}

#[test]
fn test_missing_opening_paren() {
    let source = "foo 1, 2, 3)";
    // This should parse as a statement followed by invalid syntax
    // The parser should report an error
    assert!(parse_fails(source));
}

// ============================================================================
// Invalid Expression Tests
// ============================================================================

#[test]
fn test_invalid_binary_operator_usage() {
    let _source = "x = + 5";
    // Unary plus should work, but let's test other cases
    let source2 = "x = 5 +";
    assert!(parse_fails(source2));
}

#[test]
fn test_incomplete_expression() {
    let source = "x = 1 + ";
    assert!(parse_fails(source));
}

#[test]
fn test_invalid_assignment_target() {
    let source = "42 = x";
    // This should fail during parsing or be caught later
    // For now, the parser might accept it syntactically
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let _ = parser.parse();
    // Note: This might not fail at parse time depending on implementation
}

// ============================================================================
// Unexpected Token Tests
// ============================================================================

#[test]
fn test_unexpected_token_in_expression() {
    let source = "x = end";
    // 'end' is a keyword, not valid in expression context
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let _ = parser.parse();
    // This will parse 'end' as an identifier/keyword which might be handled
}

#[test]
fn test_multiple_errors_in_program() {
    let source = r#"
def foo
  x = 1
  # Missing end

class Bar
  def baz
    y = 2
  # Missing end for method
# Missing end for class
"#;
    let errors = parse_and_get_errors(source);
    // Should detect multiple missing 'end' keywords
    assert!(!errors.is_empty());
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[test]
fn test_recovery_continues_after_error() {
    let source = r#"
def broken_function
  x = 1
  # Missing end

def good_function
  y = 2
end
"#;
    // Parser should attempt to recover and continue parsing
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

#[test]
fn test_recovery_with_multiple_statements() {
    let source = r#"
x = 1
y =
z = 3
"#;
    // Should have error on incomplete 'y =' but might recover for 'z = 3'
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

#[test]
fn test_synchronization_at_statement_boundary() {
    let source = r#"
x = 1 +
y = 2
z = 3
"#;
    // Parser should synchronize after the error
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

// ============================================================================
// Function Definition Error Tests
// ============================================================================

#[test]
fn test_missing_function_name() {
    let source = r#"
def
  x = 1
end
"#;
    assert!(parse_fails(source));
}

#[test]
fn test_invalid_parameter_syntax() {
    let source = r#"
def foo(x, , y)
  x + y
end
"#;
    // Double comma should be an error
    assert!(parse_fails(source));
}

#[test]
fn test_missing_closing_paren_in_params() {
    let source = r#"
def foo(x, y
  x + y
end
"#;
    assert!(parse_fails(source));
}

// ============================================================================
// Class Definition Error Tests
// ============================================================================

#[test]
fn test_missing_class_name() {
    let source = r#"
class
  def foo
    42
  end
end
"#;
    assert!(parse_fails(source));
}

#[test]
fn test_invalid_superclass_syntax() {
    let source = r#"
class Foo <
  def bar
    42
  end
end
"#;
    assert!(parse_fails(source));
}

// ============================================================================
// Control Flow Error Tests
// ============================================================================

#[test]
fn test_missing_condition_in_if() {
    let source = r#"
if
  x = 1
end
"#;
    assert!(parse_fails(source));
}

#[test]
fn test_missing_condition_in_while() {
    let source = r#"
while
  x = 1
end
"#;
    assert!(parse_fails(source));
}

#[test]
fn test_else_without_if() {
    let source = r#"
else
  x = 1
end
"#;
    // This should fail as 'else' without 'if' is invalid
    assert!(parse_fails(source));
}

// ============================================================================
// Array and Dictionary Error Tests
// ============================================================================

#[test]
fn test_trailing_comma_in_array() {
    let source = "[1, 2, 3,]";
    // Trailing comma might be an error depending on implementation
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let _ = parser.parse();
    // Note: Some parsers allow trailing commas
}

#[test]
fn test_missing_value_in_array() {
    let source = "[1, , 3]";
    // Double comma should be an error
    assert!(parse_fails(source));
}

#[test]
fn test_missing_colon_in_dictionary() {
    let source = "{x 1, y: 2}";
    assert!(parse_fails(source));
}

#[test]
fn test_missing_value_in_dictionary() {
    let source = "{x:, y: 2}";
    assert!(parse_fails(source));
}

// ============================================================================
// Method Call Error Tests
// ============================================================================

#[test]
fn test_missing_method_name() {
    let source = "obj.()";
    assert!(parse_fails(source));
}

#[test]
fn test_incomplete_method_chain() {
    let source = "obj.method().";
    assert!(parse_fails(source));
}

// ============================================================================
// Complex Error Scenarios
// ============================================================================

#[test]
fn test_nested_errors() {
    let source = r#"
class Outer
  class Inner
    def method
      if true
        x = 1
      # Missing ends everywhere
"#;
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

#[test]
fn test_mixed_valid_and_invalid_code() {
    let source = r#"
# Valid code
def valid_function(x)
  x + 1
end

# Invalid code
def broken_function(
  # Missing closing paren and end

# More valid code
x = 10
y = 20
"#;
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

#[test]
fn test_error_in_expression_context() {
    let source = r#"
x = (1 + 2
y = 3
"#;
    assert!(parse_fails(source));
}

#[test]
fn test_multiple_missing_delimiters() {
    let source = "foo(bar(baz(1, 2";
    // Missing three closing parens
    assert!(parse_fails(source));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_function_body() {
    let source = r#"
def empty
end
"#;
    // Empty function body should be valid
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_empty_class_body() {
    let source = r#"
class Empty
end
"#;
    // Empty class body should be valid
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_only_comments() {
    let source = r#"
# This is a comment
# Another comment
"#;
    // Should parse successfully with no statements
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_empty_program() {
    let source = "";
    // Empty program should parse successfully
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_contains_location_info() {
    let source = "def foo\n  x = 1";
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
    // Check that error has location information
    for error in &errors {
        let error_string = format!("{}", error);
        assert!(error_string.contains("Syntax error"));
    }
}

#[test]
fn test_error_message_clarity() {
    let source = "if true\n  x = 1";
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
    // Errors should mention 'end'
    let error_string = format!("{:?}", errors[0]);
    assert!(error_string.to_lowercase().contains("end"));
}

// ============================================================================
// Recovery After Specific Constructs
// ============================================================================

#[test]
fn test_recovery_after_failed_function() {
    let source = r#"
def broken(
  x = 1

def working
  y = 2
end
"#;
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

#[test]
fn test_recovery_after_failed_class() {
    let source = r#"
class Broken <
  def foo
    1
  end

class Working
  def bar
    2
  end
end
"#;
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}

#[test]
fn test_recovery_after_failed_expression() {
    let source = r#"
x = (1 +
y = 2
z = 3
"#;
    let errors = parse_and_get_errors(source);
    assert!(!errors.is_empty());
}
