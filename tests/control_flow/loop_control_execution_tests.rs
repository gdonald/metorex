// Unit tests for loop-control execution in the VM

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for Break statement

#[test]
fn test_break_in_while_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        // x = 0
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        // while true
        Statement::While {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(2, 7),
            },
            body: vec![
                // x = x + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
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
                // if x >= 5: break
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::GreaterEqual,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(4, 6),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 5,
                            position: pos(4, 11),
                        }),
                        position: pos(4, 8),
                    },
                    then_branch: vec![Statement::Break {
                        value: None,
                        position: pos(5, 5),
                    }],
                    elsif_branches: vec![],
                    else_branch: None,
                    position: pos(4, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();
    assert_eq!(vm.environment().get("x"), Some(Object::Int(5)));
}

#[test]
fn test_continue_in_while_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        // x = 0
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        // count = 0
        Statement::Assignment {
            target: Expression::Identifier {
                name: "count".to_string(),
                position: pos(2, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(2, 9),
            },
            position: pos(2, 1),
        },
        // while x < 10
        Statement::While {
            condition: Expression::BinaryOp {
                op: BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(3, 7),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 10,
                    position: pos(3, 11),
                }),
                position: pos(3, 9),
            },
            body: vec![
                // x = x + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(4, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(4, 11),
                        }),
                        position: pos(4, 9),
                    },
                    position: pos(4, 3),
                },
                // if x % 2 == 0: continue
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: pos(5, 6),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 2,
                                position: pos(5, 10),
                            }),
                            position: pos(5, 8),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 0,
                            position: pos(5, 15),
                        }),
                        position: pos(5, 12),
                    },
                    then_branch: vec![Statement::Continue {
                        value: None,
                        position: pos(6, 5),
                    }],
                    elsif_branches: vec![],
                    else_branch: None,
                    position: pos(5, 3),
                },
                // count = count + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "count".to_string(),
                        position: pos(7, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "count".to_string(),
                            position: pos(7, 11),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(7, 19),
                        }),
                        position: pos(7, 17),
                    },
                    position: pos(7, 3),
                },
            ],
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();
    // count should only increment on odd numbers (1,3,5,7,9) = 5 times
    assert_eq!(vm.environment().get("count"), Some(Object::Int(5)));
}

// ── Kernel#loop ──────────────────────────────────────────────────────────────

use metorex::lexer::Lexer;
use metorex::parser::Parser;

fn run_source(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_source_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

#[test]
fn kernel_loop_runs_until_break() {
    let result = run_source("count = 0\nloop do\n  count += 1\n  break if count == 10\nend\ncount");
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn kernel_loop_returns_the_break_value() {
    let result = run_source("loop { break 123 }");
    assert_eq!(result, Some(Object::Int(123)));
}

#[test]
fn kernel_loop_returns_nil_for_a_bare_break() {
    let result = run_source("loop { break }");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn kernel_loop_ends_on_stop_iteration() {
    let result = run_source(
        "tries = 0\nloop do\n  tries += 1\n  raise StopIteration if tries == 3\nend\ntries",
    );
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn kernel_loop_ends_on_a_stop_iteration_subclass() {
    let result = run_source(
        "class Finished < StopIteration\nend\nreached = 0\nloop do\n  reached += 1\n  raise Finished\nend\nreached",
    );
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn kernel_loop_ends_on_an_anonymous_stop_iteration_subclass() {
    let result = run_source(
        "finish = Class.new StopIteration\nreached = 0\nloop do\n  reached += 1\n  raise finish\nend\nreached",
    );
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn kernel_loop_does_not_swallow_other_errors() {
    let error = run_source_err(r#"loop { raise ArgumentError, "escapes" }"#);
    assert!(error.contains("escapes"));
}

#[test]
fn kernel_loop_rejects_arguments() {
    let error = run_source_err("loop(1) { break }");
    assert!(error.contains("loop() expects 0 arguments, got 1"));
}

#[test]
fn kernel_loop_is_a_private_instance_method_on_kernel() {
    let result = run_source("Kernel.private_instance_methods(false).include?(:loop)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn an_anonymous_exception_subclass_is_rescued_by_its_named_ancestor() {
    let result = run_source(
        r#"
finish = Class.new StopIteration
begin
  raise finish
rescue StopIteration => error
  error.class.equal? finish
end
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn an_anonymous_exception_subclass_has_no_class_name() {
    let result = run_source(
        r#"
finish = Class.new StopIteration
begin
  raise finish
rescue StopIteration => error
  error.class.name
end
"#,
    );
    assert_eq!(result, Some(Object::Nil));
}

// ── throw arity and tap without a block ──────────────────────────────────────

#[test]
fn throw_with_too_many_arguments_raises_argument_error() {
    let error = run_source_err("throw(:one, :two, :three)");
    assert!(error.contains("wrong number of arguments (given 3, expected 1..2)"));
}

#[test]
fn a_bare_throw_raises_argument_error() {
    let error = run_source_err("throw");
    assert!(error.contains("wrong number of arguments (given 0, expected 1..2)"));
}

#[test]
fn throw_without_a_catch_raises_uncaught_throw_error() {
    let error = run_source_err("throw(:nothing_catches_this)");
    assert!(error.contains("UncaughtThrowError") || error.contains("uncaught throw"));
}

#[test]
fn catch_answers_the_thrown_value() {
    let result = run_source(
        r#"
catch(:done) do
  10.times do |i|
    throw(:done, i) if i == 3
  end
  :never
end
"#,
    );
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn tap_answers_the_receiver() {
    let result = run_source(
        r#"
class Widget
end
widget = Widget.new
widget.tap { :ignored }.equal?(widget)
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn tap_without_a_block_raises_local_jump_error() {
    let error = run_source_err("3.tap");
    assert!(error.contains("no block given (yield)"));
}

// ── taint was removed from Ruby ──────────────────────────────────────────────

#[test]
fn taint_and_tainted_are_absent() {
    let result =
        run_source("[Object.new.respond_to?(:taint), Object.new.respond_to?(:tainted?)].inspect");
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("[false, false]".to_string())
    );
}
