// Targeted coverage tests for uncovered lines in src/parser/statements/class.rs.

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

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    match Parser::new(tokens).parse() {
        Ok(_) => String::new(),
        Err(errs) => format!("{:?}", errs),
    }
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── `class Foo::Bar::Baz` with nested namespace (lines 34-62) ────────────────

#[test]
fn class_with_deep_namespace() {
    let result = run(r#"
module A
  module B
    class C
      def hi
        "deep"
      end
    end
  end
end
A::B::C.new.hi
"#);
    assert_eq!(result, Some(Object::string("deep")));
}

// ── `class Foo::` with missing name errors (lines 44-46) ─────────────────────

#[test]
fn class_namespace_missing_name_errors() {
    let err = parse_err(
        r#"
class Foo::
end
"#,
    );
    assert!(err.contains("constant") || err.contains("Expected"));
}

// ── `class Foo < Bar::` with missing name errors (lines 83-86) ───────────────

#[test]
fn class_superclass_missing_final_segment_errors() {
    let err = parse_err(
        r#"
class Foo < Bar::
end
"#,
    );
    assert!(err.contains("constant") || err.contains("Expected"));
}

// ── `class <` without a name errors (lines 72-74) ────────────────────────────

#[test]
fn class_superclass_missing_name_errors() {
    let err = parse_err(
        r#"
class Foo <
end
"#,
    );
    assert!(err.contains("superclass") || err.contains("Expected"));
}

// ── Fully qualified superclass (lines 76-88) ─────────────────────────────────

#[test]
fn class_with_qualified_superclass() {
    let result = run(r#"
module Mns
  class Base
    def root
      "root"
    end
  end
end
class Child < Mns::Base
end
Child.new.root
"#);
    assert_eq!(result, Some(Object::string("root")));
}

// ── `class` with no name errors (lines 30-32) ────────────────────────────────

#[test]
fn class_no_name_errors() {
    let err = parse_err(
        r#"
class
end
"#,
    );
    assert!(err.contains("class") || err.contains("Expected"));
}

// ── Missing `end` errors (line 104) ─────────────────────────────────────────

#[test]
fn class_missing_end_errors() {
    let err = parse_err(
        r#"
class NoEnd
"#,
    );
    assert!(err.contains("end") || err.contains("Expected") || err.contains("Unexpected"));
}

// ── `class <<` without target (line 139) ────────────────────────────────────

#[test]
fn singleton_class_missing_end_errors() {
    let err = parse_err(
        r#"
class << self
"#,
    );
    assert!(err.contains("end") || err.contains("Expected") || err.contains("Unexpected"));
}

// ── `module ::Name` (lines 162-164) ──────────────────────────────────────────

#[test]
fn module_with_top_level_leading_colons() {
    let result = run(r#"
module ::TopLevelMod
  def hi
    "tlm"
  end
end
class UserTLM
  include TopLevelMod
end
UserTLM.new.hi
"#);
    assert_eq!(result, Some(Object::string("tlm")));
}

// ── `module` with no name errors (line 168) ─────────────────────────────────

#[test]
fn module_no_name_errors() {
    let err = parse_err(
        r#"
module
end
"#,
    );
    assert!(err.contains("module") || err.contains("Expected"));
}

// ── Module missing end errors (line 180) ─────────────────────────────────────

#[test]
fn module_missing_end_errors() {
    let err = parse_err(
        r#"
module NoEnd
"#,
    );
    assert!(err.contains("end") || err.contains("Expected") || err.contains("Unexpected"));
}

// ── `include` qualified name (lines 208-218) ─────────────────────────────────

#[test]
fn include_qualified_name() {
    let result = run(r#"
module Outer
  module Inner
    def hi_qualified
      "qual"
    end
  end
end
class UserQ
  include Outer::Inner
end
UserQ.new.hi_qualified
"#);
    assert_eq!(result, Some(Object::string("qual")));
}

#[test]
fn include_qualified_missing_segment_errors() {
    let err = parse_err(
        r#"
class Host
  include Foo::
end
"#,
    );
    assert!(err.contains("constant") || err.contains("Expected"));
}

// ── alias with keyword method names (lines 270-279) ──────────────────────────

#[test]
fn alias_with_symbol_literal() {
    let result = run(r#"
class S1
  def orig
    1
  end
  alias :new_name :orig
end
S1.new.new_name
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn alias_class_keyword() {
    let result = run(r#"
class S2
  def my_class
    "mine"
  end
  alias :class :my_class
end
S2.new.class
"#);
    assert_eq!(result, Some(Object::string("mine")));
}

#[test]
fn alias_bad_method_name_errors() {
    let err = parse_err(
        r#"
class S3
  alias 123 456
end
"#,
    );
    assert!(err.contains("method name") || err.contains("Expected"));
}

// ── `extend` with no name errors (line 236) ──────────────────────────────────

#[test]
fn extend_no_name_errors() {
    let err = parse_err(
        r#"
class Host
  extend
end
"#,
    );
    assert!(err.contains("module") || err.contains("Expected"));
}

// ── bare `include` reports the missing argument at runtime ───────────────────

#[test]
fn include_no_name_errors() {
    let err = run_err(
        r#"
class Host
  include
end
"#,
    );
    assert!(
        err.contains("wrong number of arguments (given 0, expected 1+)"),
        "unexpected error: {err}"
    );
}

// ── Undefined module for extend (via runtime) ────────────────────────────────

#[test]
fn extend_undefined_module_errors() {
    let err = run_err(
        r#"
class ExtMissing
  extend NoSuchModule
end
"#,
    );
    assert!(err.contains("NoSuchModule") || err.contains("Undefined") || err.contains("module"));
}
