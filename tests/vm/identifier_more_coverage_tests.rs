// Additional targeted coverage tests for src/vm/eval/identifier.rs.

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

// ── Uppercase constant resolved via globals fallback (line 45) ──────────────

#[test]
fn uppercase_constant_in_globals_resolves() {
    // Register a global constant directly and look it up as a bare identifier.
    let result = run(r#"
GLOBAL_TEST = 999
GLOBAL_TEST
"#);
    assert_eq!(result, Some(Object::Int(999)));
}

// ── Constant inherited from superclass (line 90) ────────────────────────────

#[test]
fn constant_inherited_from_parent_class_via_chain() {
    let result = run(r#"
class ParentConst
  INHERITED_VAL = 42
end
class ChildConst < ParentConst
  def fetch
    INHERITED_VAL
  end
end
ChildConst.new.fetch
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Constant resolved through enclosing def_scope_stack (lines 97-102) ──────

#[test]
fn constant_in_enclosing_module_from_class_body() {
    // OUTER_CONST is defined in Outer; Inner references it unqualified.
    let result = run(r#"
module OuterScope
  OUTER_CONST = 7
  class Inner
    INNER_VAL = OUTER_CONST * 2
  end
end
OuterScope::Inner::INNER_VAL
"#);
    assert_eq!(result, Some(Object::Int(14)));
}

#[test]
fn enclosing_module_name_resolves_to_self() {
    // Before `ModX` is fully bound in globals, `ModX` inside its own body
    // should still resolve to the module itself (lines 101-102).
    let result = run(r#"
module ModX
  SELF_REF = ModX
end
ModX::SELF_REF == ModX
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Bare native method dispatch (lines 150-154) ─────────────────────────────

#[test]
fn bare_identifier_dispatches_to_native_class_method() {
    // Bare `class` inside instance method should call native class method.
    let result = run(r#"
class HasClass
  def my_class
    self.class.name
  end
end
HasClass.new.my_class
"#);
    assert_eq!(result, Some(Object::string("HasClass")));
}

// ── Constant from Object class fallback (lines 106-109) ─────────────────────

#[test]
fn constant_on_object_class_accessible_anywhere() {
    let result = run(r#"
class Object
  MY_OBJECT_CONST = 100
end
class Somewhere
  def fetch
    MY_OBJECT_CONST
  end
end
Somewhere.new.fetch
"#);
    assert_eq!(result, Some(Object::Int(100)));
}
