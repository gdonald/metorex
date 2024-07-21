use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::error::MetorexError;
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

#[test]
fn block_returns_last_expression() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "block".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Lambda {
                parameters: vec!["x".to_string()],
                body: vec![Statement::Expression {
                    expression: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(1, 10),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 5,
                            position: pos(1, 15),
                        }),
                        position: pos(1, 13),
                    },
                    position: pos(1, 9),
                }],
                captured_vars: None,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "block".to_string(),
                    position: pos(2, 1),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 7,
                    position: pos(2, 8),
                }],
                trailing_block: None,
                position: pos(2, 1),
            },
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program)
        .expect("block execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(12)));
}

#[test]
fn block_captures_outer_variable() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "outer".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 3,
                position: pos(1, 10),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "block".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Lambda {
                parameters: vec!["y".to_string()],
                body: vec![Statement::Expression {
                    expression: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "outer".to_string(),
                            position: pos(2, 15),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "y".to_string(),
                            position: pos(2, 23),
                        }),
                        position: pos(2, 19),
                    },
                    position: pos(2, 11),
                }],
                captured_vars: Some(vec!["outer".to_string()]),
                position: pos(2, 5),
            },
            position: pos(2, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(3, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "block".to_string(),
                    position: pos(3, 1),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 9,
                    position: pos(3, 8),
                }],
                trailing_block: None,
                position: pos(3, 1),
            },
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program)
        .expect("block execution failed");
    assert_eq!(vm.environment().get("sum"), Some(Object::Int(12)));
}

#[test]
fn block_argument_mismatch_raises_error() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "block".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Lambda {
                parameters: vec!["x".to_string()],
                body: vec![Statement::Expression {
                    expression: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(1, 12),
                    },
                    position: pos(1, 12),
                }],
                captured_vars: None,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Expression {
            expression: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "block".to_string(),
                    position: pos(2, 1),
                }),
                arguments: vec![],
                trailing_block: None,
                position: pos(2, 1),
            },
            position: pos(2, 1),
        },
    ];

    match vm.execute_program(&program) {
        Err(MetorexError::RuntimeError { message, .. }) => {
            assert!(
                message.contains("expected 1 argument"),
                "unexpected runtime message: {}",
                message
            );
        }
        other => panic!("expected runtime error, got {:?}", other),
    }
}

#[test]
fn block_return_statement_exits_early() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "block".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Lambda {
                parameters: vec![],
                body: vec![
                    Statement::Return {
                        value: Some(Expression::IntLiteral {
                            value: 42,
                            position: pos(1, 15),
                        }),
                        position: pos(1, 11),
                    },
                    Statement::Expression {
                        expression: Expression::IntLiteral {
                            value: 0,
                            position: pos(1, 25),
                        },
                        position: pos(1, 21),
                    },
                ],
                captured_vars: None,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "value".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "block".to_string(),
                    position: pos(2, 1),
                }),
                arguments: vec![],
                trailing_block: None,
                position: pos(2, 1),
            },
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program)
        .expect("block with return should execute");
    assert_eq!(vm.environment().get("value"), Some(Object::Int(42)));
}
