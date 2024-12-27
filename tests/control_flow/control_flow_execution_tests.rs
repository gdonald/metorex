// Unit tests for control flow execution in the VM

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
                else_branch: None,
                position: pos(3, 3),
            }],
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

// Tests for While loops

#[test]
fn test_while_loop_basic() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(2, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(2, 7),
            },
            position: pos(2, 1),
        },
        Statement::While {
            condition: Expression::BinaryOp {
                op: BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "i".to_string(),
                    position: pos(3, 7),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 5,
                    position: pos(3, 11),
                }),
                position: pos(3, 9),
            },
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(4, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(4, 15),
                        }),
                        position: pos(4, 13),
                    },
                    position: pos(4, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "i".to_string(),
                        position: pos(5, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(5, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(5, 11),
                        }),
                        position: pos(5, 9),
                    },
                    position: pos(5, 3),
                },
            ],
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(10)); // 0+1+2+3+4 = 10
}

#[test]
fn test_while_loop_no_iterations() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "counter".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 11),
            },
            position: pos(1, 1),
        },
        Statement::While {
            condition: Expression::BoolLiteral {
                value: false,
                position: pos(2, 7),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "counter".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::IntLiteral {
                    value: 42,
                    position: pos(3, 13),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let counter = vm.environment().get("counter").unwrap();
    assert_eq!(counter, Object::Int(0)); // Should remain 0
}

#[test]
fn test_while_loop_with_break() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::While {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(2, 7),
            },
            body: vec![
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::GreaterEqual,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(3, 6),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 3,
                            position: pos(3, 11),
                        }),
                        position: pos(3, 8),
                    },
                    then_branch: vec![Statement::Break {
                        position: pos(4, 5),
                    }],
                    else_branch: None,
                    position: pos(3, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "i".to_string(),
                        position: pos(6, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(6, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(6, 11),
                        }),
                        position: pos(6, 9),
                    },
                    position: pos(6, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let i = vm.environment().get("i").unwrap();
    assert_eq!(i, Object::Int(3));
}

#[test]
fn test_while_loop_with_continue() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(2, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(2, 7),
            },
            position: pos(2, 1),
        },
        Statement::While {
            condition: Expression::BinaryOp {
                op: BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "i".to_string(),
                    position: pos(3, 7),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 5,
                    position: pos(3, 11),
                }),
                position: pos(3, 9),
            },
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "i".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(4, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(4, 11),
                        }),
                        position: pos(4, 9),
                    },
                    position: pos(4, 3),
                },
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(Expression::Identifier {
                                name: "i".to_string(),
                                position: pos(5, 6),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 2,
                                position: pos(5, 10),
                            }),
                            position: pos(5, 8),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 0,
                            position: pos(5, 15),
                        }),
                        position: pos(5, 13),
                    },
                    then_branch: vec![Statement::Continue {
                        position: pos(6, 5),
                    }],
                    else_branch: None,
                    position: pos(5, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(8, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(8, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(8, 15),
                        }),
                        position: pos(8, 13),
                    },
                    position: pos(8, 3),
                },
            ],
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(9)); // 1+3+5 = 9 (skips even numbers)
}

// Tests for For loops

#[test]
fn test_for_loop_over_array() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 7),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 20),
                    },
                    Expression::IntLiteral {
                        value: 4,
                        position: pos(2, 23),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "sum".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(3, 9),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 15),
                    }),
                    position: pos(3, 13),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(10)); // 1+2+3+4 = 10
}

#[test]
fn test_for_loop_over_empty_array() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "count".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 9),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![],
                position: pos(2, 13),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "count".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "count".to_string(),
                        position: pos(3, 11),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 1,
                        position: pos(3, 19),
                    }),
                    position: pos(3, 17),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let count = vm.environment().get("count").unwrap();
    assert_eq!(count, Object::Int(0)); // Should remain 0
}

#[test]
fn test_for_loop_with_break() {
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
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 20),
                    },
                    Expression::IntLiteral {
                        value: 4,
                        position: pos(2, 23),
                    },
                    Expression::IntLiteral {
                        value: 5,
                        position: pos(2, 26),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(3, 6),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 3,
                            position: pos(3, 11),
                        }),
                        position: pos(3, 8),
                    },
                    then_branch: vec![Statement::Break {
                        position: pos(4, 5),
                    }],
                    else_branch: None,
                    position: pos(3, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(6, 3),
                    },
                    value: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(6, 12),
                    },
                    position: pos(6, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(2)); // Last value before break
}

#[test]
fn test_for_loop_with_continue() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 7),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 20),
                    },
                    Expression::IntLiteral {
                        value: 4,
                        position: pos(2, 23),
                    },
                    Expression::IntLiteral {
                        value: 5,
                        position: pos(2, 26),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: pos(3, 6),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 2,
                                position: pos(3, 10),
                            }),
                            position: pos(3, 8),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 0,
                            position: pos(3, 15),
                        }),
                        position: pos(3, 13),
                    },
                    then_branch: vec![Statement::Continue {
                        position: pos(4, 5),
                    }],
                    else_branch: None,
                    position: pos(3, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(6, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(6, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(6, 15),
                        }),
                        position: pos(6, 13),
                    },
                    position: pos(6, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(9)); // 1+3+5 = 9 (skips even numbers)
}

#[test]
fn test_for_loop_variable_scope() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 100,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![Expression::IntLiteral {
                    value: 42,
                    position: pos(2, 14),
                }],
                position: pos(2, 13),
            },
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(3, 3),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let x = vm.environment().get("x").unwrap();
    assert_eq!(x, Object::Int(100)); // Should retain original value after loop
}

#[test]
fn test_nested_for_loops() {
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
        Statement::For {
            variable: "i".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![Statement::For {
                variable: "j".to_string(),
                iterable: Expression::Array {
                    elements: vec![
                        Expression::IntLiteral {
                            value: 10,
                            position: pos(3, 16),
                        },
                        Expression::IntLiteral {
                            value: 20,
                            position: pos(3, 20),
                        },
                    ],
                    position: pos(3, 15),
                },
                body: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(4, 5),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "result".to_string(),
                            position: pos(4, 14),
                        }),
                        right: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Add,
                            left: Box::new(Expression::Identifier {
                                name: "i".to_string(),
                                position: pos(4, 23),
                            }),
                            right: Box::new(Expression::Identifier {
                                name: "j".to_string(),
                                position: pos(4, 27),
                            }),
                            position: pos(4, 25),
                        }),
                        position: pos(4, 21),
                    },
                    position: pos(4, 5),
                }],
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(66)); // (1+10)+(1+20)+(2+10)+(2+20) = 11+21+12+22 = 66
}

#[test]
fn test_for_loop_error_on_non_iterable() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::For {
        variable: "x".to_string(),
        iterable: Expression::IntLiteral {
            value: 42,
            position: pos(1, 13),
        },
        body: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "x".to_string(),
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    }];

    let result = vm.execute_program(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Cannot iterate"));
}

// Tests for complex control flow combinations

#[test]
fn test_for_loop_in_if_branch() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "flag".to_string(),
                position: pos(1, 1),
            },
            value: Expression::BoolLiteral {
                value: true,
                position: pos(1, 8),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(2, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(2, 7),
            },
            position: pos(2, 1),
        },
        Statement::If {
            condition: Expression::Identifier {
                name: "flag".to_string(),
                position: pos(3, 4),
            },
            then_branch: vec![Statement::For {
                variable: "x".to_string(),
                iterable: Expression::Array {
                    elements: vec![
                        Expression::IntLiteral {
                            value: 1,
                            position: pos(4, 16),
                        },
                        Expression::IntLiteral {
                            value: 2,
                            position: pos(4, 19),
                        },
                        Expression::IntLiteral {
                            value: 3,
                            position: pos(4, 22),
                        },
                    ],
                    position: pos(4, 15),
                },
                body: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(5, 5),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(5, 11),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(5, 17),
                        }),
                        position: pos(5, 15),
                    },
                    position: pos(5, 5),
                }],
                position: pos(4, 3),
            }],
            else_branch: None,
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(6)); // 1+2+3 = 6
}

#[test]
fn test_while_loop_in_for_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "total".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 9),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "outer".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 18),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 21),
                    },
                ],
                position: pos(2, 17),
            },
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "inner".to_string(),
                        position: pos(3, 3),
                    },
                    value: Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 11),
                    },
                    position: pos(3, 3),
                },
                Statement::While {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Less,
                        left: Box::new(Expression::Identifier {
                            name: "inner".to_string(),
                            position: pos(4, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "outer".to_string(),
                            position: pos(4, 17),
                        }),
                        position: pos(4, 15),
                    },
                    body: vec![
                        Statement::Assignment {
                            target: Expression::Identifier {
                                name: "total".to_string(),
                                position: pos(5, 5),
                            },
                            value: Expression::BinaryOp {
                                op: BinaryOp::Add,
                                left: Box::new(Expression::Identifier {
                                    name: "total".to_string(),
                                    position: pos(5, 13),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 1,
                                    position: pos(5, 21),
                                }),
                                position: pos(5, 19),
                            },
                            position: pos(5, 5),
                        },
                        Statement::Assignment {
                            target: Expression::Identifier {
                                name: "inner".to_string(),
                                position: pos(6, 5),
                            },
                            value: Expression::BinaryOp {
                                op: BinaryOp::Add,
                                left: Box::new(Expression::Identifier {
                                    name: "inner".to_string(),
                                    position: pos(6, 13),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 1,
                                    position: pos(6, 21),
                                }),
                                position: pos(6, 19),
                            },
                            position: pos(6, 5),
                        },
                    ],
                    position: pos(4, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let total = vm.environment().get("total").unwrap();
    assert_eq!(total, Object::Int(5)); // outer=2: 2 iterations, outer=3: 3 iterations = 5 total
}

#[test]
fn test_control_flow_with_return() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(1, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(1, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(1, 20),
                    },
                ],
                position: pos(1, 13),
            },
            body: vec![Statement::If {
                condition: Expression::BinaryOp {
                    op: BinaryOp::Equal,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(2, 6),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 11),
                    }),
                    position: pos(2, 8),
                },
                then_branch: vec![Statement::Return {
                    value: Some(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 12),
                    }),
                    position: pos(3, 5),
                }],
                else_branch: None,
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::Return {
            value: Some(Expression::IntLiteral {
                value: 999,
                position: pos(6, 8),
            }),
            position: pos(6, 1),
        },
    ];

    let result = vm.execute_program(&program).unwrap();
    assert_eq!(result, Some(Object::Int(2)));
}

// Tests for Break statement

#[test]
fn test_break_in_while_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        // x = 0
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
        // while true
        Statement::While {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(2, 7),
            },
            body: vec![
                // x = x + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(3, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(3, 11),
                        }),
                        position: pos(3, 9),
                    },
                    position: pos(3, 3),
                },
                // if x >= 5: break
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::GreaterEqual,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(4, 6),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 5,
                            position: pos(4, 11),
                        }),
                        position: pos(4, 8),
                    },
                    then_branch: vec![Statement::Break {
                        position: pos(5, 5),
                    }],
                    else_branch: None,
                    position: pos(4, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();
    assert_eq!(vm.environment().get("x"), Some(Object::Int(5)));
}

#[test]
fn test_continue_in_while_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        // x = 0
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
        // count = 0
        Statement::Assignment {
            target: Expression::Identifier {
                name: "count".to_string(),
                position: pos(2, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(2, 9),
            },
            position: pos(2, 1),
        },
        // while x < 10
        Statement::While {
            condition: Expression::BinaryOp {
                op: BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(3, 7),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 10,
                    position: pos(3, 11),
                }),
                position: pos(3, 9),
            },
            body: vec![
                // x = x + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(4, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(4, 11),
                        }),
                        position: pos(4, 9),
                    },
                    position: pos(4, 3),
                },
                // if x % 2 == 0: continue
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: pos(5, 6),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 2,
                                position: pos(5, 10),
                            }),
                            position: pos(5, 8),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 0,
                            position: pos(5, 15),
                        }),
                        position: pos(5, 12),
                    },
                    then_branch: vec![Statement::Continue {
                        position: pos(6, 5),
                    }],
                    else_branch: None,
                    position: pos(5, 3),
                },
                // count = count + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "count".to_string(),
                        position: pos(7, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "count".to_string(),
                            position: pos(7, 11),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(7, 19),
                        }),
                        position: pos(7, 17),
                    },
                    position: pos(7, 3),
                },
            ],
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();
    // count should only increment on odd numbers (1,3,5,7,9) = 5 times
    assert_eq!(vm.environment().get("count"), Some(Object::Int(5)));
}
