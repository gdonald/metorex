// Unit tests for method call and access expression AST nodes

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for MethodCall expressions

#[test]
fn test_simple_method_call() {
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "obj".to_string(),
            position: pos(1, 1),
        }),
        method: "foo".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_with_arguments() {
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "obj".to_string(),
            position: pos(1, 1),
        }),
        method: "calculate".to_string(),
        arguments: vec![
            Expression::IntLiteral {
                value: 1,
                position: pos(1, 15),
            },
            Expression::IntLiteral {
                value: 2,
                position: pos(1, 18),
            },
        ],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_zero_arguments() {
    // Testing optional parentheses case - represented same as no-arg call
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "obj".to_string(),
            position: pos(1, 1),
        }),
        method: "property".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_property_access() {
    // Property access is represented as MethodCall with no arguments
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "person".to_string(),
            position: pos(1, 1),
        }),
        method: "name".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_chained_method_calls() {
    // obj.method1().method2()
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "obj".to_string(),
                position: pos(1, 1),
            }),
            method: "method1".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 1),
        }),
        method: "method2".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_triple_chained_method_calls() {
    // obj.foo().bar().baz()
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "obj".to_string(),
                    position: pos(1, 1),
                }),
                method: "foo".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(1, 1),
            }),
            method: "bar".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 1),
        }),
        method: "baz".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_on_instance_variable() {
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::InstanceVariable {
            name: "name".to_string(),
            position: pos(1, 1),
        }),
        method: "upcase".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_on_class_variable() {
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::ClassVariable {
            name: "count".to_string(),
            position: pos(1, 1),
        }),
        method: "increment".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_with_complex_arguments() {
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "calculator".to_string(),
            position: pos(1, 1),
        }),
        method: "compute".to_string(),
        arguments: vec![
            Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::IntLiteral {
                    value: 1,
                    position: pos(1, 20),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 2,
                    position: pos(1, 24),
                }),
                position: pos(1, 22),
            },
            Expression::BinaryOp {
                op: BinaryOp::Multiply,
                left: Box::new(Expression::IntLiteral {
                    value: 3,
                    position: pos(1, 27),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 4,
                    position: pos(1, 31),
                }),
                position: pos(1, 29),
            },
        ],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_on_array_element() {
    // arr[0].method()
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Index {
            array: Box::new(Expression::Identifier {
                name: "arr".to_string(),
                position: pos(1, 1),
            }),
            index: Box::new(Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            }),
            position: pos(1, 1),
        }),
        method: "method".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_on_literal() {
    // "hello".upcase()
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::StringLiteral {
            value: "hello".to_string(),
            position: pos(1, 1),
        }),
        method: "upcase".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_returning_callable() {
    // obj.get_function()(args)
    let expr = Expression::Call {
        callee: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "obj".to_string(),
                position: pos(1, 1),
            }),
            method: "get_function".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 1),
        }),
        arguments: vec![Expression::Identifier {
            name: "args".to_string(),
            position: pos(1, 20),
        }],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

// Tests for SelfExpr

#[test]
fn test_self_expression() {
    let expr = Expression::SelfExpr {
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(!expr.is_literal());
    assert!(!expr.is_identifier());
}

#[test]
fn test_method_call_on_self() {
    // self.method()
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::SelfExpr {
            position: pos(1, 1),
        }),
        method: "method".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_self_in_instance_variable_assignment() {
    // In a method: @name = self.name
    let stmt = Statement::Assignment {
        target: Expression::InstanceVariable {
            name: "name".to_string(),
            position: pos(1, 1),
        },
        value: Expression::MethodCall {
            receiver: Box::new(Expression::SelfExpr {
                position: pos(1, 9),
            }),
            method: "name".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 9),
        },
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_chained_method_on_self() {
    // self.foo().bar()
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::SelfExpr {
                position: pos(1, 1),
            }),
            method: "foo".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 1),
        }),
        method: "bar".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

// Tests for mixed scenarios

#[test]
fn test_nested_property_access() {
    // obj.property.nested_property
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "obj".to_string(),
                position: pos(1, 1),
            }),
            method: "property".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 1),
        }),
        method: "nested_property".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_chain_with_arguments() {
    // obj.method1(x).method2(y).method3(z)
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "obj".to_string(),
                    position: pos(1, 1),
                }),
                method: "method1".to_string(),
                arguments: vec![Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(1, 13),
                }],
                trailing_block: None,
                position: pos(1, 1),
            }),
            method: "method2".to_string(),
            arguments: vec![Expression::Identifier {
                name: "y".to_string(),
                position: pos(1, 25),
            }],
            trailing_block: None,
            position: pos(1, 1),
        }),
        method: "method3".to_string(),
        arguments: vec![Expression::Identifier {
            name: "z".to_string(),
            position: pos(1, 37),
        }],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call_in_binary_operation() {
    // obj.value() + 10
    let expr = Expression::BinaryOp {
        op: BinaryOp::Add,
        left: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "obj".to_string(),
                position: pos(1, 1),
            }),
            method: "value".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 1),
        }),
        right: Box::new(Expression::IntLiteral {
            value: 10,
            position: pos(1, 15),
        }),
        position: pos(1, 13),
    };
    assert_eq!(expr.position(), pos(1, 13));
}

#[test]
fn test_array_of_method_calls() {
    // [obj1.method(), obj2.method(), obj3.method()]
    let expr = Expression::Array {
        elements: vec![
            Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "obj1".to_string(),
                    position: pos(1, 2),
                }),
                method: "method".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(1, 2),
            },
            Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "obj2".to_string(),
                    position: pos(1, 17),
                }),
                method: "method".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(1, 17),
            },
            Expression::MethodCall {
                receiver: Box::new(Expression::Identifier {
                    name: "obj3".to_string(),
                    position: pos(1, 32),
                }),
                method: "method".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(1, 32),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}
