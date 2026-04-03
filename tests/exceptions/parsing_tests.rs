// Unit tests for exception handling AST nodes

use metorex::ast::{Expression, RescueClause, Statement};
use metorex::lexer::Position;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for Raise statements

#[test]
fn test_raise_with_expression() {
    let stmt = Statement::Raise {
        exception: Some(Expression::StringLiteral {
            value: "Error occurred".to_string(),
            position: pos(1, 7),
        }),
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_raise_with_call() {
    let stmt = Statement::Raise {
        exception: Some(Expression::Call {
            callee: Box::new(Expression::Identifier {
                name: "StandardError".to_string(),
                position: pos(1, 7),
            }),
            arguments: vec![Expression::StringLiteral {
                value: "Something went wrong".to_string(),
                position: pos(1, 21),
            }],
            trailing_block: None,
            position: pos(1, 7),
        }),
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_bare_raise() {
    // Bare raise re-raises the current exception
    let stmt = Statement::Raise {
        exception: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

// Tests for simple Begin statements

#[test]
fn test_begin_with_single_rescue() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "file".to_string(),
                    position: pos(2, 3),
                }),
                method: "read".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["IOError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Error reading file".to_string(),
                    position: pos(4, 5),
                },
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_begin_with_multiple_rescue_clauses() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "db".to_string(),
                    position: pos(2, 3),
                }),
                method: "query".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![
            RescueClause {
                exception_types: vec!["NetworkError".to_string()],
                variable_name: Some("e".to_string()),
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "Network error".to_string(),
                        position: pos(4, 5),
                    },
                    position: pos(4, 5),
                }],
                position: pos(3, 1),
            },
            RescueClause {
                exception_types: vec!["TimeoutError".to_string()],
                variable_name: Some("e".to_string()),
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "Timeout".to_string(),
                        position: pos(6, 5),
                    },
                    position: pos(6, 5),
                }],
                position: pos(5, 1),
            },
        ],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_begin_with_catch_all_rescue() {
    // Rescue with no exception types catches all
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec![], // Empty means catch all
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Caught something".to_string(),
                    position: pos(4, 5),
                },
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_begin_with_else_clause() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["StandardError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Error".to_string(),
                    position: pos(4, 5),
                },
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: Some(vec![Statement::Expression {
            expression: Expression::StringLiteral {
                value: "Success".to_string(),
                position: pos(6, 3),
            },
            position: pos(6, 3),
        }]),
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_begin_with_ensure_block() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["StandardError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Error".to_string(),
                    position: pos(4, 5),
                },
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: Some(vec![Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "resource".to_string(),
                    position: pos(6, 3),
                }),
                method: "close".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(6, 3),
            },
            position: pos(6, 3),
        }]),
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_begin_with_all_clauses() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "file".to_string(),
                    position: pos(2, 3),
                }),
                method: "read".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["IOError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Error".to_string(),
                    position: pos(4, 5),
                },
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: Some(vec![Statement::Expression {
            expression: Expression::StringLiteral {
                value: "Success".to_string(),
                position: pos(6, 3),
            },
            position: pos(6, 3),
        }]),
        ensure_block: Some(vec![Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "file".to_string(),
                    position: pos(8, 3),
                }),
                method: "close".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(8, 3),
            },
            position: pos(8, 3),
        }]),
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_begin_with_multiple_exception_types() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec![
                "NetworkError".to_string(),
                "TimeoutError".to_string(),
                "ConnectionError".to_string(),
            ],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Network problem".to_string(),
                    position: pos(4, 5),
                },
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_rescue_without_variable_binding() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["StandardError".to_string()],
            variable_name: None, // No variable binding
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Error occurred".to_string(),
                    position: pos(4, 5),
                },
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for nested exception handling

#[test]
fn test_nested_begin_blocks() {
    let inner_begin = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(3, 5),
            },
            position: pos(3, 5),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["InnerError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Inner error".to_string(),
                    position: pos(5, 7),
                },
                position: pos(5, 7),
            }],
            position: pos(4, 3),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(2, 3),
    };

    let stmt = Statement::Begin {
        body: vec![inner_begin],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["OuterError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "Outer error".to_string(),
                    position: pos(8, 5),
                },
                position: pos(8, 5),
            }],
            position: pos(7, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_raise_in_rescue_clause() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["StandardError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![
                Statement::Expression {
                    expression: Expression::MethodCall {
                        receiver: Box::new(Expression::Identifier {
                            name: "logger".to_string(),
                            position: pos(4, 5),
                        }),
                        method: "error".to_string(),
                        arguments: vec![Expression::Identifier {
                            name: "e".to_string(),
                            position: pos(4, 19),
                        }],
                        trailing_block: None,
                        position: pos(4, 5),
                    },
                    position: pos(4, 5),
                },
                Statement::Raise {
                    exception: None, // Bare raise - re-raise
                    position: pos(5, 5),
                },
            ],
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_begin_in_method() {
    // Simulating: def method() begin ... rescue ... end end
    let begin_stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["StandardError".to_string()],
            variable_name: Some("e".to_string()),
            body: vec![Statement::Return {
                value: Some(Expression::NilLiteral {
                    position: pos(4, 12),
                }),
                position: pos(4, 5),
            }],
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 3),
    };

    let method = Statement::MethodDef {
        name: "safe_method".to_string(),
        parameters: vec![],
        body: vec![begin_stmt],
        position: pos(1, 1),
    };

    assert_eq!(method.position(), pos(1, 1));
}

#[test]
fn test_empty_rescue_body() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![RescueClause {
            exception_types: vec!["StandardError".to_string()],
            variable_name: None,
            body: vec![], // Empty rescue - just swallow the exception
            position: pos(3, 1),
        }],
        else_clause: None,
        ensure_block: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_begin_with_return_in_ensure() {
    let stmt = Statement::Begin {
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        rescue_clauses: vec![],
        else_clause: None,
        ensure_block: Some(vec![
            Statement::Expression {
                expression: Expression::MethodCall {
                    receiver: Box::new(Expression::Identifier {
                        name: "logger".to_string(),
                        position: pos(4, 3),
                    }),
                    method: "info".to_string(),
                    arguments: vec![Expression::StringLiteral {
                        value: "Done".to_string(),
                        position: pos(4, 17),
                    }],
                    trailing_block: None,
                    position: pos(4, 3),
                },
                position: pos(4, 3),
            },
            Statement::Return {
                value: Some(Expression::BoolLiteral {
                    value: true,
                    position: pos(5, 10),
                }),
                position: pos(5, 3),
            },
        ]),
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}
