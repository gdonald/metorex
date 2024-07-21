// Unit tests for class instantiation and instance creation in the VM

use metorex::ast::{Expression, Parameter, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

#[test]
fn test_simple_class_definition() {
    let mut vm = VirtualMachine::new();

    // class Point
    // end
    let program = vec![Statement::ClassDef {
        name: "Point".to_string(),
        superclass: None,
        body: vec![],
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    // Check that Point is defined as a Class object
    let point_class = vm.environment().get("Point").unwrap();
    assert!(matches!(point_class, Object::Class(_)));
}

#[test]
fn test_class_with_initialize_method() {
    let mut vm = VirtualMachine::new();

    // class Point
    //   def initialize(x, y)
    //     @x = x
    //     @y = y
    //   end
    // end
    let program = vec![Statement::ClassDef {
        name: "Point".to_string(),
        superclass: None,
        body: vec![Statement::MethodDef {
            name: "initialize".to_string(),
            parameters: vec![
                Parameter::simple("x".to_string(), pos(2, 18)),
                Parameter::simple("y".to_string(), pos(2, 21)),
            ],
            body: vec![
                Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: "x".to_string(),
                        position: pos(3, 5),
                    },
                    value: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 10),
                    },
                    position: pos(3, 5),
                },
                Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: "y".to_string(),
                        position: pos(4, 5),
                    },
                    value: Expression::Identifier {
                        name: "y".to_string(),
                        position: pos(4, 10),
                    },
                    position: pos(4, 5),
                },
            ],
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    }];

    vm.execute_program(&program).unwrap();

    // Verify class was created
    let point_class = vm.environment().get("Point").unwrap();
    assert!(matches!(point_class, Object::Class(_)));
}

#[test]
fn test_create_instance_without_initialize() {
    let mut vm = VirtualMachine::new();

    // class Simple
    // end
    // s = Simple()
    let program = vec![
        Statement::ClassDef {
            name: "Simple".to_string(),
            superclass: None,
            body: vec![],
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "s".to_string(),
                position: pos(3, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Simple".to_string(),
                    position: pos(3, 5),
                }),
                arguments: vec![],
                trailing_block: None,
                position: pos(3, 5),
            },
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    // Check that s is an instance
    let s = vm.environment().get("s").unwrap();
    assert!(matches!(s, Object::Instance(_)));
}

#[test]
fn test_create_instance_with_initialize() {
    let mut vm = VirtualMachine::new();

    // class Point
    //   def initialize(x, y)
    //     @x = x
    //     @y = y
    //   end
    // end
    // p = Point(10, 20)
    let program = vec![
        Statement::ClassDef {
            name: "Point".to_string(),
            superclass: None,
            body: vec![Statement::MethodDef {
                name: "initialize".to_string(),
                parameters: vec![
                    Parameter::simple("x".to_string(), pos(2, 18)),
                    Parameter::simple("y".to_string(), pos(2, 21)),
                ],
                body: vec![
                    Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "x".to_string(),
                            position: pos(3, 5),
                        },
                        value: Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(3, 10),
                        },
                        position: pos(3, 5),
                    },
                    Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "y".to_string(),
                            position: pos(4, 5),
                        },
                        value: Expression::Identifier {
                            name: "y".to_string(),
                            position: pos(4, 10),
                        },
                        position: pos(4, 5),
                    },
                ],
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "p".to_string(),
                position: pos(7, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Point".to_string(),
                    position: pos(7, 5),
                }),
                arguments: vec![
                    Expression::IntLiteral {
                        value: 10,
                        position: pos(7, 11),
                    },
                    Expression::IntLiteral {
                        value: 20,
                        position: pos(7, 15),
                    },
                ],
                trailing_block: None,
                position: pos(7, 5),
            },
            position: pos(7, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    // Check that p is an instance
    let p = vm.environment().get("p").unwrap();
    assert!(matches!(p, Object::Instance(_)));
}

#[test]
fn test_instance_variable_access() {
    let mut vm = VirtualMachine::new();

    // class Counter
    //   def initialize(value)
    //     @count = value
    //   end
    //   def get_count()
    //     @count
    //   end
    // end
    // c = Counter(42)
    // result = c.get_count()
    let program = vec![
        Statement::ClassDef {
            name: "Counter".to_string(),
            superclass: None,
            body: vec![
                Statement::MethodDef {
                    name: "initialize".to_string(),
                    parameters: vec![Parameter::simple("value".to_string(), pos(2, 18))],
                    body: vec![Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "count".to_string(),
                            position: pos(3, 5),
                        },
                        value: Expression::Identifier {
                            name: "value".to_string(),
                            position: pos(3, 14),
                        },
                        position: pos(3, 5),
                    }],
                    position: pos(2, 3),
                },
                Statement::MethodDef {
                    name: "get_count".to_string(),
                    parameters: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::InstanceVariable {
                            name: "count".to_string(),
                            position: pos(6, 5),
                        }),
                        position: pos(6, 5),
                    }],
                    position: pos(5, 3),
                },
            ],
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "c".to_string(),
                position: pos(9, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Counter".to_string(),
                    position: pos(9, 5),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 42,
                    position: pos(9, 13),
                }],
                trailing_block: None,
                position: pos(9, 5),
            },
            position: pos(9, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(10, 1),
            },
            value: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "c".to_string(),
                    position: pos(10, 10),
                }),
                method: "get_count".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(10, 10),
            },
            position: pos(10, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(42));
}

#[test]
fn test_class_variable_access() {
    let mut vm = VirtualMachine::new();

    // class Counter
    //   @@total = 0
    //   def initialize()
    //     @@total = @@total + 1
    //   end
    //   def get_total()
    //     @@total
    //   end
    // end
    // c1 = Counter()
    // c2 = Counter()
    // result = c1.get_total()
    let program = vec![
        Statement::ClassDef {
            name: "Counter".to_string(),
            superclass: None,
            body: vec![
                Statement::Assignment {
                    target: Expression::ClassVariable {
                        name: "total".to_string(),
                        position: pos(2, 3),
                    },
                    value: Expression::IntLiteral {
                        value: 0,
                        position: pos(2, 13),
                    },
                    position: pos(2, 3),
                },
                Statement::MethodDef {
                    name: "initialize".to_string(),
                    parameters: vec![],
                    body: vec![Statement::Assignment {
                        target: Expression::ClassVariable {
                            name: "total".to_string(),
                            position: pos(4, 5),
                        },
                        value: Expression::BinaryOp {
                            op: metorex::ast::BinaryOp::Add,
                            left: Box::new(Expression::ClassVariable {
                                name: "total".to_string(),
                                position: pos(4, 15),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 1,
                                position: pos(4, 25),
                            }),
                            position: pos(4, 15),
                        },
                        position: pos(4, 5),
                    }],
                    position: pos(3, 3),
                },
                Statement::MethodDef {
                    name: "get_total".to_string(),
                    parameters: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::ClassVariable {
                            name: "total".to_string(),
                            position: pos(7, 5),
                        }),
                        position: pos(7, 5),
                    }],
                    position: pos(6, 3),
                },
            ],
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "c1".to_string(),
                position: pos(10, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Counter".to_string(),
                    position: pos(10, 6),
                }),
                arguments: vec![],
                trailing_block: None,
                position: pos(10, 6),
            },
            position: pos(10, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "c2".to_string(),
                position: pos(11, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Counter".to_string(),
                    position: pos(11, 6),
                }),
                arguments: vec![],
                trailing_block: None,
                position: pos(11, 6),
            },
            position: pos(11, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(12, 1),
            },
            value: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "c1".to_string(),
                    position: pos(12, 10),
                }),
                method: "get_total".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(12, 10),
            },
            position: pos(12, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(2));
}

#[test]
fn test_class_with_inheritance() {
    let mut vm = VirtualMachine::new();

    // class Base
    //   def initialize(x)
    //     @x = x
    //   end
    // end
    // class Derived < Base
    // end
    // d = Derived(42)
    let program = vec![
        Statement::ClassDef {
            name: "Base".to_string(),
            superclass: None,
            body: vec![Statement::MethodDef {
                name: "initialize".to_string(),
                parameters: vec![Parameter::simple("x".to_string(), pos(2, 18))],
                body: vec![Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: "x".to_string(),
                        position: pos(3, 5),
                    },
                    value: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 10),
                    },
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Derived".to_string(),
            superclass: Some("Base".to_string()),
            body: vec![],
            position: pos(6, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "d".to_string(),
                position: pos(8, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Derived".to_string(),
                    position: pos(8, 5),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 42,
                    position: pos(8, 13),
                }],
                trailing_block: None,
                position: pos(8, 5),
            },
            position: pos(8, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    // Check that d is an instance of Derived
    let d = vm.environment().get("d").unwrap();
    assert!(matches!(d, Object::Instance(_)));

    // Verify it's a Derived instance
    if let Object::Instance(instance_rc) = d {
        let instance = instance_rc.borrow();
        assert_eq!(instance.class_name(), "Derived");
        // Verify initialize was called and @x was set
        assert_eq!(instance.get_var("x"), Some(&Object::Int(42)));
    }
}

#[test]
fn test_error_when_superclass_not_found() {
    let mut vm = VirtualMachine::new();

    // class Child < NonExistent
    // end
    let program = vec![Statement::ClassDef {
        name: "Child".to_string(),
        superclass: Some("NonExistent".to_string()),
        body: vec![],
        position: pos(1, 1),
    }];

    let result = vm.execute_program(&program);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Undefined superclass"));
}

#[test]
fn test_error_when_superclass_not_a_class() {
    let mut vm = VirtualMachine::new();

    // x = 42
    // class Child < x
    // end
    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 42,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Child".to_string(),
            superclass: Some("x".to_string()),
            body: vec![],
            position: pos(2, 1),
        },
    ];

    let result = vm.execute_program(&program);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be a class"));
}

#[test]
fn test_error_when_calling_class_with_wrong_argument_count() {
    let mut vm = VirtualMachine::new();

    // class Simple
    // end
    // s = Simple(42)  # Error: no initialize method, but arguments provided
    let program = vec![
        Statement::ClassDef {
            name: "Simple".to_string(),
            superclass: None,
            body: vec![],
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "s".to_string(),
                position: pos(3, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Simple".to_string(),
                    position: pos(3, 5),
                }),
                arguments: vec![Expression::IntLiteral {
                    value: 42,
                    position: pos(3, 12),
                }],
                trailing_block: None,
                position: pos(3, 5),
            },
            position: pos(3, 1),
        },
    ];

    let result = vm.execute_program(&program);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No initialize method"));
}
