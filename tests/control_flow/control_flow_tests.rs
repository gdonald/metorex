// Unit tests for control flow AST nodes

use metorex::ast::{BinaryOp, Expression, MatchCase, MatchPattern, Statement};
use metorex::lexer::Position;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for If statements

#[test]
fn test_if_statement_simple() {
    let stmt = Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
    assert!(!stmt.is_definition());
}

#[test]
fn test_if_statement_with_else() {
    let stmt = Statement::If {
        condition: Expression::BoolLiteral {
            value: false,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![],
        else_branch: Some(vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 2,
                position: pos(4, 3),
            },
            position: pos(4, 3),
        }]),
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_if_statement_with_comparison() {
    let stmt = Statement::If {
        condition: Expression::BinaryOp {
            op: BinaryOp::Greater,
            left: Box::new(Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 4),
            }),
            right: Box::new(Expression::IntLiteral {
                value: 0,
                position: pos(1, 8),
            }),
            position: pos(1, 6),
        },
        then_branch: vec![Statement::Expression {
            expression: Expression::StringLiteral {
                value: "positive".to_string(),
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_nested_if_statements() {
    let inner_if = Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(2, 6),
        },
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(3, 5),
            },
            position: pos(3, 5),
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(2, 3),
    };

    let stmt = Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 4),
        },
        then_branch: vec![inner_if],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for While loops

#[test]
fn test_while_loop_simple() {
    let stmt = Statement::While {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 7),
        },
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_while_loop_with_condition() {
    let stmt = Statement::While {
        condition: Expression::BinaryOp {
            op: BinaryOp::Less,
            left: Box::new(Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 7),
            }),
            right: Box::new(Expression::IntLiteral {
                value: 10,
                position: pos(1, 11),
            }),
            position: pos(1, 9),
        },
        body: vec![
            Statement::Expression {
                expression: Expression::MethodCall {
                    receiver: Box::new(Expression::Identifier {
                        name: "i".to_string(),
                        position: pos(2, 3),
                    }),
                    method: "process".to_string(),
                    arguments: vec![],
                    trailing_block: None,
                    position: pos(2, 3),
                },
                position: pos(2, 3),
            },
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "i".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "i".to_string(),
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
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for For loops

#[test]
fn test_for_loop_simple() {
    let stmt = Statement::For {
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
        body: vec![Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 3),
                }),
                method: "print".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_for_loop_with_range() {
    let stmt = Statement::For {
        variable: "i".to_string(),
        iterable: Expression::MethodCall {
            receiver: Box::new(Expression::IntLiteral {
                value: 0,
                position: pos(1, 13),
            }),
            method: "upto".to_string(),
            arguments: vec![Expression::IntLiteral {
                value: 10,
                position: pos(1, 21),
            }],
            trailing_block: None,
            position: pos(1, 13),
        },
        body: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "i".to_string(),
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_for_loop_nested() {
    let inner_for = Statement::For {
        variable: "y".to_string(),
        iterable: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: pos(2, 16),
                },
                Expression::IntLiteral {
                    value: 2,
                    position: pos(2, 19),
                },
            ],
            position: pos(2, 15),
        },
        body: vec![Statement::Expression {
            expression: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(3, 5),
                }),
                right: Box::new(Expression::Identifier {
                    name: "y".to_string(),
                    position: pos(3, 9),
                }),
                position: pos(3, 7),
            },
            position: pos(3, 5),
        }],
        position: pos(2, 3),
    };

    let stmt = Statement::For {
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
            ],
            position: pos(1, 13),
        },
        body: vec![inner_for],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for Match statements

#[test]
fn test_match_statement_simple() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(0),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "zero".to_string(),
                        position: pos(2, 12),
                    },
                    position: pos(2, 12),
                }],
                position: pos(2, 3),
            },
            MatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "one".to_string(),
                        position: pos(3, 12),
                    },
                    position: pos(3, 12),
                }],
                position: pos(3, 3),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "other".to_string(),
                        position: pos(4, 12),
                    },
                    position: pos(4, 12),
                }],
                position: pos(4, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_match_with_guard() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Identifier("n".to_string()),
            guard: Some(Expression::BinaryOp {
                op: BinaryOp::Greater,
                left: Box::new(Expression::Identifier {
                    name: "n".to_string(),
                    position: pos(2, 12),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 0,
                    position: pos(2, 16),
                }),
                position: pos(2, 14),
            }),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "positive".to_string(),
                    position: pos(2, 25),
                },
                position: pos(2, 25),
            }],
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_with_string_patterns() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "command".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::StringLiteral("start".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::MethodCall {
                        receiver: Box::new(Expression::Identifier {
                            name: "server".to_string(),
                            position: pos(2, 18),
                        }),
                        method: "start".to_string(),
                        arguments: vec![],
                        trailing_block: None,
                        position: pos(2, 18),
                    },
                    position: pos(2, 18),
                }],
                position: pos(2, 3),
            },
            MatchCase {
                pattern: MatchPattern::StringLiteral("stop".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::MethodCall {
                        receiver: Box::new(Expression::Identifier {
                            name: "server".to_string(),
                            position: pos(3, 17),
                        }),
                        method: "stop".to_string(),
                        arguments: vec![],
                        trailing_block: None,
                        position: pos(3, 17),
                    },
                    position: pos(3, 17),
                }],
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_with_boolean_patterns() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "flag".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::BoolLiteral(true),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 15),
                    },
                    position: pos(2, 15),
                }],
                position: pos(2, 3),
            },
            MatchCase {
                pattern: MatchPattern::BoolLiteral(false),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 16),
                    },
                    position: pos(3, 16),
                }],
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_with_nil_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "value".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::NilLiteral,
                guard: None,
                body: vec![Statement::Return {
                    value: Some(Expression::IntLiteral {
                        value: 0,
                        position: pos(2, 19),
                    }),
                    position: pos(2, 12),
                }],
                position: pos(2, 3),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::Identifier {
                        name: "value".to_string(),
                        position: pos(3, 12),
                    },
                    position: pos(3, 12),
                }],
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_with_array_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("first".to_string()),
                MatchPattern::Identifier("second".to_string()),
                MatchPattern::Wildcard,
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "first".to_string(),
                    position: pos(2, 25),
                },
                position: pos(2, 25),
            }],
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_with_type_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "obj".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::Type("String".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "is string".to_string(),
                        position: pos(2, 19),
                    },
                    position: pos(2, 19),
                }],
                position: pos(2, 3),
            },
            MatchCase {
                pattern: MatchPattern::Type("Integer".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "is integer".to_string(),
                        position: pos(3, 20),
                    },
                    position: pos(3, 20),
                }],
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

// Complex control flow combinations

#[test]
fn test_for_loop_with_break() {
    let stmt = Statement::For {
        variable: "item".to_string(),
        iterable: Expression::Identifier {
            name: "items".to_string(),
            position: pos(1, 14),
        },
        body: vec![
            Statement::If {
                condition: Expression::BinaryOp {
                    op: BinaryOp::Equal,
                    left: Box::new(Expression::Identifier {
                        name: "item".to_string(),
                        position: pos(2, 6),
                    }),
                    right: Box::new(Expression::NilLiteral {
                        position: pos(2, 14),
                    }),
                    position: pos(2, 11),
                },
                then_branch: vec![Statement::Break {
                    position: pos(3, 5),
                }],
                elsif_branches: vec![],
                else_branch: None,
                position: pos(2, 3),
            },
            Statement::Expression {
                expression: Expression::MethodCall {
                    receiver: Box::new(Expression::Identifier {
                        name: "item".to_string(),
                        position: pos(5, 3),
                    }),
                    method: "process".to_string(),
                    arguments: vec![],
                    trailing_block: None,
                    position: pos(5, 3),
                },
                position: pos(5, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_in_loop() {
    let match_stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "value".to_string(),
            position: pos(2, 9),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(0),
                guard: None,
                body: vec![Statement::Continue {
                    position: pos(3, 14),
                }],
                position: pos(3, 5),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::Identifier {
                        name: "value".to_string(),
                        position: pos(4, 14),
                    },
                    position: pos(4, 14),
                }],
                position: pos(4, 5),
            },
        ],
        position: pos(2, 3),
    };

    let stmt = Statement::While {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 7),
        },
        body: vec![match_stmt],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_nested_control_flow() {
    // if condition1
    //   for x in items
    //     match x
    //       case 0 -> continue
    //       case _ -> break
    //     end
    //   end
    // end

    let match_stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "x".to_string(),
            position: pos(3, 11),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(0),
                guard: None,
                body: vec![Statement::Continue {
                    position: pos(4, 17),
                }],
                position: pos(4, 7),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Break {
                    position: pos(5, 17),
                }],
                position: pos(5, 7),
            },
        ],
        position: pos(3, 5),
    };

    let for_stmt = Statement::For {
        variable: "x".to_string(),
        iterable: Expression::Identifier {
            name: "items".to_string(),
            position: pos(2, 12),
        },
        body: vec![match_stmt],
        position: pos(2, 3),
    };

    let stmt = Statement::If {
        condition: Expression::Identifier {
            name: "condition1".to_string(),
            position: pos(1, 4),
        },
        then_branch: vec![for_stmt],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}
