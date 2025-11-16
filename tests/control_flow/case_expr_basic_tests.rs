use metorex::ast::node::ExprMatchCase;
use metorex::ast::{BinaryOp, Expression, MatchPattern, Statement};
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
fn test_case_expression_literal_match() {
    let mut vm = VirtualMachine::new();

    // result = case 2 when 1 then "one" when 2 then "two" else "other" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 2,
            position: pos(),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: Expression::StringLiteral {
                    value: "one".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(2),
                guard: None,
                body: Expression::StringLiteral {
                    value: "two".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
        ],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "other".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "two");
}

#[test]
fn test_case_expression_else_clause() {
    let mut vm = VirtualMachine::new();

    // result = case 99 when 1 then "one" when 2 then "two" else "other" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 99,
            position: pos(),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: Expression::StringLiteral {
                    value: "one".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(2),
                guard: None,
                body: Expression::StringLiteral {
                    value: "two".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
        ],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "other".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "other");
}

#[test]
fn test_case_expression_no_match_returns_nil() {
    let mut vm = VirtualMachine::new();

    // result = case 99 when 1 then "one" when 2 then "two" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 99,
            position: pos(),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: Expression::StringLiteral {
                    value: "one".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(2),
                guard: None,
                body: Expression::StringLiteral {
                    value: "two".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
        ],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert!(matches!(result, Object::Nil));
}

#[test]
fn test_case_expression_variable_binding() {
    let mut vm = VirtualMachine::new();

    // result = case 42 when x then x * 2 end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 42,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: None,
            body: Expression::BinaryOp {
                op: BinaryOp::Multiply,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 2,
                    position: pos(),
                }),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "84");
}

#[test]
fn test_case_expression_type_pattern() {
    let mut vm = VirtualMachine::new();

    // result = case 42 when Integer then "it's an int" else "not an int" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 42,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Type("Integer".to_string()),
            guard: None,
            body: Expression::StringLiteral {
                value: "it's an int".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "not an int".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "it's an int");
}

#[test]
fn test_case_expression_wildcard_pattern() {
    let mut vm = VirtualMachine::new();

    // result = case "anything" when _ then "matched" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::StringLiteral {
            value: "anything".to_string(),
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Wildcard,
            guard: None,
            body: Expression::StringLiteral {
                value: "matched".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "matched");
}

#[test]
fn test_case_expression_in_arithmetic() {
    let mut vm = VirtualMachine::new();

    // result = 10 + (case 2 when 1 then 5 when 2 then 15 else 0 end)
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 2,
            position: pos(),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: Expression::IntLiteral {
                    value: 5,
                    position: pos(),
                },
                position: pos(),
            },
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(2),
                guard: None,
                body: Expression::IntLiteral {
                    value: 15,
                    position: pos(),
                },
                position: pos(),
            },
        ],
        else_case: Some(Box::new(Expression::IntLiteral {
            value: 0,
            position: pos(),
        })),
        position: pos(),
    };

    let arithmetic = Expression::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expression::IntLiteral {
            value: 10,
            position: pos(),
        }),
        right: Box::new(case_expr),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, arithmetic);
    assert_eq!(result.to_string(), "25");
}
