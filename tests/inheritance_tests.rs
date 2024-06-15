// Unit tests for inheritance features (method lookup, overriding, etc.)

use metorex::ast::{Expression, Parameter, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

#[test]
fn test_basic_inheritance_method_lookup() {
    let mut vm = VirtualMachine::new();

    // class Animal
    //   def speak()
    //     "Some sound"
    //   end
    // end
    // class Dog < Animal
    // end
    // d = Dog()
    // result = d.speak()
    let program = vec![
        Statement::ClassDef {
            name: "Animal".to_string(),
            superclass: None,
            body: vec![Statement::MethodDef {
                name: "speak".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "Some sound".to_string(),
                        position: pos(3, 5),
                    }),
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Dog".to_string(),
            superclass: Some("Animal".to_string()),
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
                    name: "Dog".to_string(),
                    position: pos(8, 5),
                }),
                arguments: vec![],
                position: pos(8, 5),
            },
            position: pos(8, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(9, 1),
            },
            value: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "d".to_string(),
                    position: pos(9, 10),
                }),
                method: "speak".to_string(),
                arguments: vec![],
                position: pos(9, 10),
            },
            position: pos(9, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(
        result,
        Object::String(std::rc::Rc::new("Some sound".to_string()))
    );
}

#[test]
fn test_method_overriding() {
    let mut vm = VirtualMachine::new();

    // class Animal
    //   def speak()
    //     "Some sound"
    //   end
    // end
    // class Dog < Animal
    //   def speak()
    //     "Woof!"
    //   end
    // end
    // d = Dog()
    // result = d.speak()
    let program = vec![
        Statement::ClassDef {
            name: "Animal".to_string(),
            superclass: None,
            body: vec![Statement::MethodDef {
                name: "speak".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "Some sound".to_string(),
                        position: pos(3, 5),
                    }),
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Dog".to_string(),
            superclass: Some("Animal".to_string()),
            body: vec![Statement::MethodDef {
                name: "speak".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "Woof!".to_string(),
                        position: pos(8, 5),
                    }),
                    position: pos(8, 5),
                }],
                position: pos(7, 3),
            }],
            position: pos(6, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "d".to_string(),
                position: pos(11, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Dog".to_string(),
                    position: pos(11, 5),
                }),
                arguments: vec![],
                position: pos(11, 5),
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
                    name: "d".to_string(),
                    position: pos(12, 10),
                }),
                method: "speak".to_string(),
                arguments: vec![],
                position: pos(12, 10),
            },
            position: pos(12, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(
        result,
        Object::String(std::rc::Rc::new("Woof!".to_string()))
    );
}

#[test]
fn test_inheritance_chain_method_lookup() {
    let mut vm = VirtualMachine::new();

    // class GrandParent
    //   def method_a()
    //     "A"
    //   end
    // end
    // class Parent < GrandParent
    //   def method_b()
    //     "B"
    //   end
    // end
    // class Child < Parent
    //   def method_c()
    //     "C"
    //   end
    // end
    // c = Child()
    // result = c.method_a() # Should find in GrandParent
    let program = vec![
        Statement::ClassDef {
            name: "GrandParent".to_string(),
            superclass: None,
            body: vec![Statement::MethodDef {
                name: "method_a".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "A".to_string(),
                        position: pos(3, 5),
                    }),
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Parent".to_string(),
            superclass: Some("GrandParent".to_string()),
            body: vec![Statement::MethodDef {
                name: "method_b".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "B".to_string(),
                        position: pos(8, 5),
                    }),
                    position: pos(8, 5),
                }],
                position: pos(7, 3),
            }],
            position: pos(6, 1),
        },
        Statement::ClassDef {
            name: "Child".to_string(),
            superclass: Some("Parent".to_string()),
            body: vec![Statement::MethodDef {
                name: "method_c".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "C".to_string(),
                        position: pos(13, 5),
                    }),
                    position: pos(13, 5),
                }],
                position: pos(12, 3),
            }],
            position: pos(11, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "c".to_string(),
                position: pos(16, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Child".to_string(),
                    position: pos(16, 5),
                }),
                arguments: vec![],
                position: pos(16, 5),
            },
            position: pos(16, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(17, 1),
            },
            value: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "c".to_string(),
                    position: pos(17, 10),
                }),
                method: "method_a".to_string(),
                arguments: vec![],
                position: pos(17, 10),
            },
            position: pos(17, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::String(std::rc::Rc::new("A".to_string())));
}

#[test]
fn test_inherited_initialize_method() {
    let mut vm = VirtualMachine::new();

    // class Animal
    //   def initialize(name)
    //     @name = name
    //   end
    //   def get_name()
    //     @name
    //   end
    // end
    // class Dog < Animal
    // end
    // d = Dog("Buddy")
    // result = d.get_name()
    let program = vec![
        Statement::ClassDef {
            name: "Animal".to_string(),
            superclass: None,
            body: vec![
                Statement::MethodDef {
                    name: "initialize".to_string(),
                    parameters: vec![Parameter::simple("name".to_string(), pos(2, 18))],
                    body: vec![Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "name".to_string(),
                            position: pos(3, 5),
                        },
                        value: Expression::Identifier {
                            name: "name".to_string(),
                            position: pos(3, 13),
                        },
                        position: pos(3, 5),
                    }],
                    position: pos(2, 3),
                },
                Statement::MethodDef {
                    name: "get_name".to_string(),
                    parameters: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::InstanceVariable {
                            name: "name".to_string(),
                            position: pos(6, 5),
                        }),
                        position: pos(6, 5),
                    }],
                    position: pos(5, 3),
                },
            ],
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Dog".to_string(),
            superclass: Some("Animal".to_string()),
            body: vec![],
            position: pos(9, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "d".to_string(),
                position: pos(11, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Dog".to_string(),
                    position: pos(11, 5),
                }),
                arguments: vec![Expression::StringLiteral {
                    value: "Buddy".to_string(),
                    position: pos(11, 9),
                }],
                position: pos(11, 5),
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
                    name: "d".to_string(),
                    position: pos(12, 10),
                }),
                method: "get_name".to_string(),
                arguments: vec![],
                position: pos(12, 10),
            },
            position: pos(12, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(
        result,
        Object::String(std::rc::Rc::new("Buddy".to_string()))
    );
}

#[test]
fn test_overriding_with_instance_variables() {
    let mut vm = VirtualMachine::new();

    // class Animal
    //   def initialize(name)
    //     @name = name
    //     @sound = "Some sound"
    //   end
    //   def speak()
    //     @sound
    //   end
    // end
    // class Dog < Animal
    //   def initialize(name)
    //     @name = name
    //     @sound = "Woof!"
    //   end
    // end
    // d = Dog("Buddy")
    // result = d.speak() # Should return "Woof!" due to overridden initialize
    let program = vec![
        Statement::ClassDef {
            name: "Animal".to_string(),
            superclass: None,
            body: vec![
                Statement::MethodDef {
                    name: "initialize".to_string(),
                    parameters: vec![Parameter::simple("name".to_string(), pos(2, 18))],
                    body: vec![
                        Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: "name".to_string(),
                                position: pos(3, 5),
                            },
                            value: Expression::Identifier {
                                name: "name".to_string(),
                                position: pos(3, 13),
                            },
                            position: pos(3, 5),
                        },
                        Statement::Assignment {
                            target: Expression::InstanceVariable {
                                name: "sound".to_string(),
                                position: pos(4, 5),
                            },
                            value: Expression::StringLiteral {
                                value: "Some sound".to_string(),
                                position: pos(4, 14),
                            },
                            position: pos(4, 5),
                        },
                    ],
                    position: pos(2, 3),
                },
                Statement::MethodDef {
                    name: "speak".to_string(),
                    parameters: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::InstanceVariable {
                            name: "sound".to_string(),
                            position: pos(7, 5),
                        }),
                        position: pos(7, 5),
                    }],
                    position: pos(6, 3),
                },
            ],
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Dog".to_string(),
            superclass: Some("Animal".to_string()),
            body: vec![Statement::MethodDef {
                name: "initialize".to_string(),
                parameters: vec![Parameter::simple("name".to_string(), pos(11, 18))],
                body: vec![
                    Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "name".to_string(),
                            position: pos(12, 5),
                        },
                        value: Expression::Identifier {
                            name: "name".to_string(),
                            position: pos(12, 13),
                        },
                        position: pos(12, 5),
                    },
                    Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "sound".to_string(),
                            position: pos(13, 5),
                        },
                        value: Expression::StringLiteral {
                            value: "Woof!".to_string(),
                            position: pos(13, 14),
                        },
                        position: pos(13, 5),
                    },
                ],
                position: pos(11, 3),
            }],
            position: pos(10, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "d".to_string(),
                position: pos(16, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Dog".to_string(),
                    position: pos(16, 5),
                }),
                arguments: vec![Expression::StringLiteral {
                    value: "Buddy".to_string(),
                    position: pos(16, 9),
                }],
                position: pos(16, 5),
            },
            position: pos(16, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(17, 1),
            },
            value: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "d".to_string(),
                    position: pos(17, 10),
                }),
                method: "speak".to_string(),
                arguments: vec![],
                position: pos(17, 10),
            },
            position: pos(17, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(
        result,
        Object::String(std::rc::Rc::new("Woof!".to_string()))
    );
}

#[test]
fn test_multiple_inheritance_levels() {
    let mut vm = VirtualMachine::new();

    // Test that each level can override the previous level
    // GrandParent defines value = 1
    // Parent defines value = 2
    // Child defines value = 3
    let program = vec![
        Statement::ClassDef {
            name: "GrandParent".to_string(),
            superclass: None,
            body: vec![Statement::MethodDef {
                name: "get_value".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::IntLiteral {
                        value: 1,
                        position: pos(3, 5),
                    }),
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::ClassDef {
            name: "Parent".to_string(),
            superclass: Some("GrandParent".to_string()),
            body: vec![Statement::MethodDef {
                name: "get_value".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::IntLiteral {
                        value: 2,
                        position: pos(8, 5),
                    }),
                    position: pos(8, 5),
                }],
                position: pos(7, 3),
            }],
            position: pos(6, 1),
        },
        Statement::ClassDef {
            name: "Child".to_string(),
            superclass: Some("Parent".to_string()),
            body: vec![Statement::MethodDef {
                name: "get_value".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::IntLiteral {
                        value: 3,
                        position: pos(13, 5),
                    }),
                    position: pos(13, 5),
                }],
                position: pos(12, 3),
            }],
            position: pos(11, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "c".to_string(),
                position: pos(16, 1),
            },
            value: Expression::Call {
                callee: Box::new(Expression::Identifier {
                    name: "Child".to_string(),
                    position: pos(16, 5),
                }),
                arguments: vec![],
                position: pos(16, 5),
            },
            position: pos(16, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(17, 1),
            },
            value: Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "c".to_string(),
                    position: pos(17, 10),
                }),
                method: "get_value".to_string(),
                arguments: vec![],
                position: pos(17, 10),
            },
            position: pos(17, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(3));
}
