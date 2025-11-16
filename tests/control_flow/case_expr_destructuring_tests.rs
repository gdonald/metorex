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
