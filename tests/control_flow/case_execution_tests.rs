use metorex::ast::{Expression, MatchCase, MatchPattern, Statement};
use metorex::lexer::Position;
use metorex::vm::VirtualMachine;

fn pos() -> Position {
    Position::new(1, 1, 0)
}

#[test]
fn test_case_integer_literal_match() {
    let mut vm = VirtualMachine::new();

    // case 42 when 42 => "matched" end
    let case_stmt = Statement::Match {
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

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_string_literal_match() {
    let mut vm = VirtualMachine::new();

    // case "hello" when "hello" => "matched" end
    let case_stmt = Statement::Match {
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

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_wildcard_match() {
    let mut vm = VirtualMachine::new();

    // case 999 when 1 => "one" when _ => "other" end
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 999,
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
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "other".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_multiple_when_clauses() {
    let mut vm = VirtualMachine::new();

    // case 2 when 0 => "zero" when 1 => "one" when 2 => "two" end
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 2,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(0),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "zero".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
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

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_type_pattern_integer() {
    let mut vm = VirtualMachine::new();

    // case 42 when Integer => "matched" end
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Type("Integer".to_string()),
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

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_type_pattern_string() {
    let mut vm = VirtualMachine::new();

    // case "hello" when String => "matched" end
    let case_stmt = Statement::Match {
        expression: Expression::StringLiteral {
            value: "hello".to_string(),
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Type("String".to_string()),
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

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_type_pattern_array() {
    let mut vm = VirtualMachine::new();

    // case [1, 2, 3] when Array => "matched" end
    let case_stmt = Statement::Match {
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
            pattern: MatchPattern::Type("Array".to_string()),
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

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_type_pattern_hash() {
    let mut vm = VirtualMachine::new();

    // case {"key" => "value"} when Hash => "matched" end
    let case_stmt = Statement::Match {
        expression: Expression::Dictionary {
            entries: vec![(
                Expression::StringLiteral {
                    value: "key".to_string(),
                    position: pos(),
                },
                Expression::StringLiteral {
                    value: "value".to_string(),
                    position: pos(),
                },
            )],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Type("Hash".to_string()),
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

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}

#[test]
fn test_case_type_pattern_multiple() {
    let mut vm = VirtualMachine::new();

    // case 3.14 when Integer => "int" when Float => "float" end
    let case_stmt = Statement::Match {
        expression: Expression::FloatLiteral {
            value: 3.14,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::Type("Integer".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "int".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Type("Float".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "float".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };

    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_ok());
}
