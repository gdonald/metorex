use metorex::ast::{BinaryOp, Expression, InterpolationPart, Statement, UnaryOp};
use metorex::error::MetorexError;
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

fn int_literal(value: i64, line: usize, column: usize) -> Expression {
    Expression::IntLiteral {
        value,
        position: pos(line, column),
    }
}

fn float_literal(value: f64, line: usize, column: usize) -> Expression {
    Expression::FloatLiteral {
        value,
        position: pos(line, column),
    }
}

fn string_literal(value: &str, line: usize, column: usize) -> Expression {
    Expression::StringLiteral {
        value: value.to_string(),
        position: pos(line, column),
    }
}

#[test]
fn evaluates_integer_addition() {
    let mut vm = VirtualMachine::new();
    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "result".to_string(),
            position: pos(1, 1),
        },
        value: Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(int_literal(40, 1, 1)),
            right: Box::new(int_literal(2, 1, 6)),
            position: pos(1, 5),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assignment]).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(42)));
}

#[test]
fn promotes_integer_and_float_in_binary_ops() {
    let mut vm = VirtualMachine::new();
    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "mixed".to_string(),
            position: pos(1, 1),
        },
        value: Expression::BinaryOp {
            op: BinaryOp::Divide,
            left: Box::new(int_literal(7, 1, 1)),
            right: Box::new(float_literal(2.0, 1, 6)),
            position: pos(1, 5),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assignment]).expect("execution failed");
    match vm.environment().get("mixed") {
        Some(Object::Float(result)) => assert!((result - 3.5).abs() < 1e-9),
        other => panic!("expected float result, got {:?}", other),
    }
}

#[test]
fn concatenates_strings_with_addition() {
    let mut vm = VirtualMachine::new();
    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "text".to_string(),
            position: pos(1, 1),
        },
        value: Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(string_literal("Hello", 1, 1)),
            right: Box::new(string_literal(", world", 1, 10)),
            position: pos(1, 6),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assignment]).expect("execution failed");
    assert_eq!(
        vm.environment().get("text"),
        Some(Object::String(Rc::new("Hello, world".to_string())))
    );
}

#[test]
fn evaluates_unary_minus() {
    let mut vm = VirtualMachine::new();
    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "value".to_string(),
            position: pos(1, 1),
        },
        value: Expression::UnaryOp {
            op: UnaryOp::Minus,
            operand: Box::new(int_literal(10, 1, 2)),
            position: pos(1, 1),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assignment]).expect("execution failed");
    assert_eq!(vm.environment().get("value"), Some(Object::Int(-10)));
}

#[test]
fn creates_array_literal_with_values() {
    let mut vm = VirtualMachine::new();
    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 1),
        },
        value: Expression::Array {
            elements: vec![
                int_literal(1, 1, 2),
                int_literal(2, 1, 5),
                Expression::BoolLiteral {
                    value: true,
                    position: pos(1, 8),
                },
            ],
            position: pos(1, 1),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assignment]).expect("execution failed");
    let array_obj = vm.environment().get("arr").expect("missing array");
    match array_obj {
        Object::Array(array_rc) => {
            let arr = array_rc.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Object::Int(1));
            assert_eq!(arr[1], Object::Int(2));
            assert_eq!(arr[2], Object::Bool(true));
        }
        other => panic!("expected array, got {:?}", other),
    }
}

#[test]
fn creates_dictionary_literal_with_entries() {
    let mut vm = VirtualMachine::new();
    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "dict".to_string(),
            position: pos(1, 1),
        },
        value: Expression::Dictionary {
            entries: vec![
                (
                    string_literal("name", 1, 3),
                    string_literal("Metorex", 1, 10),
                ),
                (string_literal("count", 2, 3), int_literal(3, 2, 12)),
            ],
            position: pos(1, 1),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assignment]).expect("execution failed");
    let dict_obj = vm.environment().get("dict").expect("missing dict");
    match dict_obj {
        Object::Dict(dict_rc) => {
            let dict = dict_rc.borrow();
            assert_eq!(
                dict.get("name"),
                Some(&Object::String(Rc::new("Metorex".to_string())))
            );
            assert_eq!(dict.get("count"), Some(&Object::Int(3)));
        }
        other => panic!("expected dictionary, got {:?}", other),
    }
}

#[test]
fn indexes_into_array_literal() {
    let mut vm = VirtualMachine::new();
    let statements = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "arr".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Array {
                elements: vec![int_literal(10, 1, 2), int_literal(20, 1, 6)],
                position: pos(1, 1),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "second".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Index {
                array: Box::new(Expression::Identifier {
                    name: "arr".to_string(),
                    position: pos(2, 1),
                }),
                index: Box::new(int_literal(1, 2, 6)),
                position: pos(2, 1),
            },
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&statements).expect("execution failed");
    assert_eq!(vm.environment().get("second"), Some(Object::Int(20)));
}

#[test]
fn indexes_into_dictionary_literal() {
    let mut vm = VirtualMachine::new();
    let statements = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "dict".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Dictionary {
                entries: vec![(string_literal("key", 1, 2), string_literal("value", 1, 9))],
                position: pos(1, 1),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Index {
                array: Box::new(Expression::Identifier {
                    name: "dict".to_string(),
                    position: pos(2, 1),
                }),
                index: Box::new(string_literal("key", 2, 8)),
                position: pos(2, 1),
            },
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        vm.environment().get("result"),
        Some(Object::String(Rc::new("value".to_string())))
    );
}

#[test]
fn array_out_of_bounds_produces_runtime_error() {
    let mut vm = VirtualMachine::new();
    let statements = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "arr".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Array {
                elements: vec![int_literal(1, 1, 2)],
                position: pos(1, 1),
            },
            position: pos(1, 1),
        },
        Statement::Expression {
            expression: Expression::Index {
                array: Box::new(Expression::Identifier {
                    name: "arr".to_string(),
                    position: pos(2, 1),
                }),
                index: Box::new(int_literal(5, 2, 6)),
                position: pos(2, 1),
            },
            position: pos(2, 1),
        },
    ];

    match vm.execute_program(&statements) {
        Err(MetorexError::RuntimeError { message, .. }) => {
            assert!(message.contains("out of bounds"), "unexpected {}", message);
        }
        other => panic!("expected runtime error, got {:?}", other),
    }
}

#[test]
fn invalid_binary_operands_raise_type_error() {
    let mut vm = VirtualMachine::new();
    let statements = vec![Statement::Expression {
        expression: Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(string_literal("hello", 1, 1)),
            right: Box::new(int_literal(5, 1, 10)),
            position: pos(1, 6),
        },
        position: pos(1, 1),
    }];

    match vm.execute_program(&statements) {
        Err(MetorexError::TypeError { message, .. }) => {
            assert!(message.contains("operator"), "unexpected {}", message);
        }
        other => panic!("expected type error, got {:?}", other),
    }
}

#[test]
fn division_by_zero_raises_runtime_error() {
    let mut vm = VirtualMachine::new();
    let statements = vec![Statement::Expression {
        expression: Expression::BinaryOp {
            op: BinaryOp::Divide,
            left: Box::new(int_literal(10, 1, 1)),
            right: Box::new(int_literal(0, 1, 6)),
            position: pos(1, 5),
        },
        position: pos(1, 1),
    }];

    match vm.execute_program(&statements) {
        Err(MetorexError::RuntimeError { message, .. }) => {
            assert!(
                message.contains("Division by zero"),
                "unexpected {}",
                message
            );
        }
        other => panic!("expected runtime error, got {:?}", other),
    }
}

#[test]
fn evaluates_interpolated_string() {
    let mut vm = VirtualMachine::new();
    vm.environment_mut().define(
        "name".to_string(),
        Object::String(Rc::new("Metorex".to_string())),
    );

    let assignment = Statement::Assignment {
        target: Expression::Identifier {
            name: "message".to_string(),
            position: pos(1, 1),
        },
        value: Expression::InterpolatedString {
            parts: vec![
                InterpolationPart::Text("Hello, ".to_string()),
                InterpolationPart::Expression(Box::new(Expression::Identifier {
                    name: "name".to_string(),
                    position: pos(1, 10),
                })),
                InterpolationPart::Text("!".to_string()),
            ],
            position: pos(1, 1),
        },
        position: pos(1, 1),
    };

    vm.execute_program(&[assignment]).expect("execution failed");
    assert_eq!(
        vm.environment().get("message"),
        Some(Object::String(Rc::new("Hello, Metorex!".to_string())))
    );
}
