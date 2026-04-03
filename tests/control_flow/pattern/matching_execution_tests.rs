// Unit tests for pattern matching execution in Metorex
// Tests cover literal patterns, identifier binding, wildcards, array/object destructuring, and guards

use metorex::ast::{Expression, MatchCase, MatchPattern, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

/// Create a Position at line 1, column 1
fn pos() -> Position {
    Position {
        line: 1,
        column: 1,
        offset: 0,
    }
}

#[test]
fn test_match_int_literal() {
    let mut vm = VirtualMachine::new();

    // match 42 when 42 => "matched" end
    let match_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::IntLiteral(42),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "matched".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_string_literal() {
    let mut vm = VirtualMachine::new();

    // match "hello" when "hello" => "matched" end
    let match_stmt = Statement::Match {
        expression: Expression::StringLiteral {
            value: "hello".to_string(),
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::StringLiteral("hello".to_string()),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "matched".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_bool_literal() {
    let mut vm = VirtualMachine::new();

    // match true when true => "matched" end
    let match_stmt = Statement::Match {
        expression: Expression::BoolLiteral {
            value: true,
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::BoolLiteral(true),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "matched".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_nil_literal() {
    let mut vm = VirtualMachine::new();

    // match nil when nil => "matched" end
    let match_stmt = Statement::Match {
        expression: Expression::NilLiteral { position: pos() },
        cases: vec![MatchCase {
            pattern: MatchPattern::NilLiteral,
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "matched".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_identifier_binding() {
    let mut vm = VirtualMachine::new();

    // match 42 when x => x end
    let match_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Object::Int(42)));
}

#[test]
fn test_match_wildcard() {
    let mut vm = VirtualMachine::new();

    // match 42 when _ => "matched" end
    let match_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Wildcard,
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "matched".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_array_exact() {
    let mut vm = VirtualMachine::new();

    // match [1, 2, 3] when [1, 2, 3] => "matched" end
    let match_stmt = Statement::Match {
        expression: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 2,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 3,
                    position: pos(),
                },
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::IntLiteral(1),
                MatchPattern::IntLiteral(2),
                MatchPattern::IntLiteral(3),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "matched".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_array_with_bindings() {
    let mut vm = VirtualMachine::new();

    // match [1, 2, 3] when [a, b, c] => a + b + c end
    let match_stmt = Statement::Match {
        expression: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 2,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 3,
                    position: pos(),
                },
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("a".to_string()),
                MatchPattern::Identifier("b".to_string()),
                MatchPattern::Identifier("c".to_string()),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Add,
                    left: Box::new(Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "a".to_string(),
                            position: pos(),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "b".to_string(),
                            position: pos(),
                        }),
                        position: pos(),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "c".to_string(),
                        position: pos(),
                    }),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Object::Int(6)));
}

#[test]
fn test_match_array_with_rest() {
    let mut vm = VirtualMachine::new();

    // match [1, 2, 3, 4, 5] when [first, ...rest] => rest end
    let match_stmt = Statement::Match {
        expression: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 2,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 3,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 4,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 5,
                    position: pos(),
                },
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("first".to_string()),
                MatchPattern::Rest("rest".to_string()),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "rest".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());

    // rest should be [2, 3, 4, 5]
    if let Some(Object::Array(array_rc)) = result.unwrap() {
        let array = array_rc.borrow();
        assert_eq!(array.len(), 4);
        assert_eq!(array[0], Object::Int(2));
        assert_eq!(array[1], Object::Int(3));
        assert_eq!(array[2], Object::Int(4));
        assert_eq!(array[3], Object::Int(5));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_match_array_rest_middle() {
    let mut vm = VirtualMachine::new();

    // match [1, 2, 3, 4, 5] when [first, ...middle, last] => middle end
    let match_stmt = Statement::Match {
        expression: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 2,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 3,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 4,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 5,
                    position: pos(),
                },
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("first".to_string()),
                MatchPattern::Rest("middle".to_string()),
                MatchPattern::Identifier("last".to_string()),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "middle".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());

    // middle should be [2, 3, 4]
    if let Some(Object::Array(array_rc)) = result.unwrap() {
        let array = array_rc.borrow();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0], Object::Int(2));
        assert_eq!(array[1], Object::Int(3));
        assert_eq!(array[2], Object::Int(4));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_match_object_pattern() {
    let mut vm = VirtualMachine::new();

    // Create a dictionary: {x: 10, y: 20}
    // match {x: 10, y: 20} when {x, y} => x + y end
    let match_stmt = Statement::Match {
        expression: Expression::Dictionary {
            entries: vec![
                (
                    Expression::StringLiteral {
                        value: "x".to_string(),
                        position: pos(),
                    },
                    Expression::IntLiteral {
                        value: 10,
                        position: pos(),
                    },
                ),
                (
                    Expression::StringLiteral {
                        value: "y".to_string(),
                        position: pos(),
                    },
                    Expression::IntLiteral {
                        value: 20,
                        position: pos(),
                    },
                ),
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Object(vec![
                ("x".to_string(), MatchPattern::Identifier("x".to_string())),
                ("y".to_string(), MatchPattern::Identifier("y".to_string())),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "y".to_string(),
                        position: pos(),
                    }),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Object::Int(30)));
}

#[test]
fn test_match_with_guard_true() {
    let mut vm = VirtualMachine::new();

    // match 42 when x if x > 40 => "big" end
    let match_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: Some(Expression::BinaryOp {
                op: metorex::ast::BinaryOp::Greater,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 40,
                    position: pos(),
                }),
                position: pos(),
            }),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "big".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_with_guard_false_fallthrough() {
    let mut vm = VirtualMachine::new();

    // match 30 when x if x > 40 => "big" when x => "small" end
    let match_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 30,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::Identifier("x".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Greater,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 40,
                        position: pos(),
                    }),
                    position: pos(),
                }),
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "big".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Identifier("x".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "small".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_multiple_cases() {
    let mut vm = VirtualMachine::new();

    // match 2 when 1 => "one" when 2 => "two" when 3 => "three" end
    let match_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 2,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "one".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::IntLiteral(2),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "two".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::IntLiteral(3),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "three".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_match_no_match_error() {
    let mut vm = VirtualMachine::new();

    // match 5 when 1 => "one" when 2 => "two" end
    // Should error because 5 doesn't match any pattern
    let match_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 5,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "one".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::IntLiteral(2),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "two".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No pattern matched")
    );
}

#[test]
fn test_match_nested_array_pattern() {
    let mut vm = VirtualMachine::new();

    // match [[1, 2], [3, 4]] when [[a, b], [c, d]] => a + b + c + d end
    let match_stmt = Statement::Match {
        expression: Expression::Array {
            elements: vec![
                Expression::Array {
                    elements: vec![
                        Expression::IntLiteral {
                            value: 1,
                            position: pos(),
                        },
                        Expression::IntLiteral {
                            value: 2,
                            position: pos(),
                        },
                    ],
                    position: pos(),
                },
                Expression::Array {
                    elements: vec![
                        Expression::IntLiteral {
                            value: 3,
                            position: pos(),
                        },
                        Expression::IntLiteral {
                            value: 4,
                            position: pos(),
                        },
                    ],
                    position: pos(),
                },
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Array(vec![
                    MatchPattern::Identifier("a".to_string()),
                    MatchPattern::Identifier("b".to_string()),
                ]),
                MatchPattern::Array(vec![
                    MatchPattern::Identifier("c".to_string()),
                    MatchPattern::Identifier("d".to_string()),
                ]),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Add,
                    left: Box::new(Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::Add,
                        left: Box::new(Expression::BinaryOp {
                            op: metorex::ast::BinaryOp::Add,
                            left: Box::new(Expression::Identifier {
                                name: "a".to_string(),
                                position: pos(),
                            }),
                            right: Box::new(Expression::Identifier {
                                name: "b".to_string(),
                                position: pos(),
                            }),
                            position: pos(),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "c".to_string(),
                            position: pos(),
                        }),
                        position: pos(),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "d".to_string(),
                        position: pos(),
                    }),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some(Object::Int(10)));
}

#[test]
fn test_match_mixed_array_patterns() {
    let mut vm = VirtualMachine::new();

    // match [1, "hello", true] when [1, x, y] => x end
    let match_stmt = Statement::Match {
        expression: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: pos(),
                },
                Expression::StringLiteral {
                    value: "hello".to_string(),
                    position: pos(),
                },
                Expression::BoolLiteral {
                    value: true,
                    position: pos(),
                },
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::IntLiteral(1),
                MatchPattern::Identifier("x".to_string()),
                MatchPattern::Identifier("y".to_string()),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                },
                position: pos(),
            }],
            position: pos(),
        }],
        position: pos(),
    };

    let result = vm.execute_program(&[match_stmt]);
    assert!(result.is_ok());
    if let Some(Object::String(s)) = result.unwrap() {
        assert_eq!(s.as_ref(), "hello");
    } else {
        panic!("Expected string result");
    }
}
