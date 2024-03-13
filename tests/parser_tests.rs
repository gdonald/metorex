// Comprehensive parser tests for Metorex

use metorex::ast::{BinaryOp, Expression, Statement, UnaryOp};
use metorex::lexer::Lexer;
use metorex::parser::Parser;

/// Helper function to parse source code into statements
fn parse_source(source: &str) -> Result<Vec<Statement>, String> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|errors| {
        errors
            .iter()
            .map(|e| format!("{}", e))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

// ============================================================================
// Literal Tests
// ============================================================================

#[test]
fn test_integer_literals() {
    let result = parse_source("42");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::IntLiteral { value, .. } => assert_eq!(*value, 42),
            _ => panic!("Expected IntLiteral, got {:?}", expression),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_float_literals() {
    let result = parse_source("3.14");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::FloatLiteral { value, .. } => assert_eq!(*value, 3.14),
            _ => panic!("Expected FloatLiteral"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_string_literals() {
    let result = parse_source("\"hello world\"");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::StringLiteral { value, .. } => assert_eq!(value, "hello world"),
            _ => panic!("Expected StringLiteral"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_boolean_literals() {
    let result = parse_source("true\nfalse");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 2);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::BoolLiteral { value, .. } => assert!(*value),
            _ => panic!("Expected BoolLiteral"),
        },
        _ => panic!("Expected Expression statement"),
    }

    match &statements[1] {
        Statement::Expression { expression, .. } => match expression {
            Expression::BoolLiteral { value, .. } => assert!(!*value),
            _ => panic!("Expected BoolLiteral"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_nil_literal() {
    let result = parse_source("nil");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::NilLiteral { .. } => {}
            _ => panic!("Expected NilLiteral"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

// ============================================================================
// Arithmetic Expression Tests
// ============================================================================

#[test]
fn test_simple_addition() {
    let result = parse_source("1 + 2");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::BinaryOp { op, .. } => assert!(matches!(op, BinaryOp::Add)),
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_operator_precedence() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let result = parse_source("1 + 2 * 3");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::BinaryOp {
                op: BinaryOp::Add,
                left,
                right,
                ..
            } => {
                // Left should be 1
                match left.as_ref() {
                    Expression::IntLiteral { value, .. } => assert_eq!(*value, 1),
                    _ => panic!("Expected IntLiteral on left"),
                }
                // Right should be (2 * 3)
                match right.as_ref() {
                    Expression::BinaryOp {
                        op: BinaryOp::Multiply,
                        ..
                    } => {}
                    _ => panic!("Expected Multiply on right"),
                }
            }
            _ => panic!("Expected BinaryOp Add"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_unary_minus() {
    let result = parse_source("-42");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::UnaryOp {
                op: UnaryOp::Minus,
                operand,
                ..
            } => match operand.as_ref() {
                Expression::IntLiteral { value, .. } => assert_eq!(*value, 42),
                _ => panic!("Expected IntLiteral"),
            },
            _ => panic!("Expected UnaryOp"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_parenthesized_expression() {
    let result = parse_source("(1 + 2) * 3");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::BinaryOp {
                op: BinaryOp::Multiply,
                left,
                ..
            } => {
                // Left should be a grouped expression
                match left.as_ref() {
                    Expression::Grouped { .. } => {}
                    _ => panic!("Expected Grouped expression"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

// ============================================================================
// Comparison Tests
// ============================================================================

#[test]
fn test_comparison_operators() {
    let sources = vec![
        ("1 == 2", BinaryOp::Equal),
        ("1 != 2", BinaryOp::NotEqual),
        ("1 < 2", BinaryOp::Less),
        ("1 > 2", BinaryOp::Greater),
        ("1 <= 2", BinaryOp::LessEqual),
        ("1 >= 2", BinaryOp::GreaterEqual),
    ];

    for (source, expected_op) in sources {
        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);
        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            Statement::Expression { expression, .. } => match expression {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(op, &expected_op, "Wrong operator for: {}", source)
                }
                _ => panic!("Expected BinaryOp for: {}", source),
            },
            _ => panic!("Expected Expression statement for: {}", source),
        }
    }
}

// ============================================================================
// Assignment Tests
// ============================================================================

#[test]
fn test_simple_assignment() {
    let result = parse_source("x = 42");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Assignment { target, value, .. } => {
            match target {
                Expression::Identifier { name, .. } => assert_eq!(name, "x"),
                _ => panic!("Expected Identifier"),
            }
            match value {
                Expression::IntLiteral { value, .. } => assert_eq!(*value, 42),
                _ => panic!("Expected IntLiteral"),
            }
        }
        _ => panic!("Expected Assignment statement"),
    }
}

#[test]
fn test_compound_assignment() {
    let result = parse_source("x += 1");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Assignment { value, .. } => {
            // Compound assignment should be converted to BinaryOp
            match value {
                Expression::BinaryOp {
                    op: BinaryOp::Add, ..
                } => {}
                _ => panic!("Expected BinaryOp Add"),
            }
        }
        _ => panic!("Expected Assignment statement"),
    }
}

// ============================================================================
// Variable Tests
// ============================================================================

#[test]
fn test_instance_variables() {
    let result = parse_source("@instance_var");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::InstanceVariable { name, .. } => assert_eq!(name, "instance_var"),
            _ => panic!("Expected InstanceVariable"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_class_variables() {
    let result = parse_source("@@class_var");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::ClassVariable { name, .. } => assert_eq!(name, "class_var"),
            _ => panic!("Expected ClassVariable"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

// ============================================================================
// Array Tests
// ============================================================================

#[test]
fn test_array_literal() {
    let result = parse_source("[1, 2, 3]");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Array { elements, .. } => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected Array"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_empty_array() {
    let result = parse_source("[]");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Array { elements, .. } => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected Array"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_array_indexing() {
    let result = parse_source("arr[0]");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Index { array, index, .. } => {
                match array.as_ref() {
                    Expression::Identifier { name, .. } => assert_eq!(name, "arr"),
                    _ => panic!("Expected Identifier"),
                }
                match index.as_ref() {
                    Expression::IntLiteral { value, .. } => assert_eq!(*value, 0),
                    _ => panic!("Expected IntLiteral"),
                }
            }
            _ => panic!("Expected Index"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

// ============================================================================
// Dictionary Tests
// ============================================================================

#[test]
fn test_dictionary_literal() {
    let result = parse_source("{\"key\": \"value\", \"count\": 42}");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Dictionary { entries, .. } => {
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("Expected Dictionary"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_empty_dictionary() {
    let result = parse_source("{}");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Dictionary { entries, .. } => {
                assert_eq!(entries.len(), 0);
            }
            _ => panic!("Expected Dictionary"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

// ============================================================================
// Function Call Tests
// ============================================================================

#[test]
fn test_function_call_no_args() {
    let result = parse_source("foo()");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Call {
                callee, arguments, ..
            } => {
                match callee.as_ref() {
                    Expression::Identifier { name, .. } => assert_eq!(name, "foo"),
                    _ => panic!("Expected Identifier"),
                }
                assert_eq!(arguments.len(), 0);
            }
            _ => panic!("Expected Call"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_function_call_with_args() {
    let result = parse_source("foo(1, 2, 3)");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Call {
                callee, arguments, ..
            } => {
                match callee.as_ref() {
                    Expression::Identifier { name, .. } => assert_eq!(name, "foo"),
                    _ => panic!("Expected Identifier"),
                }
                assert_eq!(arguments.len(), 3);
            }
            _ => panic!("Expected Call"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

// ============================================================================
// Method Call Tests
// ============================================================================

#[test]
fn test_method_call_no_args() {
    let result = parse_source("obj.method()");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                ..
            } => {
                match receiver.as_ref() {
                    Expression::Identifier { name, .. } => assert_eq!(name, "obj"),
                    _ => panic!("Expected Identifier"),
                }
                assert_eq!(method, "method");
                assert_eq!(arguments.len(), 0);
            }
            _ => panic!("Expected MethodCall"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_method_call_with_args() {
    let result = parse_source("obj.method(1, 2)");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::MethodCall {
                method, arguments, ..
            } => {
                assert_eq!(method, "method");
                assert_eq!(arguments.len(), 2);
            }
            _ => panic!("Expected MethodCall"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_chained_method_calls() {
    let result = parse_source("obj.method1().method2()");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::MethodCall {
                receiver, method, ..
            } => {
                assert_eq!(method, "method2");
                // Receiver should be another method call
                match receiver.as_ref() {
                    Expression::MethodCall { method, .. } => {
                        assert_eq!(method, "method1");
                    }
                    _ => panic!("Expected MethodCall"),
                }
            }
            _ => panic!("Expected MethodCall"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

// ============================================================================
// Function Definition Tests
// ============================================================================

#[test]
fn test_function_def_no_params() {
    let result = parse_source("def foo\n  42\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::FunctionDef {
            name,
            parameters,
            body,
            ..
        } => {
            assert_eq!(name, "foo");
            assert_eq!(parameters.len(), 0);
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected FunctionDef statement"),
    }
}

#[test]
fn test_function_def_with_params() {
    let result = parse_source("def add(x, y)\n  x + y\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::FunctionDef {
            name,
            parameters,
            body,
            ..
        } => {
            assert_eq!(name, "add");
            assert_eq!(parameters.len(), 2);
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected FunctionDef statement"),
    }
}

#[test]
fn test_function_def_with_default_params() {
    let result = parse_source("def greet(name, greeting = \"Hello\")\n  greeting\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::FunctionDef { parameters, .. } => {
            assert_eq!(parameters.len(), 2);
            assert!(parameters[0].is_simple());
            assert!(parameters[1].has_default());
        }
        _ => panic!("Expected FunctionDef statement"),
    }
}

// ============================================================================
// Class Definition Tests
// ============================================================================

#[test]
fn test_class_def_empty() {
    let result = parse_source("class Foo\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::ClassDef {
            name,
            superclass,
            body,
            ..
        } => {
            assert_eq!(name, "Foo");
            assert!(superclass.is_none());
            assert_eq!(body.len(), 0);
        }
        _ => panic!("Expected ClassDef statement"),
    }
}

#[test]
fn test_class_def_with_superclass() {
    let result = parse_source("class Foo < Bar\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::ClassDef {
            name, superclass, ..
        } => {
            assert_eq!(name, "Foo");
            assert_eq!(superclass.as_ref().unwrap(), "Bar");
        }
        _ => panic!("Expected ClassDef statement"),
    }
}

#[test]
fn test_class_def_with_body() {
    let result = parse_source("class Foo\n  def bar\n    42\n  end\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::ClassDef { body, .. } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                Statement::FunctionDef { name, .. } => {
                    assert_eq!(name, "bar");
                }
                _ => panic!("Expected FunctionDef in class body"),
            }
        }
        _ => panic!("Expected ClassDef statement"),
    }
}

// ============================================================================
// Control Flow Tests
// ============================================================================

#[test]
fn test_if_statement() {
    let result = parse_source("if true\n  42\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_if_else_statement() {
    let result = parse_source("if true\n  42\nelse\n  0\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
            assert_eq!(else_branch.as_ref().unwrap().len(), 1);
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_while_loop() {
    let result = parse_source("while true\n  42\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::While { body, .. } => {
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected While statement"),
    }
}

#[test]
fn test_while_do_loop() {
    let result = parse_source("while true do\n  42\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::While { body, .. } => {
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected While statement"),
    }
}

// ============================================================================
// Multiple Statement Tests
// ============================================================================

#[test]
fn test_multiple_statements() {
    let result = parse_source("x = 1\ny = 2\nz = 3");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 3);

    for stmt in statements {
        match stmt {
            Statement::Assignment { .. } => {}
            _ => panic!("Expected Assignment statement"),
        }
    }
}

#[test]
fn test_multiple_statements_with_blank_lines() {
    let result = parse_source("x = 1\n\ny = 2\n\n\nz = 3");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 3);
}

// ============================================================================
// Complex Program Tests
// ============================================================================

#[test]
fn test_complex_program() {
    let source = r#"
class Calculator
  def add(x, y)
    x + y
  end

  def multiply(x, y)
    x * y
  end
end

calc = Calculator()
result = calc.add(1, 2)
"#;

    let result = parse_source(source);
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 3);

    match &statements[0] {
        Statement::ClassDef { name, body, .. } => {
            assert_eq!(name, "Calculator");
            assert_eq!(body.len(), 2);
        }
        _ => panic!("Expected ClassDef"),
    }

    match &statements[1] {
        Statement::Assignment { .. } => {}
        _ => panic!("Expected Assignment"),
    }

    match &statements[2] {
        Statement::Assignment { .. } => {}
        _ => panic!("Expected Assignment"),
    }
}
