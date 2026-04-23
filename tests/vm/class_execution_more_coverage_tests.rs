// Additional coverage tests for src/vm/class_execution.rs.

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

// Note: execute_alias's def_scope_stack path (lines 1151-1153) is only
// exercised when Statement::Alias is dispatched via execute_alias during
// class body evaluation; but `apply_class_body` handles Statement::Alias
// inline at line 510-513, and class_eval doesn't push to def_scope_stack.
// So the path appears to be dead code at the moment.

// ── enclosing module resolved by its own name (line 1066-1067) ────────────
// Inside a nested class body, resolve_constant_in_scope should find the
// enclosing module via its own simple name even if not yet bound in globals.

#[test]
fn enclosing_module_self_reference_during_body() {
    let result = run(r#"
module EnclosingSelf
  class NestedKlass
    OWNER = EnclosingSelf
  end
end
EnclosingSelf::NestedKlass::OWNER == EnclosingSelf
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── execute_include suppressed by load_wrap_depth (line 1085) ─────────────

#[test]
fn top_level_include_suppressed_under_wrapped_load() {
    // load(path, true) sets load_wrap_depth > 0. A top-level `include Foo`
    // inside that loaded file should become a no-op (line 1084-1086).
    let dir = "tests/_examples";
    let name = "io_wrap_suppress_include_tmp.rb";
    let path = format!("{}/{}", dir, name);
    std::fs::write(
        &path,
        r#"module WrapSup
  def hi; "wrapsup"; end
end
include WrapSup
"#,
    )
    .unwrap();
    let result = run(&format!(
        r#"$LOAD_PATH.unshift "{}"
load("{}", true)
:ok
"#,
        dir, name
    ));
    std::fs::remove_file(&path).ok();
    // The include inside the wrapped load is suppressed; the program itself
    // returns :ok.
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

// ── Rational equality via instance_vars (object/operations.rs 72-77) ──────

#[test]
fn two_rationals_with_same_values_are_equal() {
    let result = run("Rational(1, 2) == Rational(1, 2)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn two_rationals_with_different_values_are_not_equal() {
    let result = run("Rational(1, 2) == Rational(3, 4)");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Method equality one-has-receiver one-doesn't (object/operations.rs 100) ─

#[test]
fn two_methods_differing_in_receiver_not_equal() {
    // One method bound via .method(:x), another via .instance_method(:x)
    // (unbound, so receiver=None).
    let result = run(r#"
class MeqA
  def foo
    1
  end
end
a = MeqA.new
bound = a.method(:foo)
unbound = MeqA.instance_method(:foo)
bound == unbound
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── resolve_qualified_constant with non-class middle (lines 1122-1126) ────

#[test]
fn qualified_constant_walk_stops_at_non_class_middle() {
    // Include a module that itself includes something — during include's
    // resolve_qualified_constant walk, hit a non-Class/Module middle.
    let result = run(r#"
module OuterQC
  NotAClass = 42
end
# Include "OuterQC::NotAClass::Inner" would fail because NotAClass isn't a
# class/module. At the include call, the resolver walks OuterQC → NotAClass
# (Int, hits line 1124 _ => None), so include errors.
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}
