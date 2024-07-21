// Unit tests for function and method definition AST nodes

use metorex::ast::{Expression, Parameter, Statement};
use metorex::lexer::Position;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for basic function definitions

#[test]
fn test_simple_function_definition() {
    let stmt = Statement::FunctionDef {
        name: "greet".to_string(),
        parameters: vec![Parameter::simple("name".to_string(), pos(1, 11))],
        body: vec![Statement::Return {
            value: Some(Expression::StringLiteral {
                value: "Hello".to_string(),
                position: pos(2, 3),
            }),
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}

#[test]
fn test_function_with_no_parameters() {
    let stmt = Statement::FunctionDef {
        name: "get_greeting".to_string(),
        parameters: vec![],
        body: vec![Statement::Return {
            value: Some(Expression::StringLiteral {
                value: "Hello, World!".to_string(),
                position: pos(2, 3),
            }),
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_function_with_multiple_parameters() {
    let stmt = Statement::FunctionDef {
        name: "add".to_string(),
        parameters: vec![
            Parameter::simple("a".to_string(), pos(1, 9)),
            Parameter::simple("b".to_string(), pos(1, 12)),
        ],
        body: vec![Statement::Return {
            value: Some(Expression::BinaryOp {
                op: metorex::ast::BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "a".to_string(),
                    position: pos(2, 3),
                }),
                right: Box::new(Expression::Identifier {
                    name: "b".to_string(),
                    position: pos(2, 7),
                }),
                position: pos(2, 5),
            }),
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for parameters with default values

#[test]
fn test_parameter_with_default_value() {
    let param = Parameter::with_default(
        "greeting".to_string(),
        Expression::StringLiteral {
            value: "Hello".to_string(),
            position: pos(1, 20),
        },
        pos(1, 10),
    );

    assert!(param.has_default());
    assert!(!param.is_simple());
    assert!(!param.is_variadic);
    assert!(!param.is_keyword);
}

#[test]
fn test_function_with_default_parameters() {
    let stmt = Statement::FunctionDef {
        name: "greet".to_string(),
        parameters: vec![
            Parameter::simple("name".to_string(), pos(1, 11)),
            Parameter::with_default(
                "greeting".to_string(),
                Expression::StringLiteral {
                    value: "Hello".to_string(),
                    position: pos(1, 30),
                },
                pos(1, 17),
            ),
        ],
        body: vec![Statement::Return {
            value: Some(Expression::BinaryOp {
                op: metorex::ast::BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "greeting".to_string(),
                    position: pos(2, 3),
                }),
                right: Box::new(Expression::Identifier {
                    name: "name".to_string(),
                    position: pos(2, 14),
                }),
                position: pos(2, 12),
            }),
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_function_with_multiple_default_parameters() {
    let stmt = Statement::FunctionDef {
        name: "format_message".to_string(),
        parameters: vec![
            Parameter::simple("text".to_string(), pos(1, 20)),
            Parameter::with_default(
                "prefix".to_string(),
                Expression::StringLiteral {
                    value: "[INFO]".to_string(),
                    position: pos(1, 35),
                },
                pos(1, 26),
            ),
            Parameter::with_default(
                "suffix".to_string(),
                Expression::StringLiteral {
                    value: "!".to_string(),
                    position: pos(1, 55),
                },
                pos(1, 48),
            ),
        ],
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for variadic parameters (*args)

#[test]
fn test_variadic_parameter() {
    let param = Parameter::variadic("args".to_string(), pos(1, 10));

    assert!(param.is_variadic);
    assert!(!param.is_keyword);
    assert!(!param.has_default());
    assert!(!param.is_simple());
}

#[test]
fn test_function_with_variadic_parameter() {
    let stmt = Statement::FunctionDef {
        name: "sum_all".to_string(),
        parameters: vec![Parameter::variadic("numbers".to_string(), pos(1, 13))],
        body: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "total".to_string(),
                    position: pos(2, 3),
                },
                value: Expression::IntLiteral {
                    value: 0,
                    position: pos(2, 11),
                },
                position: pos(2, 3),
            },
            Statement::Return {
                value: Some(Expression::Identifier {
                    name: "total".to_string(),
                    position: pos(3, 3),
                }),
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_function_with_regular_and_variadic_parameters() {
    let stmt = Statement::FunctionDef {
        name: "process".to_string(),
        parameters: vec![
            Parameter::simple("first".to_string(), pos(1, 13)),
            Parameter::variadic("rest".to_string(), pos(1, 20)),
        ],
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for keyword parameters (**kwargs)

#[test]
fn test_keyword_parameter() {
    let param = Parameter::keyword("kwargs".to_string(), pos(1, 10));

    assert!(param.is_keyword);
    assert!(!param.is_variadic);
    assert!(!param.has_default());
    assert!(!param.is_simple());
}

#[test]
fn test_function_with_keyword_parameter() {
    let stmt = Statement::FunctionDef {
        name: "configure".to_string(),
        parameters: vec![Parameter::keyword("options".to_string(), pos(1, 15))],
        body: vec![Statement::Expression {
            expression: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "puts".to_string(),
                    position: pos(2, 3),
                }),
                arguments: vec![Expression::StringLiteral {
                    value: "Configuring...".to_string(),
                    position: pos(2, 8),
                }],
                trailing_block: None,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for mixed parameter types

#[test]
fn test_function_with_all_parameter_types() {
    let stmt = Statement::FunctionDef {
        name: "complex_function".to_string(),
        parameters: vec![
            Parameter::simple("required".to_string(), pos(1, 22)),
            Parameter::with_default(
                "optional".to_string(),
                Expression::StringLiteral {
                    value: "default".to_string(),
                    position: pos(1, 42),
                },
                pos(1, 32),
            ),
            Parameter::variadic("args".to_string(), pos(1, 52)),
            Parameter::keyword("kwargs".to_string(), pos(1, 59)),
        ],
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for method definitions

#[test]
fn test_simple_method_definition() {
    let stmt = Statement::MethodDef {
        name: "get_name".to_string(),
        parameters: vec![],
        body: vec![Statement::Return {
            value: Some(Expression::InstanceVariable {
                name: "name".to_string(),
                position: pos(2, 3),
            }),
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}

#[test]
fn test_method_with_parameters() {
    let stmt = Statement::MethodDef {
        name: "set_value".to_string(),
        parameters: vec![Parameter::simple("value".to_string(), pos(1, 15))],
        body: vec![Statement::Assignment {
            target: Expression::InstanceVariable {
                name: "value".to_string(),
                position: pos(2, 3),
            },
            value: Expression::Identifier {
                name: "value".to_string(),
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_method_with_default_parameters() {
    let stmt = Statement::MethodDef {
        name: "initialize".to_string(),
        parameters: vec![
            Parameter::simple("name".to_string(), pos(1, 16)),
            Parameter::with_default(
                "age".to_string(),
                Expression::IntLiteral {
                    value: 0,
                    position: pos(1, 28),
                },
                pos(1, 22),
            ),
        ],
        body: vec![
            Statement::Assignment {
                target: Expression::InstanceVariable {
                    name: "name".to_string(),
                    position: pos(2, 3),
                },
                value: Expression::Identifier {
                    name: "name".to_string(),
                    position: pos(2, 11),
                },
                position: pos(2, 3),
            },
            Statement::Assignment {
                target: Expression::InstanceVariable {
                    name: "age".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::Identifier {
                    name: "age".to_string(),
                    position: pos(3, 10),
                },
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for function bodies with complex statements

#[test]
fn test_function_with_conditional_body() {
    let stmt = Statement::FunctionDef {
        name: "max".to_string(),
        parameters: vec![
            Parameter::simple("a".to_string(), pos(1, 9)),
            Parameter::simple("b".to_string(), pos(1, 12)),
        ],
        body: vec![Statement::If {
            condition: Expression::BinaryOp {
                op: metorex::ast::BinaryOp::Greater,
                left: Box::new(Expression::Identifier {
                    name: "a".to_string(),
                    position: pos(2, 6),
                }),
                right: Box::new(Expression::Identifier {
                    name: "b".to_string(),
                    position: pos(2, 10),
                }),
                position: pos(2, 8),
            },
            then_branch: vec![Statement::Return {
                value: Some(Expression::Identifier {
                    name: "a".to_string(),
                    position: pos(3, 5),
                }),
                position: pos(3, 5),
            }],
            else_branch: Some(vec![Statement::Return {
                value: Some(Expression::Identifier {
                    name: "b".to_string(),
                    position: pos(5, 5),
                }),
                position: pos(5, 5),
            }]),
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_function_with_loop_body() {
    let stmt = Statement::FunctionDef {
        name: "factorial".to_string(),
        parameters: vec![Parameter::simple("n".to_string(), pos(1, 15))],
        body: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(2, 3),
                },
                value: Expression::IntLiteral {
                    value: 1,
                    position: pos(2, 12),
                },
                position: pos(2, 3),
            },
            Statement::While {
                condition: Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Greater,
                    left: Box::new(Expression::Identifier {
                        name: "n".to_string(),
                        position: pos(3, 9),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 1,
                        position: pos(3, 13),
                    }),
                    position: pos(3, 11),
                },
                body: vec![
                    Statement::Assignment {
                        target: Expression::Identifier {
                            name: "result".to_string(),
                            position: pos(4, 5),
                        },
                        value: Expression::BinaryOp {
                            op: metorex::ast::BinaryOp::Multiply,
                            left: Box::new(Expression::Identifier {
                                name: "result".to_string(),
                                position: pos(4, 14),
                            }),
                            right: Box::new(Expression::Identifier {
                                name: "n".to_string(),
                                position: pos(4, 23),
                            }),
                            position: pos(4, 21),
                        },
                        position: pos(4, 5),
                    },
                    Statement::Assignment {
                        target: Expression::Identifier {
                            name: "n".to_string(),
                            position: pos(5, 5),
                        },
                        value: Expression::BinaryOp {
                            op: metorex::ast::BinaryOp::Subtract,
                            left: Box::new(Expression::Identifier {
                                name: "n".to_string(),
                                position: pos(5, 9),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 1,
                                position: pos(5, 13),
                            }),
                            position: pos(5, 11),
                        },
                        position: pos(5, 5),
                    },
                ],
                position: pos(3, 3),
            },
            Statement::Return {
                value: Some(Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(7, 3),
                }),
                position: pos(7, 3),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_function_with_exception_handling() {
    let stmt = Statement::FunctionDef {
        name: "safe_operation".to_string(),
        parameters: vec![Parameter::simple("x".to_string(), pos(1, 20))],
        body: vec![Statement::Begin {
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Divide,
                    left: Box::new(Expression::IntLiteral {
                        value: 100,
                        position: pos(3, 5),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 11),
                    }),
                    position: pos(3, 9),
                }),
                position: pos(3, 5),
            }],
            rescue_clauses: vec![metorex::ast::RescueClause {
                exception_types: vec!["ZeroDivisionError".to_string()],
                variable_name: None,
                body: vec![Statement::Return {
                    value: Some(Expression::NilLiteral {
                        position: pos(5, 5),
                    }),
                    position: pos(5, 5),
                }],
                position: pos(4, 3),
            }],
            else_clause: None,
            ensure_block: None,
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for nested function definitions

#[test]
fn test_nested_function_definition() {
    let stmt = Statement::FunctionDef {
        name: "outer".to_string(),
        parameters: vec![Parameter::simple("x".to_string(), pos(1, 11))],
        body: vec![Statement::FunctionDef {
            name: "inner".to_string(),
            parameters: vec![Parameter::simple("y".to_string(), pos(2, 13))],
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 5),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "y".to_string(),
                        position: pos(3, 9),
                    }),
                    position: pos(3, 7),
                }),
                position: pos(3, 5),
            }],
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for Parameter helper methods

#[test]
fn test_parameter_simple_constructor() {
    let param = Parameter::simple("arg".to_string(), pos(1, 10));

    assert!(param.is_simple());
    assert!(!param.has_default());
    assert!(!param.is_variadic);
    assert!(!param.is_keyword);
    assert_eq!(param.name, "arg");
}

#[test]
fn test_parameter_with_default_constructor() {
    let param = Parameter::with_default(
        "arg".to_string(),
        Expression::IntLiteral {
            value: 42,
            position: pos(1, 20),
        },
        pos(1, 10),
    );

    assert!(!param.is_simple());
    assert!(param.has_default());
    assert!(!param.is_variadic);
    assert!(!param.is_keyword);
}

#[test]
fn test_parameter_variadic_constructor() {
    let param = Parameter::variadic("args".to_string(), pos(1, 10));

    assert!(!param.is_simple());
    assert!(!param.has_default());
    assert!(param.is_variadic);
    assert!(!param.is_keyword);
}

#[test]
fn test_parameter_keyword_constructor() {
    let param = Parameter::keyword("kwargs".to_string(), pos(1, 10));

    assert!(!param.is_simple());
    assert!(!param.has_default());
    assert!(!param.is_variadic);
    assert!(param.is_keyword);
}

// Tests for empty function bodies

#[test]
fn test_function_with_empty_body() {
    let stmt = Statement::FunctionDef {
        name: "noop".to_string(),
        parameters: vec![],
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}

#[test]
fn test_method_with_empty_body() {
    let stmt = Statement::MethodDef {
        name: "noop".to_string(),
        parameters: vec![],
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}
