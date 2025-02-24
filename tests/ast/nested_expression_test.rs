use super::helpers::pos;
use metorex::ast::{BinaryOp, Expression};

#[test]
fn test_nested_binary_ops() {
    // (1 + 2) * 3
    let expr = Expression::BinaryOp {
        op: BinaryOp::Multiply,
        left: Box::new(Expression::Grouped {
            expression: Box::new(Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::IntLiteral {
                    value: 1,
                    position: pos(1, 2),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 2,
                    position: pos(1, 6),
                }),
                position: pos(1, 4),
            }),
            position: pos(1, 1),
        }),
        right: Box::new(Expression::IntLiteral {
            value: 3,
            position: pos(1, 11),
        }),
        position: pos(1, 9),
    };
    assert_eq!(expr.position(), pos(1, 9));
}

#[test]
fn test_chained_method_calls() {
    // obj.foo().bar()
    let expr = Expression::MethodCall {
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
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_complex_expression() {
    // array[i].method(x + y)
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Index {
            array: Box::new(Expression::Identifier {
                name: "array".to_string(),
                position: pos(1, 1),
            }),
            index: Box::new(Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 7),
            }),
            position: pos(1, 1),
        }),
        method: "method".to_string(),
        arguments: vec![Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 17),
            }),
            right: Box::new(Expression::Identifier {
                name: "y".to_string(),
                position: pos(1, 21),
            }),
            position: pos(1, 19),
        }],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}
