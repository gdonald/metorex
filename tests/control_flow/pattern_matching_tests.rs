// Unit tests for pattern matching AST nodes

use metorex::ast::{Expression, MatchCase, MatchPattern, Statement};
use metorex::lexer::Position;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for basic literal patterns

#[test]
fn test_match_int_literal_pattern() {
    let pattern = MatchPattern::IntLiteral(42);
    let case = MatchCase {
        pattern,
        guard: None,
        body: vec![Statement::Expression {
            expression: Expression::StringLiteral {
                value: "matched".to_string(),
                position: pos(1, 10),
            },
            position: pos(1, 10),
        }],
        position: pos(1, 1),
    };

    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 7),
        },
        cases: vec![case],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_string_literal_pattern() {
    let pattern = MatchPattern::StringLiteral("hello".to_string());
    let case = MatchCase {
        pattern,
        guard: None,
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(1, 15),
            },
            position: pos(1, 15),
        }],
        position: pos(1, 1),
    };

    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "s".to_string(),
            position: pos(1, 7),
        },
        cases: vec![case],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_bool_literal_patterns() {
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
                        position: pos(2, 12),
                    },
                    position: pos(2, 12),
                }],
                position: pos(2, 1),
            },
            MatchCase {
                pattern: MatchPattern::BoolLiteral(false),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 13),
                    },
                    position: pos(3, 13),
                }],
                position: pos(3, 1),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_nil_literal_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "value".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::NilLiteral,
            guard: None,
            body: vec![Statement::Return {
                value: Some(Expression::IntLiteral {
                    value: 0,
                    position: pos(2, 17),
                }),
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_float_literal_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::FloatLiteral(3.14),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "pi".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for identifier patterns (variable binding)

#[test]
fn test_match_identifier_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "value".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for wildcard pattern

#[test]
fn test_match_wildcard_pattern() {
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
                        position: pos(2, 10),
                    },
                    position: pos(2, 10),
                }],
                position: pos(2, 1),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "other".to_string(),
                        position: pos(3, 10),
                    },
                    position: pos(3, 10),
                }],
                position: pos(3, 1),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for array patterns

#[test]
fn test_match_array_pattern_empty() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "empty".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_array_pattern_with_literals() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 7),
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
                    value: "matched [1, 2, 3]".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_array_pattern_with_variables() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("first".to_string()),
                MatchPattern::Identifier("second".to_string()),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "first".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_array_pattern_with_wildcard() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("first".to_string()),
                MatchPattern::Wildcard,
                MatchPattern::Identifier("third".to_string()),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "first".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_nested_array_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "matrix".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Array(vec![
                    MatchPattern::IntLiteral(1),
                    MatchPattern::IntLiteral(2),
                ]),
                MatchPattern::Array(vec![
                    MatchPattern::IntLiteral(3),
                    MatchPattern::IntLiteral(4),
                ]),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "2x2 matrix".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for rest patterns

#[test]
fn test_match_array_with_rest_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 7),
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
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_array_rest_at_end() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("a".to_string()),
                MatchPattern::Identifier("b".to_string()),
                MatchPattern::Rest("remaining".to_string()),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "remaining".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for object/dictionary patterns

#[test]
fn test_match_object_pattern_empty() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "obj".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Object(vec![]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "empty object".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_object_pattern_simple() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "point".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Object(vec![
                ("x".to_string(), MatchPattern::Identifier("x".to_string())),
                ("y".to_string(), MatchPattern::Identifier("y".to_string())),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_object_pattern_with_literals() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "config".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Object(vec![
                (
                    "type".to_string(),
                    MatchPattern::StringLiteral("production".to_string()),
                ),
                ("debug".to_string(), MatchPattern::BoolLiteral(false)),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "production mode".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_object_pattern_with_nested_patterns() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "data".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Object(vec![
                (
                    "user".to_string(),
                    MatchPattern::Object(vec![
                        (
                            "name".to_string(),
                            MatchPattern::Identifier("name".to_string()),
                        ),
                        (
                            "age".to_string(),
                            MatchPattern::Identifier("age".to_string()),
                        ),
                    ]),
                ),
                (
                    "status".to_string(),
                    MatchPattern::Identifier("status".to_string()),
                ),
            ]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "name".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for type patterns

#[test]
fn test_match_type_pattern() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "value".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::Type("String".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "is string".to_string(),
                        position: pos(2, 10),
                    },
                    position: pos(2, 10),
                }],
                position: pos(2, 1),
            },
            MatchCase {
                pattern: MatchPattern::Type("Integer".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "is integer".to_string(),
                        position: pos(3, 10),
                    },
                    position: pos(3, 10),
                }],
                position: pos(3, 1),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for guards

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
                op: metorex::ast::BinaryOp::Greater,
                left: Box::new(Expression::Identifier {
                    name: "n".to_string(),
                    position: pos(2, 10),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 0,
                    position: pos(2, 14),
                }),
                position: pos(2, 12),
            }),
            body: vec![Statement::Expression {
                expression: Expression::StringLiteral {
                    value: "positive".to_string(),
                    position: pos(2, 20),
                },
                position: pos(2, 20),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_multiple_cases_with_guards() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::Identifier("n".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Greater,
                    left: Box::new(Expression::Identifier {
                        name: "n".to_string(),
                        position: pos(2, 10),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 100,
                        position: pos(2, 14),
                    }),
                    position: pos(2, 12),
                }),
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "large".to_string(),
                        position: pos(2, 20),
                    },
                    position: pos(2, 20),
                }],
                position: pos(2, 1),
            },
            MatchCase {
                pattern: MatchPattern::Identifier("n".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Greater,
                    left: Box::new(Expression::Identifier {
                        name: "n".to_string(),
                        position: pos(3, 10),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 14),
                    }),
                    position: pos(3, 12),
                }),
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "small".to_string(),
                        position: pos(3, 20),
                    },
                    position: pos(3, 20),
                }],
                position: pos(3, 1),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for complex pattern matching scenarios

#[test]
fn test_match_array_with_object_elements() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "data".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![MatchPattern::Object(vec![
                ("id".to_string(), MatchPattern::Identifier("id".to_string())),
                (
                    "name".to_string(),
                    MatchPattern::Identifier("name".to_string()),
                ),
            ])]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "id".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_object_with_array_values() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "data".to_string(),
            position: pos(1, 7),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Object(vec![(
                "items".to_string(),
                MatchPattern::Array(vec![
                    MatchPattern::Identifier("first".to_string()),
                    MatchPattern::Rest("rest".to_string()),
                ]),
            )]),
            guard: None,
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "first".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 10),
            }],
            position: pos(2, 1),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_match_mixed_patterns_comprehensive() {
    let stmt = Statement::Match {
        expression: Expression::Identifier {
            name: "value".to_string(),
            position: pos(1, 7),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(0),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "zero".to_string(),
                        position: pos(2, 10),
                    },
                    position: pos(2, 10),
                }],
                position: pos(2, 1),
            },
            MatchCase {
                pattern: MatchPattern::Array(vec![]),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "empty array".to_string(),
                        position: pos(3, 10),
                    },
                    position: pos(3, 10),
                }],
                position: pos(3, 1),
            },
            MatchCase {
                pattern: MatchPattern::Object(vec![]),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "empty object".to_string(),
                        position: pos(4, 10),
                    },
                    position: pos(4, 10),
                }],
                position: pos(4, 1),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "anything else".to_string(),
                        position: pos(5, 10),
                    },
                    position: pos(5, 10),
                }],
                position: pos(5, 1),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}
