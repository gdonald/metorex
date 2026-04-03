use metorex::ast::{Expression, Statement};
use metorex::error::MetorexError;
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

fn string_literal(value: &str, position: Position) -> Expression {
    Expression::StringLiteral {
        value: value.to_string(),
        position,
    }
}

fn int_literal(value: i64, position: Position) -> Expression {
    Expression::IntLiteral { value, position }
}

#[test]
fn calls_string_length_method() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "len".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(string_literal("hello", pos(1, 1))),
            method: "length".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 10),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(vm.environment().get("len"), Some(Object::Int(5)));
}

#[test]
fn string_upcase_returns_new_string() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "shout".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(string_literal("Metorex", pos(1, 1))),
            method: "upcase".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(
        vm.environment().get("shout"),
        Some(Object::String(Rc::new("METOREX".to_string())))
    );
}

#[test]
fn array_push_and_pop_updates_collection() {
    let mut vm = VirtualMachine::new();

    let setup = Statement::Assignment {
        target: Expression::Identifier {
            name: "items".to_string(),
            position: pos(1, 1),
        },
        value: Expression::Array {
            elements: vec![int_literal(1, pos(1, 9)), int_literal(2, pos(1, 12))],
            position: pos(1, 9),
        },
        position: pos(1, 1),
    };

    let push_call = Statement::Expression {
        expression: Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "items".to_string(),
                position: pos(2, 1),
            }),
            method: "push".to_string(),
            arguments: vec![int_literal(3, pos(2, 13))],
            trailing_block: None,
            position: pos(2, 7),
        },
        position: pos(2, 1),
    };

    let pop_assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "last".to_string(),
            position: pos(3, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "items".to_string(),
                position: pos(3, 10),
            }),
            method: "pop".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(3, 16),
        },
        position: pos(3, 1),
    };

    vm.execute_program(&[setup, push_call, pop_assign])
        .expect("execution failed");

    match vm.environment().get("items") {
        Some(Object::Array(array_rc)) => {
            let array = array_rc.borrow();
            assert_eq!(array.len(), 2);
            assert_eq!(array[0], Object::Int(1));
            assert_eq!(array[1], Object::Int(2));
        }
        other => panic!("expected array, got {:?}", other),
    }

    assert_eq!(vm.environment().get("last"), Some(Object::Int(3)));
}

#[test]
fn array_append_adds_element() {
    let mut vm = VirtualMachine::new();

    let setup = Statement::Assignment {
        target: Expression::Identifier {
            name: "items".to_string(),
            position: pos(1, 1),
        },
        value: Expression::Array {
            elements: vec![int_literal(1, pos(1, 9)), int_literal(2, pos(1, 12))],
            position: pos(1, 9),
        },
        position: pos(1, 1),
    };

    let append_call = Statement::Expression {
        expression: Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "items".to_string(),
                position: pos(2, 1),
            }),
            method: "append".to_string(),
            arguments: vec![int_literal(3, pos(2, 15))],
            trailing_block: None,
            position: pos(2, 7),
        },
        position: pos(2, 1),
    };

    vm.execute_program(&[setup, append_call])
        .expect("execution failed");

    match vm.environment().get("items") {
        Some(Object::Array(array_rc)) => {
            let array = array_rc.borrow();
            assert_eq!(array.len(), 3);
            assert_eq!(array[0], Object::Int(1));
            assert_eq!(array[1], Object::Int(2));
            assert_eq!(array[2], Object::Int(3));
        }
        other => panic!("expected array, got {:?}", other),
    }
}

#[test]
fn undefined_method_returns_runtime_error() {
    let mut vm = VirtualMachine::new();

    let statements = vec![Statement::Expression {
        expression: Expression::MethodCall {
            receiver: Box::new(string_literal("oops", pos(1, 1))),
            method: "unknown".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 10),
        },
        position: pos(1, 1),
    }];

    let result = vm.execute_program(&statements);
    assert!(matches!(result, Err(MetorexError::RuntimeError { .. })));
}

#[test]
fn string_trim_removes_whitespace() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "trimmed".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(string_literal("  hello  ", pos(1, 1))),
            method: "trim".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(
        vm.environment().get("trimmed"),
        Some(Object::String(Rc::new("hello".to_string())))
    );
}

#[test]
fn string_reverse_reverses_characters() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "reversed".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(string_literal("abc", pos(1, 1))),
            method: "reverse".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(
        vm.environment().get("reversed"),
        Some(Object::String(Rc::new("cba".to_string())))
    );
}

#[test]
fn string_chars_returns_array_of_characters() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "chars".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(string_literal("Hi", pos(1, 1))),
            method: "chars".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");

    match vm.environment().get("chars") {
        Some(Object::Array(array_rc)) => {
            let array = array_rc.borrow();
            assert_eq!(array.len(), 2);
            assert_eq!(array[0], Object::String(Rc::new("H".to_string())));
            assert_eq!(array[1], Object::String(Rc::new("i".to_string())));
        }
        other => panic!("expected array, got {:?}", other),
    }
}

#[test]
fn string_bytes_returns_array_of_byte_values() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "bytes".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(string_literal("AB", pos(1, 1))),
            method: "bytes".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");

    match vm.environment().get("bytes") {
        Some(Object::Array(array_rc)) => {
            let array = array_rc.borrow();
            assert_eq!(array.len(), 2);
            assert_eq!(array[0], Object::Int(65)); // 'A'
            assert_eq!(array[1], Object::Int(66)); // 'B'
        }
        other => panic!("expected array, got {:?}", other),
    }
}

#[test]
fn string_reverse_handles_unicode() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "reversed".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(string_literal("こんにちは", pos(1, 1))),
            method: "reverse".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(
        vm.environment().get("reversed"),
        Some(Object::String(Rc::new("はちにんこ".to_string())))
    );
}

fn float_literal(value: f64, position: Position) -> Expression {
    Expression::FloatLiteral { value, position }
}

#[test]
fn float_round_rounds_to_precision() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "rounded".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(float_literal(69.85, pos(1, 1))),
            method: "round".to_string(),
            arguments: vec![int_literal(1, pos(1, 20))],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(vm.environment().get("rounded"), Some(Object::Float(69.9)));
}

#[test]
fn float_round_with_zero_precision() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "rounded".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(float_literal(3.7, pos(1, 1))),
            method: "round".to_string(),
            arguments: vec![int_literal(0, pos(1, 20))],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(vm.environment().get("rounded"), Some(Object::Float(4.0)));
}

#[test]
fn float_round_with_two_decimals() {
    let mut vm = VirtualMachine::new();

    let assign = Statement::Assignment {
        target: Expression::Identifier {
            name: "rounded".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(float_literal(3.14159, pos(1, 1))),
            method: "round".to_string(),
            arguments: vec![int_literal(2, pos(1, 20))],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assign]).expect("execution failed");
    assert_eq!(vm.environment().get("rounded"), Some(Object::Float(3.14)));
}

#[test]
fn float_round_with_negative_precision_fails() {
    let mut vm = VirtualMachine::new();

    let statements = vec![Statement::Assignment {
        target: Expression::Identifier {
            name: "rounded".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(float_literal(3.7, pos(1, 1))),
            method: "round".to_string(),
            arguments: vec![int_literal(-1, pos(1, 20))],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    }];

    let result = vm.execute_program(&statements);
    assert!(matches!(result, Err(MetorexError::RuntimeError { .. })));
}

#[test]
fn float_round_with_non_integer_precision_fails() {
    let mut vm = VirtualMachine::new();

    let statements = vec![Statement::Assignment {
        target: Expression::Identifier {
            name: "rounded".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(float_literal(3.7, pos(1, 1))),
            method: "round".to_string(),
            arguments: vec![string_literal("hello", pos(1, 20))],
            trailing_block: None,
            position: pos(1, 15),
        },
        position: pos(1, 1),
    }];

    let result = vm.execute_program(&statements);
    assert!(matches!(result, Err(MetorexError::TypeError { .. })));
}
