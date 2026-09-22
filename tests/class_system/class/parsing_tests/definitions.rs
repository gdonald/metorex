// The AST a class definition parses into.

use super::*;
// Tests for basic class definitions

#[test]
fn test_empty_class() {
    let stmt = Statement::ClassDef {
        name: "Empty".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
    assert!(!stmt.is_control_flow());
}

#[test]
fn test_class_with_simple_name() {
    let stmt = Statement::ClassDef {
        name: "Person".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}

#[test]
fn test_class_with_uppercase_name() {
    let stmt = Statement::ClassDef {
        name: "MyClass".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for class inheritance

#[test]
fn test_class_with_superclass() {
    let stmt = Statement::ClassDef {
        name: "Dog".to_string(),
        namespace: None,
        superclass: Some("Animal".to_string()),
        superclass_expression: None,
        body: vec![],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}

#[test]
fn test_class_inheritance_chain() {
    // Grandparent class
    let _grandparent = Statement::ClassDef {
        name: "LivingThing".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![],
        position: pos(1, 1),
    };

    // Parent class inheriting from grandparent
    let _parent = Statement::ClassDef {
        name: "Animal".to_string(),
        namespace: None,
        superclass: Some("LivingThing".to_string()),
        superclass_expression: None,
        body: vec![],
        position: pos(5, 1),
    };

    // Child class inheriting from parent
    let child = Statement::ClassDef {
        name: "Dog".to_string(),
        namespace: None,
        superclass: Some("Animal".to_string()),
        superclass_expression: None,
        body: vec![],
        position: pos(10, 1),
    };

    assert_eq!(child.position(), pos(10, 1));
}

// Tests for class with methods

#[test]
fn test_class_with_single_method() {
    let stmt = Statement::ClassDef {
        name: "Greeter".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "greet".to_string(),
            parameters: vec![],
            body: vec![Statement::Return {
                value: Some(Expression::StringLiteral {
                    value: "Hello".to_string(),
                    position: pos(3, 5),
                }),
                position: pos(3, 5),
            }],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}

#[test]
fn test_class_with_multiple_methods() {
    let stmt = Statement::ClassDef {
        name: "Calculator".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![
            Statement::MethodDef {
                is_class_method: false,
                name: "add".to_string(),
                parameters: vec![
                    Parameter::simple("a".to_string(), pos(2, 11)),
                    Parameter::simple("b".to_string(), pos(2, 14)),
                ],
                body: vec![Statement::Return {
                    value: Some(Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "a".to_string(),
                            position: pos(3, 5),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "b".to_string(),
                            position: pos(3, 9),
                        }),
                        position: pos(3, 7),
                    }),
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
                end_position: pos(2, 3),
            },
            Statement::MethodDef {
                is_class_method: false,
                name: "subtract".to_string(),
                parameters: vec![
                    Parameter::simple("a".to_string(), pos(6, 16)),
                    Parameter::simple("b".to_string(), pos(6, 19)),
                ],
                body: vec![Statement::Return {
                    value: Some(Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::Subtract,
                        left: Box::new(Expression::Identifier {
                            name: "a".to_string(),
                            position: pos(7, 5),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "b".to_string(),
                            position: pos(7, 9),
                        }),
                        position: pos(7, 7),
                    }),
                    position: pos(7, 5),
                }],
                position: pos(6, 3),
                end_position: pos(6, 3),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for constructor (initialize method)

#[test]
fn test_class_with_constructor() {
    let stmt = Statement::ClassDef {
        name: "Person".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "initialize".to_string(),
            parameters: vec![
                Parameter::simple("name".to_string(), pos(2, 17)),
                Parameter::simple("age".to_string(), pos(2, 23)),
            ],
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
                        name: "age".to_string(),
                        position: pos(4, 5),
                    },
                    value: Expression::Identifier {
                        name: "age".to_string(),
                        position: pos(4, 12),
                    },
                    position: pos(4, 5),
                },
            ],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_class_with_constructor_and_methods() {
    let stmt = Statement::ClassDef {
        name: "Rectangle".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![
            Statement::MethodDef {
                is_class_method: false,
                name: "initialize".to_string(),
                parameters: vec![
                    Parameter::simple("width".to_string(), pos(2, 17)),
                    Parameter::simple("height".to_string(), pos(2, 24)),
                ],
                body: vec![
                    Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "width".to_string(),
                            position: pos(3, 5),
                        },
                        value: Expression::Identifier {
                            name: "width".to_string(),
                            position: pos(3, 14),
                        },
                        position: pos(3, 5),
                    },
                    Statement::Assignment {
                        target: Expression::InstanceVariable {
                            name: "height".to_string(),
                            position: pos(4, 5),
                        },
                        value: Expression::Identifier {
                            name: "height".to_string(),
                            position: pos(4, 15),
                        },
                        position: pos(4, 5),
                    },
                ],
                position: pos(2, 3),
                end_position: pos(2, 3),
            },
            Statement::MethodDef {
                is_class_method: false,
                name: "area".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::Multiply,
                        left: Box::new(Expression::InstanceVariable {
                            name: "width".to_string(),
                            position: pos(8, 5),
                        }),
                        right: Box::new(Expression::InstanceVariable {
                            name: "height".to_string(),
                            position: pos(8, 14),
                        }),
                        position: pos(8, 12),
                    }),
                    position: pos(8, 5),
                }],
                position: pos(7, 3),
                end_position: pos(7, 3),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for instance variables

#[test]
fn test_class_with_instance_variable_initialization() {
    let stmt = Statement::ClassDef {
        name: "Counter".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "initialize".to_string(),
            parameters: vec![],
            body: vec![Statement::Assignment {
                target: Expression::InstanceVariable {
                    name: "count".to_string(),
                    position: pos(3, 5),
                },
                value: Expression::IntLiteral {
                    value: 0,
                    position: pos(3, 14),
                },
                position: pos(3, 5),
            }],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_class_with_multiple_instance_variables() {
    let stmt = Statement::ClassDef {
        name: "Person".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "initialize".to_string(),
            parameters: vec![
                Parameter::simple("name".to_string(), pos(2, 17)),
                Parameter::simple("age".to_string(), pos(2, 23)),
                Parameter::simple("email".to_string(), pos(2, 28)),
            ],
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
                        name: "age".to_string(),
                        position: pos(4, 5),
                    },
                    value: Expression::Identifier {
                        name: "age".to_string(),
                        position: pos(4, 12),
                    },
                    position: pos(4, 5),
                },
                Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: "email".to_string(),
                        position: pos(5, 5),
                    },
                    value: Expression::Identifier {
                        name: "email".to_string(),
                        position: pos(5, 14),
                    },
                    position: pos(5, 5),
                },
            ],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for class variables

#[test]
fn test_class_with_class_variable() {
    let stmt = Statement::ClassDef {
        name: "SharedCounter".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "initialize".to_string(),
            parameters: vec![],
            body: vec![Statement::Assignment {
                target: Expression::ClassVariable {
                    name: "count".to_string(),
                    position: pos(3, 5),
                },
                value: Expression::IntLiteral {
                    value: 0,
                    position: pos(3, 15),
                },
                position: pos(3, 5),
            }],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for methods with various parameter types

#[test]
fn test_class_method_with_default_parameters() {
    let stmt = Statement::ClassDef {
        name: "Configurator".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "setup".to_string(),
            parameters: vec![
                Parameter::simple("name".to_string(), pos(2, 12)),
                Parameter::with_default(
                    "debug".to_string(),
                    Expression::BoolLiteral {
                        value: false,
                        position: pos(2, 26),
                    },
                    pos(2, 18),
                ),
            ],
            body: vec![],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}
