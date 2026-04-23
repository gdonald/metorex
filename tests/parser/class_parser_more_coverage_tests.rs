// Additional coverage tests for src/parser/statements/class.rs.

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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse();
    match stmts {
        Err(es) => es[0].to_string(),
        Ok(stmts) => {
            let mut vm = VirtualMachine::new();
            vm.execute_program(&stmts).unwrap_err().to_string()
        }
    }
}

// ── Deeply nested `::` namespace in class path (lines 52-55) ──────────────

#[test]
fn deeply_nested_namespace_in_class_def() {
    // More than two `::` levels so ns_expr gets built up as a ScopeResolution
    // chain before the final segment becomes `name`.
    let result = run(r#"
module A
  module B
    module C
    end
  end
end
class A::B::C::MyLeafClass
  def hi
    "deep"
  end
end
A::B::C::MyLeafClass.new.hi
"#);
    assert_eq!(result, Some(Object::string("deep")));
}

// ── class body with trailing whitespace before end (line 104 break) ───────

#[test]
fn class_body_with_trailing_whitespace_before_end() {
    // Trailing whitespace/newlines before `end` — the inner `if check(End)
    // break` at line 103-105 handles this.
    let result = run(r#"
class TrailingWS
  def hi
    1
  end


end
TrailingWS.new.hi
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── module body with trailing whitespace before end (line 180 break) ──────

#[test]
fn module_body_with_trailing_whitespace_before_end() {
    let result = run(r#"
module ModTrail
  def self.hi
    2
  end


end
ModTrail.hi
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── singleton class body with trailing whitespace (line 139 break) ────────

#[test]
fn singleton_class_body_with_trailing_whitespace() {
    let result = run(r#"
class SClass
  class << self
    def hi
      3
    end


  end
end
SClass.hi
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── alias with keyword method names (lines 270-278) ──────────────────────

#[test]
fn alias_with_include_keyword() {
    let result = run(r#"
class AlInc
  def include
    "inc"
  end
  alias my_inc include
end
AlInc.new.my_inc
"#);
    assert_eq!(result, Some(Object::string("inc")));
}

#[test]
fn alias_with_extend_keyword() {
    let result = run(r#"
class AlExt
  def extend
    "ext"
  end
  alias my_ext extend
end
AlExt.new.my_ext
"#);
    assert_eq!(result, Some(Object::string("ext")));
}

#[test]
fn alias_with_class_keyword() {
    // alias `my_cls` -> `class` (overridden method)
    let result = run(r#"
class AlCls
  def class
    "cls_override"
  end
  alias my_cls class
end
AlCls.new.my_cls
"#);
    assert_eq!(result, Some(Object::string("cls_override")));
}

#[test]
fn alias_with_module_keyword() {
    let result = run(r#"
class AlMod
  def module
    "mod"
  end
  alias my_mod module
end
AlMod.new.my_mod
"#);
    assert_eq!(result, Some(Object::string("mod")));
}

// `def if`, `def else`, `def do`, `def def`, `def end` aren't accepted at
// the parser level (keywords can't follow `def`). So we use `define_method`
// to install a method under those names, then `alias <new> <keyword>` so
// parse_alias_method_name's keyword arm runs.

#[test]
fn alias_with_def_keyword() {
    let result = run(r#"
class AlDef
  define_method(:def) { "dd" }
  alias my_def def
end
AlDef.new.my_def
"#);
    assert_eq!(result, Some(Object::string("dd")));
}

#[test]
fn alias_with_end_keyword() {
    let result = run(r#"
class AlEnd
  define_method(:end) { "the_end" }
  alias my_end end
end
AlEnd.new.my_end
"#);
    assert_eq!(result, Some(Object::string("the_end")));
}

#[test]
fn alias_with_if_keyword() {
    let result = run(r#"
class AlIf
  define_method(:if) { "iff" }
  alias my_if if
end
AlIf.new.my_if
"#);
    assert_eq!(result, Some(Object::string("iff")));
}

#[test]
fn alias_with_else_keyword() {
    let result = run(r#"
class AlElse
  define_method(:else) { "elsee" }
  alias my_else else
end
AlElse.new.my_else
"#);
    assert_eq!(result, Some(Object::string("elsee")));
}

#[test]
fn alias_with_do_keyword() {
    let result = run(r#"
class AlDo
  define_method(:do) { "doo" }
  alias my_do do
end
AlDo.new.my_do
"#);
    assert_eq!(result, Some(Object::string("doo")));
}

// ── alias with invalid method name errors ─────────────────────────────────

#[test]
fn alias_with_unexpected_token_errors() {
    let err = run_err(
        r#"
class AlBad
  alias x 123
end
"#,
    );
    assert!(
        err.contains("method name") || err.contains("alias") || err.contains("Expected"),
        "unexpected: {}",
        err
    );
}
