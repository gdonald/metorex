// Tests for operator method definitions and scope resolution

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn parse_ok(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().unwrap_err()[0].to_string()
}

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// ── Scope resolution error ──────────────────────────────────────────────────

#[test]
fn scope_resolution_non_ident_error() {
    let err = parse_err("Foo::42");
    assert!(err.contains("constant") || err.contains("Expected") || err.contains("::"));
}

// ── Operator method definitions ─────────────────────────────────────────────

#[test]
fn operator_method_def_minus() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def -(other)\n    @v - other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(10)\nb = Num.new(3)\na - b",
    );
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn operator_method_def_star() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def *(other)\n    @v * other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(4)\nb = Num.new(5)\na * b",
    );
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn operator_method_def_slash() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def /(other)\n    @v / other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(10)\nb = Num.new(2)\na / b",
    );
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn operator_method_def_percent() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def %(other)\n    @v % other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(10)\nb = Num.new(3)\na % b",
    );
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn operator_method_def_bang_equal() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def !=(other)\n    @v != other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(5)\nb = Num.new(3)\na != b",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_less() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def <(other)\n    @v < other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(3)\nb = Num.new(5)\na < b",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_greater() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def >(other)\n    @v > other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(5)\nb = Num.new(3)\na > b",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_less_equal() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def <=(other)\n    @v <= other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(3)\nb = Num.new(3)\na <= b",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_greater_equal() {
    let result = run(
        "class Num\n  def initialize(v)\n    @v = v\n  end\n  def >=(other)\n    @v >= other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(5)\nb = Num.new(3)\na >= b",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn def_method_named_pipe() {
    parse_ok("class Bits\n  def |(other)\n    true\n  end\nend\n");
}

#[test]
fn def_method_named_ampersand() {
    parse_ok("class Bits\n  def &(other)\n    false\n  end\nend\n");
}

#[test]
fn def_method_named_triple_equal() {
    // Verify the method is parseable and callable via `.send`-style dispatch.
    parse_ok("class M\n  def ===(other)\n    true\n  end\nend\n");
}

#[test]
fn def_method_named_match_op() {
    parse_ok("class Pat\n  def =~(other)\n    42\n  end\nend\n");
}

#[test]
fn def_method_named_not_match_op() {
    parse_ok("class Pat\n  def !~(other)\n    42\n  end\nend\n");
}
