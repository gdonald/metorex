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
fn test_case_expression_nested() {
    let mut vm = VirtualMachine::new();

    // result = case 1 when 1 then case 2 when 2 then "both match" end else "outer no match" end
    let inner_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 2,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(2),
            guard: None,
            body: Expression::StringLiteral {
                value: "both match".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let outer_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(1),
            guard: None,
            body: inner_case,
            position: pos(),
        }],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "outer no match".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, outer_case);
    assert_eq!(result.to_string(), "both match");
}

#[test]
fn test_case_expression_nested_with_different_values() {
    let mut vm = VirtualMachine::new();

    // Outer: case 1 when 1 then (inner case)
    // Inner: case "a" when "a" then "nested match" when "b" then "b match" else "no match" end
    let inner_case = Expression::Case {
        expression: Box::new(Expression::StringLiteral {
            value: "a".to_string(),
            position: pos(),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::StringLiteral("a".to_string()),
                guard: None,
                body: Expression::StringLiteral {
                    value: "nested match".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
            ExprMatchCase {
                pattern: MatchPattern::StringLiteral("b".to_string()),
                guard: None,
                body: Expression::StringLiteral {
                    value: "b match".to_string(),
                    position: pos(),
                },
                position: pos(),
            },
        ],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "no match".to_string(),
            position: pos(),
        })),
        position: pos(),
    };

    let outer_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(1),
            guard: None,
            body: inner_case,
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, outer_case);
    assert_eq!(result.to_string(), "nested match");
}

#[test]
fn test_case_expression_nested_scope_isolation() {
    let mut vm = VirtualMachine::new();

    // Outer binds 'x', inner also binds 'x' (should be isolated)
    // result = case 10 when x then (case 20 when x then x end) end
    let inner_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 20,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: None,
            body: Expression::Identifier {
                name: "x".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let outer_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 10,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: None,
            body: inner_case,
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, outer_case);
    // Inner case should return 20 (its own binding), not 10 (outer binding)
    assert_eq!(result.to_string(), "20");
}

#[test]
fn test_case_expression_nested_with_outer_variable_reference() {
    let mut vm = VirtualMachine::new();

    // Set up an outer variable
    let setup_stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "multiplier".to_string(),
            position: pos(),
        },
        value: Expression::IntLiteral {
            value: 3,
            position: pos(),
        },
        position: pos(),
    };
    vm.execute_program(&[setup_stmt]).expect("Setup failed");

    // Outer case binds 'x', inner case body references both 'y' (its binding) and 'x' (outer case binding)
    // result = case 10 when x then (case 5 when y then x + y * multiplier end) end
    let inner_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 5,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Identifier("y".to_string()),
            guard: None,
            body: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(),
                }),
                right: Box::new(Expression::BinaryOp {
                    op: BinaryOp::Multiply,
                    left: Box::new(Expression::Identifier {
                        name: "y".to_string(),
                        position: pos(),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "multiplier".to_string(),
                        position: pos(),
                    }),
                    position: pos(),
                }),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let outer_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 10,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Identifier("x".to_string()),
            guard: None,
            body: inner_case,
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, outer_case);
    // x=10, y=5, multiplier=3 => 10 + (5 * 3) = 10 + 15 = 25
    assert_eq!(result.to_string(), "25");
}

#[test]
fn test_case_expression_deeply_nested() {
    let mut vm = VirtualMachine::new();

    // Three levels of nesting
    // innermost: case "c" when "c" then "deep" end
    let innermost_case = Expression::Case {
        expression: Box::new(Expression::StringLiteral {
            value: "c".to_string(),
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::StringLiteral("c".to_string()),
            guard: None,
            body: Expression::StringLiteral {
                value: "deep".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    // middle: case "b" when "b" then (innermost) end
    let middle_case = Expression::Case {
        expression: Box::new(Expression::StringLiteral {
            value: "b".to_string(),
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::StringLiteral("b".to_string()),
            guard: None,
            body: innermost_case,
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    // outer: case "a" when "a" then (middle) end
    let outer_case = Expression::Case {
        expression: Box::new(Expression::StringLiteral {
            value: "a".to_string(),
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::StringLiteral("a".to_string()),
            guard: None,
            body: middle_case,
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, outer_case);
    assert_eq!(result.to_string(), "deep");
}

#[test]
fn test_case_expression_nested_in_else_clause() {
    let mut vm = VirtualMachine::new();

    // Outer doesn't match, evaluates else clause which contains a nested case
    // result = case 99 when 1 then "matched" else (case "x" when "x" then "from else" end) end
    let inner_case = Expression::Case {
        expression: Box::new(Expression::StringLiteral {
            value: "x".to_string(),
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::StringLiteral("x".to_string()),
            guard: None,
            body: Expression::StringLiteral {
                value: "from else".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let outer_case = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 99,
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(1),
            guard: None,
            body: Expression::StringLiteral {
                value: "matched".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: Some(Box::new(inner_case)),
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, outer_case);
    assert_eq!(result.to_string(), "from else");
}
