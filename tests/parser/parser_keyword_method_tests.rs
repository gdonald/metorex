// Tests for using keywords as method names in method calls

use metorex::lexer::Lexer;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

#[test]
fn keyword_method_name_if() {
    let err = run_err("nil.if");
    assert!(err.contains("if") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_def() {
    let err = run_err("nil.def");
    assert!(err.contains("def") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_end() {
    let err = run_err("nil.end");
    assert!(err.contains("end") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_do() {
    let err = run_err("nil.do");
    assert!(err.contains("do") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_else() {
    let err = run_err("nil.else");
    assert!(err.contains("else") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_elsif() {
    let err = run_err("nil.elsif");
    assert!(err.contains("elsif") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_unless() {
    let err = run_err("nil.unless");
    assert!(err.contains("unless") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_while() {
    let err = run_err("nil.while");
    assert!(err.contains("while") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_for() {
    let err = run_err("nil.for");
    assert!(err.contains("for") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_in() {
    let err = run_err("nil.in");
    assert!(err.contains("in") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_begin() {
    let err = run_err("nil.begin");
    assert!(err.contains("begin") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_rescue() {
    let err = run_err("nil.rescue");
    assert!(err.contains("rescue") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_ensure() {
    let err = run_err("nil.ensure");
    assert!(err.contains("ensure") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_raise() {
    let err = run_err("nil.raise");
    assert!(err.contains("raise") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_break() {
    let err = run_err("nil.break");
    assert!(err.contains("break") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_continue() {
    let err = run_err("nil.continue");
    assert!(err.contains("continue") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_return() {
    let err = run_err("nil.return");
    assert!(err.contains("return") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_lambda() {
    let err = run_err("nil.lambda");
    assert!(err.contains("lambda") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_super() {
    let err = run_err("nil.super");
    assert!(err.contains("super") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_case() {
    let err = run_err("nil.case");
    assert!(err.contains("case") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_when() {
    let err = run_err("nil.when");
    assert!(err.contains("when") || err.contains("method") || err.contains("nil"));
}
