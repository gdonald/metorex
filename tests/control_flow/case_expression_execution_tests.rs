use metorex::ast::node::ExprMatchCase;
use metorex::ast::{BinaryOp, Expression, MatchPattern, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

fn pos() -> Position {
    Position::new(1, 1, 0)
}

// Helper to execute a case expression and get the result
fn execute_case_expr(vm: &mut VirtualMachine, case_expr: Expression) -> Object {
    // Wrap in assignment so we can test the result
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

    // Get the result variable
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
fn test_case_expression_array_destructuring() {
    let mut vm = VirtualMachine::new();

    // result = case [1, 2, 3] when [a, b, c] then a + b + c end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::Array {
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
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("a".to_string()),
                MatchPattern::Identifier("b".to_string()),
                MatchPattern::Identifier("c".to_string()),
            ]),
            guard: None,
            body: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::BinaryOp {
                    op: BinaryOp::Add,
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
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "6");
}

#[test]
fn test_case_expression_array_rest_pattern() {
    let mut vm = VirtualMachine::new();

    // result = case [10, 20, 30, 40] when [first, ...rest] then first end
    let case_expr = Expression::Case {
        expression: Box::new(Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 10,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 20,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 30,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 40,
                    position: pos(),
                },
            ],
            position: pos(),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("first".to_string()),
                MatchPattern::Rest("rest".to_string()),
            ]),
            guard: None,
            body: Expression::Identifier {
                name: "first".to_string(),
                position: pos(),
            },
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };

    let result = execute_case_expr(&mut vm, case_expr);
    assert_eq!(result.to_string(), "10");
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

    let assign_stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "result".to_string(),
            position: pos(),
        },
        value: arithmetic,
        position: pos(),
    };

    vm.execute_program(&[assign_stmt])
        .expect("Program execution failed");
    let result = vm.environment().get("result").expect("result not found");
    assert_eq!(result.to_string(), "25");
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
