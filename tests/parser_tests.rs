use metorex::ast::{Expression, Statement};
use metorex::error::MetorexError;
use metorex::lexer::Lexer;
use metorex::parser::Parser;

fn parse_source(source: &str) -> Result<Vec<Statement>, Vec<MetorexError>> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_parse_integer_literal() {
    let result = parse_source("42");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::IntLiteral { value, .. } => assert_eq!(*value, 42),
            _ => panic!("Expected IntLiteral"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_parse_arithmetic() {
    let result = parse_source("1 + 2 * 3");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);
}

#[test]
fn test_parse_assignment() {
    let result = parse_source("x = 42");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Assignment { .. } => {}
        _ => panic!("Expected Assignment statement"),
    }
}

#[test]
fn test_parse_function_def() {
    let result = parse_source("def foo(x, y)\n  x + y\nend");
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
            assert_eq!(parameters.len(), 2);
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected FunctionDef statement"),
    }
}

#[test]
fn test_parse_class_def() {
    let result = parse_source("class Foo\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::ClassDef { name, .. } => {
            assert_eq!(name, "Foo");
        }
        _ => panic!("Expected ClassDef statement"),
    }
}

#[test]
fn test_parse_if_statement() {
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
fn test_parse_while_loop() {
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
fn test_parse_method_call() {
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
fn test_parse_array_literal() {
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
