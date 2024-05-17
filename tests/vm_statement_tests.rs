use metorex::ast::{Expression, Statement};
use metorex::error::MetorexError;
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

fn int_literal(value: i64, position: Position) -> Expression {
    Expression::IntLiteral { value, position }
}

#[test]
fn expression_statement_executes_without_error() {
    let mut vm = VirtualMachine::new();

    let expression_position = pos(1, 1);
    let stmt_position = pos(1, 1);

    let statements = vec![Statement::Expression {
        expression: int_literal(5, expression_position),
        position: stmt_position,
    }];

    let result = vm.execute_program(&statements);
    assert!(matches!(result, Ok(None)));
}

#[test]
fn assignment_creates_variable_in_environment() {
    let mut vm = VirtualMachine::new();
    let stmt_position = pos(1, 1);
    let target_position = pos(1, 1);
    let value_position = pos(1, 5);

    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "x".to_string(),
            position: target_position,
        },
        value: int_literal(42, value_position),
        position: stmt_position,
    };

    let result = vm.execute_program(&[assignment]);
    assert!(matches!(result, Ok(None)));
    assert_eq!(vm.environment().get("x"), Some(Object::Int(42)));
}

#[test]
fn assignment_updates_existing_variable() {
    let mut vm = VirtualMachine::new();
    vm.environment_mut()
        .define("counter".to_string(), Object::Int(1));

    let stmt_position = pos(1, 1);
    let target_position = pos(1, 1);
    let value_position = pos(1, 10);

    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "counter".to_string(),
            position: target_position,
        },
        value: int_literal(2, value_position),
        position: stmt_position,
    };

    let result = vm.execute_program(&[assignment]);
    assert!(matches!(result, Ok(None)));
    assert_eq!(vm.environment().get("counter"), Some(Object::Int(2)));
}

#[test]
fn return_statement_halts_execution_and_returns_value() {
    let mut vm = VirtualMachine::new();

    let return_position = pos(1, 1);
    let value_position = pos(1, 8);
    let assignment_position = pos(2, 1);
    let assignment_target_position = pos(2, 1);
    let assignment_value_position = pos(2, 10);

    let statements = vec![
        Statement::Return {
            value: Some(int_literal(99, value_position)),
            position: return_position,
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "after".to_string(),
                position: assignment_target_position,
            },
            value: int_literal(123, assignment_value_position),
            position: assignment_position,
        },
    ];

    let result = vm.execute_program(&statements);
    assert!(matches!(result, Ok(Some(Object::Int(99)))));
    assert!(vm.environment().get("after").is_none());
}

#[test]
fn return_without_value_yields_nil() {
    let mut vm = VirtualMachine::new();

    let return_position = pos(1, 1);
    let statements = vec![Statement::Return {
        value: None,
        position: return_position,
    }];

    let result = vm.execute_program(&statements);
    assert!(matches!(result, Ok(Some(Object::Nil))));
}

#[test]
fn break_outside_loop_produces_runtime_error() {
    let mut vm = VirtualMachine::new();
    let break_stmt = Statement::Break {
        position: pos(1, 1),
    };

    match vm.execute_program(&[break_stmt]) {
        Err(MetorexError::RuntimeError { message, .. }) => {
            assert!(message.contains("break"), "unexpected message: {}", message);
        }
        other => panic!("expected runtime error, got {:?}", other),
    }
}

#[test]
fn continue_outside_loop_produces_runtime_error() {
    let mut vm = VirtualMachine::new();
    let continue_stmt = Statement::Continue {
        position: pos(1, 1),
    };

    match vm.execute_program(&[continue_stmt]) {
        Err(MetorexError::RuntimeError { message, .. }) => {
            assert!(
                message.contains("continue"),
                "unexpected message: {}",
                message
            );
        }
        other => panic!("expected runtime error, got {:?}", other),
    }
}
