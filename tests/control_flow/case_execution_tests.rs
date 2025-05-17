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
