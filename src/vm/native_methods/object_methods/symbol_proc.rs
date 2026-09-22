// The callable `Symbol#to_proc` builds.

use super::*;

/// Build the block `Symbol#to_proc` returns: `{ |receiver, *args| receiver.send(name, *args) }`.
pub(crate) fn symbol_to_proc_block(
    name: &str,
    position: Position,
) -> crate::object::BlockStatement {
    use crate::ast::{Expression, Statement};

    let call = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: SYMBOL_PROC_RECEIVER.to_string(),
            position,
        }),
        // The call goes out through `public_send`, so a name the receiver
        // keeps to itself is refused rather than reached.
        method: "public_send".to_string(),
        arguments: vec![
            Expression::Symbol {
                value: name.to_string(),
                position,
            },
            Expression::Splat {
                expression: Box::new(Expression::Identifier {
                    name: SYMBOL_PROC_ARGS.to_string(),
                    position,
                }),
                position,
            },
        ],
        trailing_block: None,
        position,
    };

    let mut made = crate::object::BlockStatement::new(
        vec![
            SYMBOL_PROC_RECEIVER.to_string(),
            format!("*{SYMBOL_PROC_ARGS}"),
        ],
        vec![Statement::Expression {
            expression: call,
            position,
        }],
        std::collections::HashMap::new(),
    );
    // Ruby's symbol proc is a lambda, so it counts its arguments strictly.
    made.is_lambda = true;
    // What the callable stands for, which is what it says of itself in place
    // of a file and a line.
    made.from_symbol = Some(name.to_string());
    made
}

/// The names the block `Symbol#to_proc` builds takes. Ruby reports them with
/// no names at all, so the pair is recognized by these.
pub(crate) const SYMBOL_PROC_RECEIVER: &str = "__symbol_proc_receiver";
pub(crate) const SYMBOL_PROC_ARGS: &str = "__symbol_proc_args";

/// Map an operator method name back to its `BinaryOp`, for calls that arrive
/// by name (`send(:+, 2)`) instead of through operator syntax.
pub(crate) fn binary_op_for_method_name(name: &str) -> Option<crate::ast::BinaryOp> {
    use crate::ast::BinaryOp;
    Some(match name {
        "+" => BinaryOp::Add,
        "-" => BinaryOp::Subtract,
        "*" => BinaryOp::Multiply,
        "/" => BinaryOp::Divide,
        "%" => BinaryOp::Modulo,
        "**" => BinaryOp::Power,
        "==" => BinaryOp::Equal,
        "===" => BinaryOp::CaseEqual,
        "!=" => BinaryOp::NotEqual,
        "<" => BinaryOp::Less,
        ">" => BinaryOp::Greater,
        "<=" => BinaryOp::LessEqual,
        ">=" => BinaryOp::GreaterEqual,
        "<=>" => BinaryOp::Spaceship,
        "&" => BinaryOp::BitwiseAnd,
        "|" => BinaryOp::BitwiseOr,
        "^" => BinaryOp::Xor,
        _ => return None,
    })
}
