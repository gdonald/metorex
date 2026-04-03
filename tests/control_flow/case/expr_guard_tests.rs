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
fn test_case_expression_guard_clause() {
    let mut vm = VirtualMachine::new();

    // result = case 5 when x if x < 0 then "negative" when x if x > 0 then "positive" else "zero" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 5,
            position: pos(),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::Identifier("x".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: BinaryOp::Less,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 0,
                        position: pos(),
                    }),
                    position: pos(),
                }),
                body: Expression::StringLiteral {
                    value: "negative".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
            ExprMatchCase {
                pattern: MatchPattern::Identifier("x".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: BinaryOp::Greater,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 0,
                        position: pos(),
                    }),
                    position: pos(),
                }),
                body: Expression::StringLiteral {
                    value: "positive".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
        ],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "zero".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "positive");
}

#[test]
fn test_case_expression_guard_with_array_binding() {
    let mut vm = VirtualMachine::new();

    // result = case [3, 7] when [a, b] if a < b then "ascending" else "other" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 3,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 7,
                    position: pos(),
                },
            ],
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("a".to_string()),
                MatchPattern::Identifier("b".to_string()),
            ]),
            guard: Some(Expression::BinaryOp {
                op: BinaryOp::Less,
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
            body: Expression::StringLiteral {
                value: "ascending".to_string(),
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
    assert_eq!(result.to_string(), "ascending");
}

#[test]
fn test_case_expression_guard_references_outer_scope() {
    let mut vm = VirtualMachine::new();

    // Set up an outer variable
    let setup_stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "threshold".to_string(),
            position: pos(),
        },
        value: Expression::IntLiteral {
            value: 5,
            position: pos(),
        },
        position: pos(),
    };
    vm.execute_program(&[setup_stmt]).expect("Setup failed");

    // result = case 10 when x if x > threshold then "above" else "below" end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 10,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: Some(Expression::BinaryOp {
                op: BinaryOp::Greater,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                }),
                right: Box::new(Expression::Identifier {
                    name: "threshold".to_string(),
                    position: pos(),
                }),
                position: pos(),
            }),
            body: Expression::StringLiteral {
                value: "above".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "below".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "above");
}
