use metorex::ast::node::ExprMatchCase;
use metorex::ast::{Expression, MatchPattern, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

fn pos() -> Position {
    Position::new(1, 1, 0)
}

fn execute_case_expr(vm: &mut VirtualMachine, case_expr: Expression) -> Object {
    let assign_stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "result".to_string(),
            position: pos(),
        },
        value: case_expr,
        position: pos(),
    };
    vm.execute_program(&[assign_stmt])
        .expect("Program execution failed");
    vm.environment().get("result").expect("result not found")
}

#[test]
fn test_case_expression_multiple_patterns_match_first() {
    let mut vm = VirtualMachine::new();

    // result = case 1 when 1, 2, 3 then "small" else "large" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Multiple(vec![
                MatchPattern::IntLiteral(1),
                MatchPattern::IntLiteral(2),
                MatchPattern::IntLiteral(3),
            ]),
            guard: None,
            body: Expression::StringLiteral {
                value: "small".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "large".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "small");
}

#[test]
fn test_case_expression_multiple_patterns_match_middle() {
    let mut vm = VirtualMachine::new();

    // result = case 2 when 1, 2, 3 then "small" else "large" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 2,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Multiple(vec![
                MatchPattern::IntLiteral(1),
                MatchPattern::IntLiteral(2),
                MatchPattern::IntLiteral(3),
            ]),
            guard: None,
            body: Expression::StringLiteral {
                value: "small".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "large".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "small");
}

#[test]
fn test_case_expression_multiple_patterns_match_last() {
    let mut vm = VirtualMachine::new();

    // result = case 3 when 1, 2, 3 then "small" else "large" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 3,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Multiple(vec![
                MatchPattern::IntLiteral(1),
                MatchPattern::IntLiteral(2),
                MatchPattern::IntLiteral(3),
            ]),
            guard: None,
            body: Expression::StringLiteral {
                value: "small".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "large".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "small");
}

#[test]
fn test_case_expression_multiple_patterns_no_match() {
    let mut vm = VirtualMachine::new();

    // result = case 5 when 1, 2, 3 then "small" else "large" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 5,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Multiple(vec![
                MatchPattern::IntLiteral(1),
                MatchPattern::IntLiteral(2),
                MatchPattern::IntLiteral(3),
            ]),
            guard: None,
            body: Expression::StringLiteral {
                value: "small".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "large".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "large");
}

#[test]
fn test_case_expression_multiple_patterns_with_strings() {
    let mut vm = VirtualMachine::new();

    // result = case "b" when "a", "b", "c" then "vowel-ish" else "other" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::StringLiteral {
            value: "b".to_string(),
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Multiple(vec![
                MatchPattern::StringLiteral("a".to_string()),
                MatchPattern::StringLiteral("b".to_string()),
                MatchPattern::StringLiteral("c".to_string()),
            ]),
            guard: None,
            body: Expression::StringLiteral {
                value: "vowel-ish".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "other".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "vowel-ish");
}

#[test]
fn test_case_expression_multiple_when_clauses_with_multiple_patterns() {
    let mut vm = VirtualMachine::new();

    // result = case 4 when 1, 2, 3 then "small" when 4, 5 then "medium" else "large" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 4,
            position: pos(),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::Multiple(vec![
                    MatchPattern::IntLiteral(1),
                    MatchPattern::IntLiteral(2),
                    MatchPattern::IntLiteral(3),
                ]),
                guard: None,
                body: Expression::StringLiteral {
                    value: "small".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
            ExprMatchCase {
                pattern: MatchPattern::Multiple(vec![
                    MatchPattern::IntLiteral(4),
                    MatchPattern::IntLiteral(5),
                ]),
                guard: None,
                body: Expression::StringLiteral {
                    value: "medium".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
        ],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "large".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "medium");
}
