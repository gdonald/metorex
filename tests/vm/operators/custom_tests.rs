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

// ── Custom operator methods ─────────────────────────────────────────────────

#[test]
fn custom_operator_divide() {
    assert_eq!(
        run(
            "class Num\n  def initialize(v)\n    @v = v\n  end\n  def /(other)\n    @v / other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(10)\nb = Num.new(2)\na / b"
        ),
        Some(Object::Int(5))
    );
}

#[test]
fn custom_operator_modulo() {
    assert_eq!(
        run(
            "class Num\n  def initialize(v)\n    @v = v\n  end\n  def %(other)\n    @v % other.val\n  end\n  def val\n    @v\n  end\nend\na = Num.new(10)\nb = Num.new(3)\na % b"
        ),
        Some(Object::Int(1))
    );
}

#[test]
fn custom_operator_equal_equal() {
    assert_eq!(
        run(
            "class V\n  def initialize(v)\n    @v = v\n  end\n  def ==(other)\n    @v == other.val\n  end\n  def val\n    @v\n  end\nend\nV.new(5) == V.new(5)"
        ),
        Some(Object::Bool(true))
    );
}

// ── Chained ternary ─────────────────────────────────────────────────────────

#[test]
fn chained_ternary_with_methods() {
    let result = run(r#"
a = "hello"
x = a.length > 10 ? "long" : a.length > 3 ? "medium" : "short"
x
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("medium".to_string())))
    );
}
