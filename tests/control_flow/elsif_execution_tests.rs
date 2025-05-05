// Unit tests for elsif control flow execution in the VM

use metorex::ast::{BinaryOp, ElsifBranch, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

#[test]
fn test_elsif_first_condition_true() {
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
            value: Expression::IntLiteral {
                value: 1,
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![ElsifBranch {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(3, 7),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(4, 3),
                },
                value: Expression::IntLiteral {
                    value: 2,
                    position: pos(4, 12),
                },
                position: pos(4, 3),
            }],
            position: pos(3, 1),
        }],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(6, 3),
            },
            value: Expression::IntLiteral {
                value: 3,
                position: pos(6, 12),
            },
            position: pos(6, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(1)); // First branch should execute
}

#[test]
fn test_elsif_second_condition_true() {
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
            value: Expression::IntLiteral {
                value: 1,
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![ElsifBranch {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(3, 7),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(4, 3),
                },
                value: Expression::IntLiteral {
                    value: 2,
                    position: pos(4, 12),
                },
                position: pos(4, 3),
            }],
            position: pos(3, 1),
        }],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(6, 3),
            },
            value: Expression::IntLiteral {
                value: 3,
                position: pos(6, 12),
            },
            position: pos(6, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(2)); // Elsif branch should execute
}

#[test]
fn test_elsif_multiple_branches() {
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
            value: Expression::IntLiteral {
                value: 1,
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![
            ElsifBranch {
                condition: Expression::BoolLiteral {
                    value: false,
                    position: pos(3, 7),
                },
                body: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::IntLiteral {
                        value: 2,
                        position: pos(4, 12),
                    },
                    position: pos(4, 3),
                }],
                position: pos(3, 1),
            },
            ElsifBranch {
                condition: Expression::BoolLiteral {
                    value: true,
                    position: pos(5, 7),
                },
                body: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(6, 3),
                    },
                    value: Expression::IntLiteral {
                        value: 3,
                        position: pos(6, 12),
                    },
                    position: pos(6, 3),
                }],
                position: pos(5, 1),
            },
        ],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(8, 3),
            },
            value: Expression::IntLiteral {
                value: 4,
                position: pos(8, 12),
            },
            position: pos(8, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(3)); // Second elsif branch should execute
}

#[test]
fn test_elsif_all_false_fallback_to_else() {
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
            value: Expression::IntLiteral {
                value: 1,
                position: pos(2, 12),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![ElsifBranch {
            condition: Expression::BoolLiteral {
                value: false,
                position: pos(3, 7),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(4, 3),
                },
                value: Expression::IntLiteral {
                    value: 2,
                    position: pos(4, 12),
                },
                position: pos(4, 3),
            }],
            position: pos(3, 1),
        }],
        else_branch: Some(vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(6, 3),
            },
            value: Expression::IntLiteral {
                value: 3,
                position: pos(6, 12),
            },
            position: pos(6, 3),
        }]),
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(3)); // Else branch should execute
}

#[test]
fn test_elsif_without_else_no_match() {
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
        Statement::If {
            condition: Expression::BoolLiteral {
                value: false,
                position: pos(2, 4),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::IntLiteral {
                    value: 1,
                    position: pos(3, 12),
                },
                position: pos(3, 3),
            }],
            elsif_branches: vec![ElsifBranch {
                condition: Expression::BoolLiteral {
                    value: false,
                    position: pos(4, 7),
                },
                body: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(5, 3),
                    },
                    value: Expression::IntLiteral {
                        value: 2,
                        position: pos(5, 12),
                    },
                    position: pos(5, 3),
                }],
                position: pos(4, 1),
            }],
            else_branch: None,
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(0)); // Should remain 0 since no branch matched
}

#[test]
fn test_elsif_with_comparison() {
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
        Statement::If {
            condition: Expression::BinaryOp {
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 4),
                }),
                op: BinaryOp::Less,
                right: Box::new(Expression::IntLiteral {
                    value: 0,
                    position: pos(2, 8),
                }),
                position: pos(2, 4),
            },
            then_branch: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::StringLiteral {
                    value: "negative".to_string(),
                    position: pos(3, 12),
                },
                position: pos(3, 3),
            }],
            elsif_branches: vec![
                ElsifBranch {
                    condition: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(4, 7),
                        }),
                        op: BinaryOp::Equal,
                        right: Box::new(Expression::IntLiteral {
                            value: 0,
                            position: pos(4, 12),
                        }),
                        position: pos(4, 7),
                    },
                    body: vec![Statement::Assignment {
                        target: Expression::Identifier {
                            name: "result".to_string(),
                            position: pos(5, 3),
                        },
                        value: Expression::StringLiteral {
                            value: "zero".to_string(),
                            position: pos(5, 12),
                        },
                        position: pos(5, 3),
                    }],
                    position: pos(4, 1),
                },
                ElsifBranch {
                    condition: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(6, 7),
                        }),
                        op: BinaryOp::Less,
                        right: Box::new(Expression::IntLiteral {
                            value: 10,
                            position: pos(6, 11),
                        }),
                        position: pos(6, 7),
                    },
                    body: vec![Statement::Assignment {
                        target: Expression::Identifier {
                            name: "result".to_string(),
                            position: pos(7, 3),
                        },
                        value: Expression::StringLiteral {
                            value: "small".to_string(),
                            position: pos(7, 12),
                        },
                        position: pos(7, 3),
                    }],
                    position: pos(6, 1),
                },
            ],
            else_branch: Some(vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "result".to_string(),
                    position: pos(9, 3),
                },
                value: Expression::StringLiteral {
                    value: "large".to_string(),
                    position: pos(9, 12),
                },
                position: pos(9, 3),
            }]),
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(Rc::new("small".to_string()))); // Second elsif should match
}
