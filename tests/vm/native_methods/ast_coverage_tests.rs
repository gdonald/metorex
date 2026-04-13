// AST method coverage tests — exercises binary_op_str, serialize_expression,
// and serialize_statement branches not yet reached by other tests.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// Returns the "op" field of the first parsed BinaryOp expression.
fn binary_op_of(code: &str) -> String {
    let result = run(&format!("parse({:?})", code));
    match result {
        Some(Object::Array(arr)) => {
            let arr = arr.borrow();
            if let Some(Object::Dict(d)) = arr.first() {
                let d = d.borrow();
                if let Some(Object::String(s)) = d.get("op") {
                    return s.as_ref().clone();
                }
            }
            "no_op".to_string()
        }
        _ => "none".to_string(),
    }
}

// ── binary_op_str: Power (**) ─────────────────────────────────────────────────

#[test]
fn parse_binary_op_power() {
    assert_eq!(binary_op_of("2 ** 3"), "**");
}

// ── binary_op_str: BitwiseAnd (&) ─────────────────────────────────────────────

#[test]
fn parse_binary_op_bitwise_and() {
    assert_eq!(binary_op_of("2 & 3"), "&");
}

// ── binary_op_str: BitwiseOr (|) ──────────────────────────────────────────────

#[test]
fn parse_binary_op_bitwise_or() {
    assert_eq!(binary_op_of("2 | 3"), "|");
}

// ── binary_op_str: Xor (^) ────────────────────────────────────────────────────

#[test]
fn parse_binary_op_xor() {
    assert_eq!(binary_op_of("2 ^ 3"), "^");
}

// ── binary_op_str: And (&&) ───────────────────────────────────────────────────

#[test]
fn parse_binary_op_and() {
    assert_eq!(binary_op_of("a && b"), "&&");
}

// ── binary_op_str: Or (||) ────────────────────────────────────────────────────

#[test]
fn parse_binary_op_or() {
    assert_eq!(binary_op_of("a || b"), "||");
}

// ── serialize_statement: Continue (next keyword) ─────────────────────────────

#[test]
fn parse_next_statement_is_continue() {
    // parse a loop containing `next`; the while body's first element should
    // serialize as {"type": "Continue"}.
    let result = run(r#"parse("i = 0\nwhile i < 1\n  next\n  i += 1\nend")"#);
    match result {
        Some(Object::Array(arr)) => {
            let arr = arr.borrow();
            // Second stmt is the while loop
            if let Some(Object::Dict(while_d)) = arr.get(1) {
                let while_d = while_d.borrow();
                if let Some(Object::Array(body)) = while_d.get("body") {
                    let body = body.borrow();
                    if let Some(Object::Dict(first_stmt)) = body.first() {
                        let first_stmt = first_stmt.borrow();
                        if let Some(Object::String(ty)) = first_stmt.get("type") {
                            assert_eq!(ty.as_ref(), "Continue");
                            return;
                        }
                    }
                }
            }
            panic!("Could not find Continue in parsed while body");
        }
        other => panic!("expected Array from parse(), got {:?}", other),
    }
}
