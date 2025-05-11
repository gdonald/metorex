// Unit tests for unless control flow execution in the VM

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

#[test]
fn test_unless_condition_false_executes_body() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::Unless {
        condition: Expression::BoolLiteral {
            value: false,
            position: pos(1, 8),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 3),
            },
            value: Expression::IntLiteral {
                value: 42,
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        else_branch: None,
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(42));
}

#[test]
fn test_unless_condition_true_skips_body() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 10),
            },
            position: pos(1, 1),
        },
        Statement::Unless {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(2, 8),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::IntLiteral {
                    value: 42,
                    position: pos(3, 12),
                },
                position: pos(3, 3),
            }],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(0)); // Should remain 0
}

#[test]
fn test_unless_with_else_condition_false() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::Unless {
        condition: Expression::BoolLiteral {
            value: false,
            position: pos(1, 8),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 3),
            },
            value: Expression::StringLiteral {
                value: "then".to_string(),
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(4, 3),
            },
            value: Expression::StringLiteral {
                value: "else".to_string(),
                position: pos(4, 12),
            },
            position: pos(4, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(Rc::new("then".to_string())));
}

#[test]
fn test_unless_with_else_condition_true() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::Unless {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 8),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 3),
            },
            value: Expression::StringLiteral {
                value: "then".to_string(),
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(4, 3),
            },
            value: Expression::StringLiteral {
                value: "else".to_string(),
                position: pos(4, 12),
            },
            position: pos(4, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(Rc::new("else".to_string())));
}

#[test]
fn test_unless_with_comparison() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 5,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Unless {
            condition: Expression::BinaryOp {
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 8),
                }),
                op: BinaryOp::Greater,
                right: Box::new(Expression::IntLiteral {
                    value: 10,
                    position: pos(2, 12),
                }),
                position: pos(2, 8),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::StringLiteral {
                    value: "not greater".to_string(),
                    position: pos(3, 12),
                },
                position: pos(3, 3),
            }],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(Rc::new("not greater".to_string())));
}

#[test]
fn test_unless_with_nil_is_falsy() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::NilLiteral {
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Unless {
            condition: Expression::Identifier {
                name: "x".to_string(),
                position: pos(2, 8),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::StringLiteral {
                    value: "x is nil".to_string(),
                    position: pos(3, 12),
                },
                position: pos(3, 3),
            }],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(Rc::new("x is nil".to_string())));
}

#[test]
fn test_unless_with_false_literal() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "flag".to_string(),
                position: pos(1, 1),
            },
            value: Expression::BoolLiteral {
                value: false,
                position: pos(1, 8),
            },
            position: pos(1, 1),
        },
        Statement::Unless {
            condition: Expression::Identifier {
                name: "flag".to_string(),
                position: pos(2, 8),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::StringLiteral {
                    value: "flag is false".to_string(),
                    position: pos(3, 12),
                },
                position: pos(3, 3),
            }],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(Rc::new("flag is false".to_string())));
}

#[test]
fn test_unless_nested_in_if() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "a".to_string(),
                position: pos(1, 1),
            },
            value: Expression::BoolLiteral {
                value: true,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "b".to_string(),
                position: pos(2, 1),
            },
            value: Expression::BoolLiteral {
                value: false,
                position: pos(2, 5),
            },
            position: pos(2, 1),
        },
        Statement::If {
            condition: Expression::Identifier {
                name: "a".to_string(),
                position: pos(3, 4),
            },
            then_branch: vec![Statement::Unless {
                condition: Expression::Identifier {
                    name: "b".to_string(),
                    position: pos(4, 10),
                },
                then_branch: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(5, 5),
                    },
                    value: Expression::StringLiteral {
                        value: "nested".to_string(),
                        position: pos(5, 14),
                    },
                    position: pos(5, 5),
                }],
                else_branch: None,
                position: pos(4, 3),
            }],
            elsif_branches: vec![],
            else_branch: None,
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(Rc::new("nested".to_string())));
}

#[test]
fn test_unless_multiple_statements_in_body() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::Unless {
        condition: Expression::BoolLiteral {
            value: false,
            position: pos(1, 8),
        },
        then_branch: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 3),
                },
                value: Expression::IntLiteral {
                    value: 10,
                    position: pos(2, 7),
                },
                position: pos(2, 3),
            },
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "y".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::IntLiteral {
                    value: 20,
                    position: pos(3, 7),
                },
                position: pos(3, 3),
            },
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(4, 3),
                },
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(4, 12),
                    }),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Identifier {
                        name: "y".to_string(),
                        position: pos(4, 16),
                    }),
                    position: pos(4, 12),
                },
                position: pos(4, 3),
            },
        ],
        else_branch: None,
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(30));
}
