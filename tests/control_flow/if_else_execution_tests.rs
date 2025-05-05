// Unit tests for if/else control flow execution in the VM

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for If/Else execution

#[test]
fn test_if_with_true_condition() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(2, 3),
            },
            value: Expression::IntLiteral {
                value: 42,
                position: pos(2, 7),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let x = vm.environment().get("x").unwrap();
    assert_eq!(x, Object::Int(42));
}

#[test]
fn test_if_with_false_condition() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::If {
            condition: Expression::BoolLiteral {
                value: false,
                position: pos(2, 4),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::IntLiteral {
                    value: 42,
                    position: pos(3, 7),
                },
                position: pos(3, 3),
            }],
            elsif_branches: vec![],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let x = vm.environment().get("x").unwrap();
    assert_eq!(x, Object::Int(0)); // Should remain 0
}

#[test]
fn test_if_else_with_true_condition() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 3),
            },
            value: Expression::StringLiteral {
                value: "yes".to_string(),
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(4, 3),
            },
            value: Expression::StringLiteral {
                value: "no".to_string(),
                position: pos(4, 12),
            },
            position: pos(4, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::string("yes"));
}

#[test]
fn test_if_else_with_false_condition() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::If {
        condition: Expression::BoolLiteral {
            value: false,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 3),
            },
            value: Expression::StringLiteral {
                value: "yes".to_string(),
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(4, 3),
            },
            value: Expression::StringLiteral {
                value: "no".to_string(),
                position: pos(4, 12),
            },
            position: pos(4, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::string("no"));
}

#[test]
fn test_if_with_comparison_condition() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 10,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::If {
            condition: Expression::BinaryOp {
                op: BinaryOp::Greater,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 4),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 5,
                    position: pos(2, 8),
                }),
                position: pos(2, 6),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::StringLiteral {
                    value: "greater".to_string(),
                    position: pos(3, 12),
                },
                position: pos(3, 3),
            }],
            elsif_branches: vec![],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::string("greater"));
}

#[test]
fn test_nested_if_statements() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 15,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::If {
            condition: Expression::BinaryOp {
                op: BinaryOp::Greater,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 4),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 10,
                    position: pos(2, 8),
                }),
                position: pos(2, 6),
            },
            then_branch: vec![Statement::If {
                condition: Expression::BinaryOp {
                    op: BinaryOp::Less,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 6),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 20,
                        position: pos(3, 10),
                    }),
                    position: pos(3, 8),
                },
                then_branch: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(4, 5),
                    },
                    value: Expression::StringLiteral {
                        value: "between".to_string(),
                        position: pos(4, 14),
                    },
                    position: pos(4, 5),
                }],
                elsif_branches: vec![],
                else_branch: None,
                position: pos(3, 3),
            }],
            elsif_branches: vec![],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::string("between"));
}

#[test]
fn test_if_truthiness_values() {
    let mut vm = VirtualMachine::new();

    // Test that 0 is truthy
    let program = vec![Statement::If {
        condition: Expression::IntLiteral {
            value: 0,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 3),
            },
            value: Expression::StringLiteral {
                value: "truthy".to_string(),
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::string("truthy"));
}

#[test]
fn test_if_nil_is_falsy() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::If {
        condition: Expression::NilLiteral {
            position: pos(1, 4),
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
        elsif_branches: vec![],
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
    assert_eq!(result, Object::string("else"));
}
