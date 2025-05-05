// Tests for variable resolution in Metorex

use metorex::ast::node::{Expression, Parameter, Statement};
use metorex::lexer::Position;
use metorex::resolver::Resolver;

#[test]
fn test_simple_variable_declaration() {
    let mut resolver = Resolver::new();

    let stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "x".to_string(),
            position: Position::default(),
        },
        value: Expression::IntLiteral {
            value: 42,
            position: Position::default(),
        },
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
    assert!(result.variables.contains_key("x"));
    assert_eq!(result.variables.get("x").unwrap().depth, 0);
}

#[test]
fn test_variable_usage_after_declaration() {
    let mut resolver = Resolver::new();

    let stmts = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: Position::default(),
            },
            value: Expression::IntLiteral {
                value: 42,
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Expression {
            expression: Expression::Identifier {
                name: "x".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        },
    ];

    let result = resolver.resolve(&stmts);
    assert!(!result.has_errors());
    assert!(result.variables.get("x").unwrap().used);
}

#[test]
fn test_undefined_variable_error() {
    let mut resolver = Resolver::new();

    let stmt = Statement::Expression {
        expression: Expression::Identifier {
            name: "undefined_var".to_string(),
            position: Position::default(),
        },
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 1);
    // Check the error message
    let error_msg = format!("{}", result.errors[0]);
    assert!(error_msg.contains("Undefined variable 'undefined_var'"));
}

#[test]
fn test_variable_shadowing_warning() {
    let mut resolver = Resolver::new();

    // Declare x in outer scope
    let stmt1 = Statement::Assignment {
        target: Expression::Identifier {
            name: "x".to_string(),
            position: Position::default(),
        },
        value: Expression::IntLiteral {
            value: 1,
            position: Position::default(),
        },
        position: Position::default(),
    };

    // Declare x in function scope (shadows outer x)
    let stmt2 = Statement::FunctionDef {
        name: "foo".to_string(),
        parameters: vec![],
        body: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: Position::default(),
            },
            value: Expression::IntLiteral {
                value: 2,
                position: Position::default(),
            },
            position: Position::default(),
        }],
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt1, stmt2]);
    assert!(!result.has_errors());
    assert!(
        !result.warnings.is_empty(),
        "Expected warnings but got none"
    );
    // Find the shadowing warning (there may be unused variable warnings too)
    let has_shadow_warning = result.warnings.iter().any(|w| w.contains("shadows"));
    if !has_shadow_warning {
        eprintln!("Warnings: {:?}", result.warnings);
    }
    assert!(
        has_shadow_warning,
        "Expected 'shadows' warning but got: {:?}",
        result.warnings
    );
}

#[test]
fn test_function_parameters_in_scope() {
    let mut resolver = Resolver::new();

    // Function with parameter x, body uses x
    let stmt = Statement::FunctionDef {
        name: "foo".to_string(),
        parameters: vec![Parameter::simple("x".to_string(), Position::default())],
        body: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "x".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        }],
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn test_nested_scopes() {
    let mut resolver = Resolver::new();

    // Outer variable
    let outer_var = Statement::Assignment {
        target: Expression::Identifier {
            name: "outer".to_string(),
            position: Position::default(),
        },
        value: Expression::IntLiteral {
            value: 1,
            position: Position::default(),
        },
        position: Position::default(),
    };

    // Function that uses outer variable
    let func = Statement::FunctionDef {
        name: "foo".to_string(),
        parameters: vec![],
        body: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "outer".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        }],
        position: Position::default(),
    };

    let result = resolver.resolve(&[outer_var, func]);
    assert!(!result.has_errors());
}

#[test]
fn test_loop_variable_scope() {
    let mut resolver = Resolver::new();

    // for i in array
    let stmt = Statement::For {
        variable: "i".to_string(),
        iterable: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: Position::default(),
                },
                Expression::IntLiteral {
                    value: 2,
                    position: Position::default(),
                },
            ],
            position: Position::default(),
        },
        body: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "i".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        }],
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn test_if_statement_scope() {
    let mut resolver = Resolver::new();

    // if condition
    //   x = 10
    // end
    let stmt = Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: Position::default(),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: Position::default(),
            },
            value: Expression::IntLiteral {
                value: 10,
                position: Position::default(),
            },
            position: Position::default(),
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn test_binary_operation_resolution() {
    let mut resolver = Resolver::new();

    let stmts = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "a".to_string(),
                position: Position::default(),
            },
            value: Expression::IntLiteral {
                value: 5,
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "b".to_string(),
                position: Position::default(),
            },
            value: Expression::IntLiteral {
                value: 10,
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Expression {
            expression: Expression::BinaryOp {
                op: metorex::ast::node::BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "a".to_string(),
                    position: Position::default(),
                }),
                right: Box::new(Expression::Identifier {
                    name: "b".to_string(),
                    position: Position::default(),
                }),
                position: Position::default(),
            },
            position: Position::default(),
        },
    ];

    let result = resolver.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn test_unused_variable_warning() {
    let mut resolver = Resolver::new();

    let stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "unused".to_string(),
            position: Position::default(),
        },
        value: Expression::IntLiteral {
            value: 42,
            position: Position::default(),
        },
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
    assert!(!result.warnings.is_empty());
    assert!(result.warnings[0].contains("Unused variable 'unused'"));
}

#[test]
fn test_underscore_prefix_no_warning() {
    let mut resolver = Resolver::new();

    let stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "_unused".to_string(),
            position: Position::default(),
        },
        value: Expression::IntLiteral {
            value: 42,
            position: Position::default(),
        },
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
    assert!(result.warnings.is_empty()); // No warning for _ prefix
}

#[test]
fn test_method_call_resolution() {
    let mut resolver = Resolver::new();

    let stmts = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "obj".to_string(),
                position: Position::default(),
            },
            value: Expression::Array {
                elements: vec![],
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "obj".to_string(),
                    position: Position::default(),
                }),
                method: "length".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: Position::default(),
            },
            position: Position::default(),
        },
    ];

    let result = resolver.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn test_lambda_parameter_scope() {
    let mut resolver = Resolver::new();

    let stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "fn".to_string(),
            position: Position::default(),
        },
        value: Expression::Lambda {
            parameters: vec!["x".to_string()],
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: Position::default(),
                },
                position: Position::default(),
            }],
            captured_vars: None,
            position: Position::default(),
        },
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn test_array_indexing_resolution() {
    let mut resolver = Resolver::new();

    let stmts = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "arr".to_string(),
                position: Position::default(),
            },
            value: Expression::Array {
                elements: vec![Expression::IntLiteral {
                    value: 1,
                    position: Position::default(),
                }],
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Expression {
            expression: Expression::Index {
                array: Box::new(Expression::Identifier {
                    name: "arr".to_string(),
                    position: Position::default(),
                }),
                index: Box::new(Expression::IntLiteral {
                    value: 0,
                    position: Position::default(),
                }),
                position: Position::default(),
            },
            position: Position::default(),
        },
    ];

    let result = resolver.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn test_multiple_errors() {
    let mut resolver = Resolver::new();

    let stmts = vec![
        Statement::Expression {
            expression: Expression::Identifier {
                name: "undefined1".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Expression {
            expression: Expression::Identifier {
                name: "undefined2".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        },
    ];

    let result = resolver.resolve(&stmts);
    assert!(result.has_errors());
    assert_eq!(result.errors.len(), 2);
}

#[test]
fn test_class_definition_scope() {
    let mut resolver = Resolver::new();

    // class MyClass
    //   def initialize
    //     @x = 10
    //   end
    // end
    let stmt = Statement::ClassDef {
        name: "MyClass".to_string(),
        superclass: None,
        body: vec![Statement::MethodDef {
            name: "initialize".to_string(),
            parameters: vec![],
            body: vec![Statement::Assignment {
                target: Expression::InstanceVariable {
                    name: "@x".to_string(),
                    position: Position::default(),
                },
                value: Expression::IntLiteral {
                    value: 10,
                    position: Position::default(),
                },
                position: Position::default(),
            }],
            position: Position::default(),
        }],
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn test_while_loop_scope() {
    let mut resolver = Resolver::new();

    let stmts = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "i".to_string(),
                position: Position::default(),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::While {
            condition: Expression::BinaryOp {
                op: metorex::ast::node::BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "i".to_string(),
                    position: Position::default(),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 10,
                    position: Position::default(),
                }),
                position: Position::default(),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "i".to_string(),
                    position: Position::default(),
                },
                value: Expression::BinaryOp {
                    op: metorex::ast::node::BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "i".to_string(),
                        position: Position::default(),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 1,
                        position: Position::default(),
                    }),
                    position: Position::default(),
                },
                position: Position::default(),
            }],
            position: Position::default(),
        },
    ];

    let result = resolver.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn test_return_statement_resolution() {
    let mut resolver = Resolver::new();

    let stmt = Statement::FunctionDef {
        name: "get_value".to_string(),
        parameters: vec![],
        body: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: Position::default(),
                },
                value: Expression::IntLiteral {
                    value: 42,
                    position: Position::default(),
                },
                position: Position::default(),
            },
            Statement::Return {
                value: Some(Expression::Identifier {
                    name: "x".to_string(),
                    position: Position::default(),
                }),
                position: Position::default(),
            },
        ],
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn test_strict_mode_disabled() {
    let mut resolver = Resolver::with_strict_mode(false);

    let stmt = Statement::Expression {
        expression: Expression::Identifier {
            name: "undefined_var".to_string(),
            position: Position::default(),
        },
        position: Position::default(),
    };

    let result = resolver.resolve(&[stmt]);
    assert!(!result.has_errors()); // No error in non-strict mode
}

#[test]
fn test_dictionary_literal_resolution() {
    let mut resolver = Resolver::new();

    let stmts = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "key".to_string(),
                position: Position::default(),
            },
            value: Expression::StringLiteral {
                value: "name".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "value".to_string(),
                position: Position::default(),
            },
            value: Expression::StringLiteral {
                value: "Alice".to_string(),
                position: Position::default(),
            },
            position: Position::default(),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "dict".to_string(),
                position: Position::default(),
            },
            value: Expression::Dictionary {
                entries: vec![(
                    Expression::Identifier {
                        name: "key".to_string(),
                        position: Position::default(),
                    },
                    Expression::Identifier {
                        name: "value".to_string(),
                        position: Position::default(),
                    },
                )],
                position: Position::default(),
            },
            position: Position::default(),
        },
    ];

    let result = resolver.resolve(&stmts);
    assert!(!result.has_errors());
}
