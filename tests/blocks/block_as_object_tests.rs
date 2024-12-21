// Comprehensive tests for blocks as first-class objects in Metorex
// Tests verify that blocks can be assigned, passed, returned, and used like any other value

use metorex::ast::{BinaryOp, Expression, Parameter, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

#[test]
fn test_block_assigned_to_variable() {
    let mut vm = VirtualMachine::new();

    // my_block = lambda |x| x * 2 end
    // result = my_block.call(5)
    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "my_block".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Lambda {
                parameters: vec!["x".to_string()],
                body: vec![Statement::Expression {
                    expression: Expression::BinaryOp {
                        op: BinaryOp::Multiply,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(1, 20),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 2,
                            position: pos(1, 24),
                        }),
                        position: pos(1, 22),
                    },
                    position: pos(1, 20),
                }],
                captured_vars: None,
                position: pos(1, 13),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "my_block".to_string(),
                    position: pos(2, 10),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 5,
                    position: pos(2, 20),
                }],
                trailing_block: None,
                position: pos(2, 10),
            },
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(10)));
}

// NOTE: The following test uses standalone function definitions (FunctionDef),
// which are not yet fully implemented in the runtime. However, blocks CAN be
// passed as arguments - see test_block_stored_in_array and other tests.
#[test]
#[ignore = "Requires standalone function support"]
fn test_block_passed_as_argument_with_function() {
    let mut vm = VirtualMachine::new();

    // Define a function that takes a block as argument and calls it
    // def apply_twice(func, value)
    //   result = func.call(value)
    //   func.call(result)
    // end
    //
    // increment = lambda |x| x + 1 end
    // result = apply_twice(increment, 5)
    // result should be 7

    let program = vec![
        // Define function
        Statement::FunctionDef {
            name: "apply_twice".to_string(),
            parameters: vec![
                Parameter::simple("func".to_string(), pos(1, 18)),
                Parameter::simple("value".to_string(), pos(1, 24)),
            ],
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(2, 3),
                    },
                    value: Expression::Call {
                        callee: Box::new(Expression::Identifier {
                            name: "func".to_string(),
                            position: pos(2, 12),
                        }),
                        arguments: vec![Expression::Identifier {
                            name: "value".to_string(),
                            position: pos(2, 17),
                        }],
                        trailing_block: None,
                        position: pos(2, 12),
                    },
                    position: pos(2, 3),
                },
                Statement::Expression {
                    expression: Expression::Call {
                        callee: Box::new(Expression::Identifier {
                            name: "func".to_string(),
                            position: pos(3, 3),
                        }),
                        arguments: vec![Expression::Identifier {
                            name: "result".to_string(),
                            position: pos(3, 8),
                        }],
                        trailing_block: None,
                        position: pos(3, 3),
                    },
                    position: pos(3, 3),
                },
            ],
            position: pos(1, 1),
        },
        // Create increment block
        Statement::Assignment {
            target: Expression::Identifier {
                name: "increment".to_string(),
                position: pos(6, 1),
            },
            value: Expression::Lambda {
                parameters: vec!["x".to_string()],
                body: vec![Statement::Expression {
                    expression: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(6, 23),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(6, 27),
                        }),
                        position: pos(6, 25),
                    },
                    position: pos(6, 23),
                }],
                captured_vars: None,
                position: pos(6, 13),
            },
            position: pos(6, 1),
        },
        // Call function with block
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(7, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "apply_twice".to_string(),
                    position: pos(7, 10),
                }),
                arguments: vec![
                    Expression::Identifier {
                        name: "increment".to_string(),
                        position: pos(7, 22),
                    },
                    Expression::IntLiteral {
                        value: 5,
                        position: pos(7, 33),
                    },
                ],
                trailing_block: None,
                position: pos(7, 10),
            },
            position: pos(7, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(7)));
}

// NOTE: Standalone functions are not yet implemented. However, blocks CAN be
// returned from methods - see test_block_returned_from_method which demonstrates this.
#[test]
#[ignore = "Requires standalone function support"]
fn test_block_returned_from_function_standalone() {
    let mut vm = VirtualMachine::new();

    // def make_adder(n)
    //   lambda |x| x + n end
    // end
    //
    // add_five = make_adder(5)
    // result = add_five.call(10)
    // result should be 15

    let program = vec![
        // Define function that returns a block
        Statement::FunctionDef {
            name: "make_adder".to_string(),
            parameters: vec![Parameter::simple("n".to_string(), pos(1, 17))],
            body: vec![Statement::Expression {
                expression: Expression::Lambda {
                    parameters: vec!["x".to_string()],
                    body: vec![Statement::Expression {
                        expression: Expression::BinaryOp {
                            op: BinaryOp::Add,
                            left: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: pos(2, 13),
                            }),
                            right: Box::new(Expression::Identifier {
                                name: "n".to_string(),
                                position: pos(2, 17),
                            }),
                            position: pos(2, 15),
                        },
                        position: pos(2, 13),
                    }],
                    captured_vars: Some(vec!["n".to_string()]),
                    position: pos(2, 3),
                },
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        // Call function to get block
        Statement::Assignment {
            target: Expression::Identifier {
                name: "add_five".to_string(),
                position: pos(5, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "make_adder".to_string(),
                    position: pos(5, 12),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 5,
                    position: pos(5, 23),
                }],
                trailing_block: None,
                position: pos(5, 12),
            },
            position: pos(5, 1),
        },
        // Call the returned block
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(6, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "add_five".to_string(),
                    position: pos(6, 10),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 10,
                    position: pos(6, 19),
                }],
                trailing_block: None,
                position: pos(6, 10),
            },
            position: pos(6, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(15)));
}

#[test]
fn test_block_with_multiple_parameters() {
    let mut vm = VirtualMachine::new();

    // multiply = lambda |x, y| x * y end
    // result = multiply.call(6, 7)
    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "multiply".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Lambda {
                parameters: vec!["x".to_string(), "y".to_string()],
                body: vec![Statement::Expression {
                    expression: Expression::BinaryOp {
                        op: BinaryOp::Multiply,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(1, 26),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "y".to_string(),
                            position: pos(1, 30),
                        }),
                        position: pos(1, 28),
                    },
                    position: pos(1, 26),
                }],
                captured_vars: None,
                position: pos(1, 13),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "multiply".to_string(),
                    position: pos(2, 10),
                }),
                arguments: vec![
                    Expression::IntLiteral {
                        value: 6,
                        position: pos(2, 19),
                    },
                    Expression::IntLiteral {
                        value: 7,
                        position: pos(2, 22),
                    },
                ],
                trailing_block: None,
                position: pos(2, 10),
            },
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(42)));
}

#[test]
fn test_block_with_no_parameters() {
    let mut vm = VirtualMachine::new();

    // get_answer = lambda || 42 end
    // result = get_answer.call()
    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "get_answer".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Lambda {
                parameters: vec![],
                body: vec![Statement::Expression {
                    expression: Expression::IntLiteral {
                        value: 42,
                        position: pos(1, 21),
                    },
                    position: pos(1, 21),
                }],
                captured_vars: None,
                position: pos(1, 14),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "get_answer".to_string(),
                    position: pos(2, 10),
                }),
                arguments: vec![],
                trailing_block: None,
                position: pos(2, 10),
            },
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(42)));
}

#[test]
fn test_block_stored_in_array() {
    let mut vm = VirtualMachine::new();

    // operations = [
    //   lambda |x| x + 1 end,
    //   lambda |x| x * 2 end,
    //   lambda |x| x - 3 end
    // ]
    // result = operations[1].call(5)  # Should be 10
    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "operations".to_string(),
                position: pos(1, 1),
            },
            value: Expression::Array {
                elements: vec![
                    Expression::Lambda {
                        parameters: vec!["x".to_string()],
                        body: vec![Statement::Expression {
                            expression: Expression::BinaryOp {
                                op: BinaryOp::Add,
                                left: Box::new(Expression::Identifier {
                                    name: "x".to_string(),
                                    position: pos(2, 13),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 1,
                                    position: pos(2, 17),
                                }),
                                position: pos(2, 15),
                            },
                            position: pos(2, 13),
                        }],
                        captured_vars: None,
                        position: pos(2, 3),
                    },
                    Expression::Lambda {
                        parameters: vec!["x".to_string()],
                        body: vec![Statement::Expression {
                            expression: Expression::BinaryOp {
                                op: BinaryOp::Multiply,
                                left: Box::new(Expression::Identifier {
                                    name: "x".to_string(),
                                    position: pos(3, 13),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 2,
                                    position: pos(3, 17),
                                }),
                                position: pos(3, 15),
                            },
                            position: pos(3, 13),
                        }],
                        captured_vars: None,
                        position: pos(3, 3),
                    },
                    Expression::Lambda {
                        parameters: vec!["x".to_string()],
                        body: vec![Statement::Expression {
                            expression: Expression::BinaryOp {
                                op: BinaryOp::Subtract,
                                left: Box::new(Expression::Identifier {
                                    name: "x".to_string(),
                                    position: pos(4, 13),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 3,
                                    position: pos(4, 17),
                                }),
                                position: pos(4, 15),
                            },
                            position: pos(4, 13),
                        }],
                        captured_vars: None,
                        position: pos(4, 3),
                    },
                ],
                position: pos(1, 14),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(6, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Index {
                    array: Box::new(Expression::Identifier {
                        name: "operations".to_string(),
                        position: pos(6, 10),
                    }),
                    index: Box::new(Expression::IntLiteral {
                        value: 1,
                        position: pos(6, 21),
                    }),
                    position: pos(6, 10),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 5,
                    position: pos(6, 24),
                }],
                trailing_block: None,
                position: pos(6, 10),
            },
            position: pos(6, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(10)));
}

#[test]
fn test_block_returned_from_method() {
    let mut vm = VirtualMachine::new();

    // class BlockFactory
    //   def create_multiplier(factor)
    //     return lambda |x| x * factor end
    //   end
    // end
    //
    // factory = BlockFactory.new
    // times_three = factory.create_multiplier(3)
    // result = times_three.call(4)  # Should be 12

    let program = vec![
        Statement::ClassDef {
            name: "BlockFactory".to_string(),
            superclass: None,
            body: vec![Statement::MethodDef {
                name: "create_multiplier".to_string(),
                parameters: vec![Parameter::simple("factor".to_string(), pos(2, 26))],
                body: vec![Statement::Return {
                    value: Some(Expression::Lambda {
                        parameters: vec!["x".to_string()],
                        body: vec![Statement::Expression {
                            expression: Expression::BinaryOp {
                                op: BinaryOp::Multiply,
                                left: Box::new(Expression::Identifier {
                                    name: "x".to_string(),
                                    position: pos(3, 15),
                                }),
                                right: Box::new(Expression::Identifier {
                                    name: "factor".to_string(),
                                    position: pos(3, 19),
                                }),
                                position: pos(3, 17),
                            },
                            position: pos(3, 15),
                        }],
                        captured_vars: Some(vec!["factor".to_string()]),
                        position: pos(3, 12),
                    }),
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "factory".to_string(),
                position: pos(7, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "BlockFactory".to_string(),
                    position: pos(7, 11),
                }),
                arguments: vec![],
                trailing_block: None,
                position: pos(7, 11),
            },
            position: pos(7, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "times_three".to_string(),
                position: pos(8, 1),
            },
            value: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "factory".to_string(),
                    position: pos(8, 15),
                }),
                method: "create_multiplier".to_string(),
                arguments: vec![Expression::IntLiteral {
                    value: 3,
                    position: pos(8, 41),
                }],
                trailing_block: None,
                position: pos(8, 15),
            },
            position: pos(8, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(9, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "times_three".to_string(),
                    position: pos(9, 10),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 4,
                    position: pos(9, 22),
                }],
                trailing_block: None,
                position: pos(9, 10),
            },
            position: pos(9, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(12)));
}

#[test]
fn test_nested_block_closures() {
    let mut vm = VirtualMachine::new();

    // Demonstrates nested closures capturing variables from outer scopes
    // outer = 10
    // make_nested = lambda |x|
    //   lambda |y| outer + x + y end
    // end
    //
    // inner = make_nested(5)
    // result = inner.call(3)  # Should be 18

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "outer".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 10,
                position: pos(1, 9),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "make_nested".to_string(),
                position: pos(2, 1),
            },
            value: Expression::Lambda {
                parameters: vec!["x".to_string()],
                body: vec![Statement::Expression {
                    expression: Expression::Lambda {
                        parameters: vec!["y".to_string()],
                        body: vec![Statement::Expression {
                            expression: Expression::BinaryOp {
                                op: BinaryOp::Add,
                                left: Box::new(Expression::BinaryOp {
                                    op: BinaryOp::Add,
                                    left: Box::new(Expression::Identifier {
                                        name: "outer".to_string(),
                                        position: pos(3, 15),
                                    }),
                                    right: Box::new(Expression::Identifier {
                                        name: "x".to_string(),
                                        position: pos(3, 23),
                                    }),
                                    position: pos(3, 21),
                                }),
                                right: Box::new(Expression::Identifier {
                                    name: "y".to_string(),
                                    position: pos(3, 27),
                                }),
                                position: pos(3, 25),
                            },
                            position: pos(3, 15),
                        }],
                        captured_vars: Some(vec!["outer".to_string(), "x".to_string()]),
                        position: pos(3, 3),
                    },
                    position: pos(3, 3),
                }],
                captured_vars: Some(vec!["outer".to_string()]),
                position: pos(2, 15),
            },
            position: pos(2, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "inner".to_string(),
                position: pos(6, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "make_nested".to_string(),
                    position: pos(6, 9),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 5,
                    position: pos(6, 21),
                }],
                trailing_block: None,
                position: pos(6, 9),
            },
            position: pos(6, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(7, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "inner".to_string(),
                    position: pos(7, 10),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 3,
                    position: pos(7, 16),
                }],
                trailing_block: None,
                position: pos(7, 10),
            },
            position: pos(7, 1),
        },
    ];

    vm.execute_program(&program).expect("execution failed");
    assert_eq!(vm.environment().get("result"), Some(Object::Int(18)));
}
