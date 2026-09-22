// The AST the statements of a class body parse into.

use super::*;
#[test]
fn test_class_method_with_variadic_parameters() {
    let stmt = Statement::ClassDef {
        name: "Logger".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "log".to_string(),
            parameters: vec![Parameter::variadic("messages".to_string(), pos(2, 11))],
            body: vec![],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_class_method_with_keyword_parameters() {
    let stmt = Statement::ClassDef {
        name: "ConfigManager".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "configure".to_string(),
            parameters: vec![Parameter::keyword("options".to_string(), pos(2, 17))],
            body: vec![],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for getter and setter methods

#[test]
fn test_class_with_getter_and_setter() {
    let stmt = Statement::ClassDef {
        name: "Box".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![
            Statement::MethodDef {
                is_class_method: false,
                name: "initialize".to_string(),
                parameters: vec![Parameter::simple("value".to_string(), pos(2, 17))],
                body: vec![Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: "value".to_string(),
                        position: pos(3, 5),
                    },
                    value: Expression::Identifier {
                        name: "value".to_string(),
                        position: pos(3, 14),
                    },
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
                end_position: pos(2, 3),
            },
            Statement::MethodDef {
                is_class_method: false,
                name: "get_value".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::InstanceVariable {
                        name: "value".to_string(),
                        position: pos(7, 5),
                    }),
                    position: pos(7, 5),
                }],
                position: pos(6, 3),
                end_position: pos(6, 3),
            },
            Statement::MethodDef {
                is_class_method: false,
                name: "set_value".to_string(),
                parameters: vec![Parameter::simple("value".to_string(), pos(10, 17))],
                body: vec![Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: "value".to_string(),
                        position: pos(11, 5),
                    },
                    value: Expression::Identifier {
                        name: "value".to_string(),
                        position: pos(11, 14),
                    },
                    position: pos(11, 5),
                }],
                position: pos(10, 3),
                end_position: pos(10, 3),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for methods with control flow

#[test]
fn test_class_method_with_conditional() {
    let stmt = Statement::ClassDef {
        name: "Validator".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "is_positive".to_string(),
            parameters: vec![Parameter::simple("n".to_string(), pos(2, 19))],
            body: vec![Statement::If {
                condition: Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Greater,
                    left: Box::new(Expression::Identifier {
                        name: "n".to_string(),
                        position: pos(3, 8),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 12),
                    }),
                    position: pos(3, 10),
                },
                then_branch: vec![Statement::Return {
                    value: Some(Expression::BoolLiteral {
                        value: true,
                        position: pos(4, 7),
                    }),
                    position: pos(4, 7),
                }],
                elsif_branches: vec![],
                else_branch: Some(vec![Statement::Return {
                    value: Some(Expression::BoolLiteral {
                        value: false,
                        position: pos(6, 7),
                    }),
                    position: pos(6, 7),
                }]),
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
fn test_class_method_with_loop() {
    let stmt = Statement::ClassDef {
        name: "Summer".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "sum_to".to_string(),
            parameters: vec![Parameter::simple("n".to_string(), pos(2, 14))],
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(3, 5),
                    },
                    value: Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 11),
                    },
                    position: pos(3, 5),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "i".to_string(),
                        position: pos(4, 5),
                    },
                    value: Expression::IntLiteral {
                        value: 1,
                        position: pos(4, 9),
                    },
                    position: pos(4, 5),
                },
                Statement::While {
                    condition: Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::LessEqual,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(5, 11),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "n".to_string(),
                            position: pos(5, 16),
                        }),
                        position: pos(5, 13),
                    },
                    body: vec![
                        Statement::Assignment {
                            target: Expression::Identifier {
                                name: "sum".to_string(),
                                position: pos(6, 7),
                            },
                            value: Expression::BinaryOp {
                                op: metorex::ast::BinaryOp::Add,
                                left: Box::new(Expression::Identifier {
                                    name: "sum".to_string(),
                                    position: pos(6, 13),
                                }),
                                right: Box::new(Expression::Identifier {
                                    name: "i".to_string(),
                                    position: pos(6, 19),
                                }),
                                position: pos(6, 17),
                            },
                            position: pos(6, 7),
                        },
                        Statement::Assignment {
                            target: Expression::Identifier {
                                name: "i".to_string(),
                                position: pos(7, 7),
                            },
                            value: Expression::BinaryOp {
                                op: metorex::ast::BinaryOp::Add,
                                left: Box::new(Expression::Identifier {
                                    name: "i".to_string(),
                                    position: pos(7, 11),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 1,
                                    position: pos(7, 15),
                                }),
                                position: pos(7, 13),
                            },
                            position: pos(7, 7),
                        },
                    ],
                    position: pos(5, 5),
                },
                Statement::Return {
                    value: Some(Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(9, 5),
                    }),
                    position: pos(9, 5),
                },
            ],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for inheritance with methods

#[test]
fn test_inherited_class_with_override() {
    let _parent = Statement::ClassDef {
        name: "Animal".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
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
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    let child = Statement::ClassDef {
        name: "Dog".to_string(),
        namespace: None,
        superclass: Some("Animal".to_string()),
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "speak".to_string(),
            parameters: vec![],
            body: vec![Statement::Return {
                value: Some(Expression::StringLiteral {
                    value: "Woof!".to_string(),
                    position: pos(9, 5),
                }),
                position: pos(9, 5),
            }],
            position: pos(8, 3),
            end_position: pos(8, 3),
        }],
        position: pos(7, 1),
    };

    assert_eq!(child.position(), pos(7, 1));
}

#[test]
fn test_inherited_class_with_additional_methods() {
    let child = Statement::ClassDef {
        name: "Dog".to_string(),
        namespace: None,
        superclass: Some("Animal".to_string()),
        superclass_expression: None,
        body: vec![
            Statement::MethodDef {
                is_class_method: false,
                name: "speak".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "Woof!".to_string(),
                        position: pos(3, 5),
                    }),
                    position: pos(3, 5),
                }],
                position: pos(2, 3),
                end_position: pos(2, 3),
            },
            Statement::MethodDef {
                is_class_method: false,
                name: "fetch".to_string(),
                parameters: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::StringLiteral {
                        value: "Fetching!".to_string(),
                        position: pos(7, 5),
                    }),
                    position: pos(7, 5),
                }],
                position: pos(6, 3),
                end_position: pos(6, 3),
            },
        ],
        position: pos(1, 1),
    };

    assert_eq!(child.position(), pos(1, 1));
}

// Tests for complex class scenarios

#[test]
fn test_class_with_nested_function() {
    let stmt = Statement::ClassDef {
        name: "Outer".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "outer_method".to_string(),
            parameters: vec![Parameter::simple("x".to_string(), pos(2, 20))],
            body: vec![Statement::FunctionDef {
                name: "inner_function".to_string(),
                parameters: vec![Parameter::simple("y".to_string(), pos(3, 24))],
                body: vec![Statement::Return {
                    value: Some(Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(4, 7),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "y".to_string(),
                            position: pos(4, 11),
                        }),
                        position: pos(4, 9),
                    }),
                    position: pos(4, 7),
                }],
                position: pos(3, 5),
                singleton_class: None,
            }],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_class_with_self_reference() {
    let stmt = Statement::ClassDef {
        name: "Chainable".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "set_value".to_string(),
            parameters: vec![Parameter::simple("value".to_string(), pos(2, 17))],
            body: vec![
                Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: "value".to_string(),
                        position: pos(3, 5),
                    },
                    value: Expression::Identifier {
                        name: "value".to_string(),
                        position: pos(3, 14),
                    },
                    position: pos(3, 5),
                },
                Statement::Return {
                    value: Some(Expression::SelfExpr {
                        position: pos(4, 5),
                    }),
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
fn test_class_with_exception_handling() {
    let stmt = Statement::ClassDef {
        name: "SafeCalculator".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![Statement::MethodDef {
            is_class_method: false,
            name: "divide".to_string(),
            parameters: vec![
                Parameter::simple("a".to_string(), pos(2, 14)),
                Parameter::simple("b".to_string(), pos(2, 17)),
            ],
            body: vec![Statement::Begin {
                body: vec![Statement::Return {
                    value: Some(Expression::BinaryOp {
                        op: metorex::ast::BinaryOp::Divide,
                        left: Box::new(Expression::Identifier {
                            name: "a".to_string(),
                            position: pos(4, 7),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "b".to_string(),
                            position: pos(4, 11),
                        }),
                        position: pos(4, 9),
                    }),
                    position: pos(4, 7),
                }],
                rescue_clauses: vec![metorex::ast::RescueClause {
                    exception_types: vec!["ZeroDivisionError".to_string()],
                    variable_name: None,
                    variable_target: None,
                    splatted_types: Vec::new(),
                    body: vec![Statement::Return {
                        value: Some(Expression::NilLiteral {
                            position: pos(6, 7),
                        }),
                        position: pos(6, 7),
                    }],
                    position: pos(5, 5),
                }],
                else_clause: None,
                ensure_block: None,
                position: pos(3, 5),
            }],
            position: pos(2, 3),
            end_position: pos(2, 3),
        }],
        position: pos(1, 1),
    };

    assert_eq!(stmt.position(), pos(1, 1));
}

// Tests for empty classes and edge cases

#[test]
fn test_multiple_empty_classes() {
    let class1 = Statement::ClassDef {
        name: "First".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![],
        position: pos(1, 1),
    };

    let class2 = Statement::ClassDef {
        name: "Second".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![],
        position: pos(5, 1),
    };

    let class3 = Statement::ClassDef {
        name: "Third".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: vec![],
        position: pos(9, 1),
    };

    assert_eq!(class1.position(), pos(1, 1));
    assert_eq!(class2.position(), pos(5, 1));
    assert_eq!(class3.position(), pos(9, 1));
}

#[test]
fn test_class_name_variations() {
    let names = vec![
        "A",
        "MyClass",
        "HTTPServer",
        "XML_Parser",
        "JSON2CSV",
        "ClassWithAVeryLongNameThatIsStillValid",
    ];

    for name in names {
        let stmt = Statement::ClassDef {
            name: name.to_string(),
            namespace: None,
            superclass: None,
            superclass_expression: None,
            body: vec![],
            position: pos(1, 1),
        };
        assert!(stmt.is_definition());
    }
}
