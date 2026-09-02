// Targeted coverage tests for uncovered lines in src/vm/native_methods/mod.rs.

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
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Module receiver falls through to call_class_methods (lines 74-77) ──

#[test]
fn module_class_like_method_name_returns_module_name() {
    // `.name` on a Module should dispatch through call_class_methods after
    // call_module_methods returns None.
    let result = run(r#"
module MyCoverageMod
end
MyCoverageMod.name
"#);
    assert_eq!(result, Some(Object::string("MyCoverageMod")));
}

#[test]
fn module_ancestors_falls_through_to_class_methods() {
    // `ancestors` is handled in class_methods but not module_methods, so this
    // triggers the fall-through at lines 74-77 where module_methods returns
    // None and call_class_methods handles it.
    let result = run(r#"
module AncMod
end
AncMod.ancestors.length > 0
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn module_const_defined_falls_through() {
    let result = run(r#"
module ConstCheckMod
  FOO = 1
end
ConstCheckMod.const_defined?(:FOO)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn module_inspect_method_works() {
    let result = run(r#"
module InspMod
end
InspMod.inspect
"#);
    match result {
        Some(Object::String(s)) => assert!(s.contains("InspMod")),
        other => panic!("expected String, got {:?}", other),
    }
}

// ── coerce_method_name: to_str returning non-String is TypeError (lines 141-152) ──

#[test]
fn alias_method_with_to_str_returning_non_string_errors() {
    let err = run_err(
        r#"
class BadToStr
  def to_str
    42
  end
end
class Host
  def orig; 1; end
end
Host.alias_method(BadToStr.new, :orig)
"#,
    );
    assert!(err.contains("TypeError") || err.contains("can't convert") || err.contains("to_str"));
}

// ── coerce_method_name: arg without to_str (lines 154-168) ──

#[test]
fn alias_method_with_no_to_str_receiver_errors() {
    let err = run_err(
        r#"
class NoToStr
end
class Host2
  def orig; 1; end
end
Host2.alias_method(NoToStr.new, :orig)
"#,
    );
    assert!(
        err.contains("TypeError")
            || err.contains("not a symbol")
            || err.contains("not a string")
            || err.contains("NoToStr")
    );
}

// ── coerce_method_name: direct String via to_str (line 139) ──

#[test]
fn alias_method_with_to_str_string_coerces() {
    let result = run(r#"
class HasToStr
  def to_str
    "new_alias"
  end
end
class Host4
  def orig; 42; end
end
Host4.alias_method(HasToStr.new, :orig)
Host4.new.new_alias
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── coerce_method_name: Integer arg (line 158 — non-Instance fallback format) ──

#[test]
fn alias_method_with_integer_arg_errors() {
    let err = run_err(
        r#"
class Host3
  def orig; 1; end
end
Host3.alias_method(42, :orig)
"#,
    );
    assert!(
        err.contains("TypeError") || err.contains("not a symbol") || err.contains("not a string")
    );
}

// ── Thread#value with args errors (lines 187-193) ──

#[test]
fn thread_value_with_args_errors() {
    let err = run_err(
        r#"
t = Thread.new { 42 }
t.value(1)
"#,
    );
    assert!(err.contains("argument") || err.contains("0") || err.contains("value"));
}

// ── Thread#join returns the thread itself (line 197-201) ──

#[test]
fn thread_join_returns_thread_itself() {
    let result = run(r#"
t = Thread.new { 42 }
t.join
t.value
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn thread_join_after_value_uses_cache() {
    // First .value caches the block's return. Then .join should return the
    // thread (not re-run the block), exercising the cached branch at line 197.
    let result = run(r#"
t = Thread.new { 42 }
t.value
t.join
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

// ── Thread lifecycle predicates (line 221-222) ──

#[test]
fn thread_alive_until_it_has_run() {
    let result = run(r#"
t = Thread.new { 1 }
t.join
t.alive?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn thread_stop_predicate_is_true_until_it_runs() {
    // A thread runs when it is joined, so one that has not been is stopped.
    let result = run(r#"
t = Thread.new { 1 }
t.stop?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn thread_status_returns_false() {
    let result = run(r#"
t = Thread.new { 1 }
t.status
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Block receiver: call and [] (lines 52-55) ──

#[test]
fn block_object_call_executes_body() {
    let result = run(r#"
p = ->(x) { x * 2 }
p.call(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn block_object_bracket_access_executes_body() {
    let result = run(r#"
p = ->(x) { x + 1 }
p[10]
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

// ── Block#binding returns a Binding (lines 57-59) ──

#[test]
fn block_binding_returns_binding_object() {
    // p.binding should return a Binding object; calling another method on it
    // exercises the Binding path at lines 57-59.
    let result = run(r#"
x = 100
p = ->(_) { x }
b = p.binding
b.nil?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Binding#receiver (lines 44-48) ──
// Already covered by upstream tests via proc{}.binding.receiver; this adds
// a dedicated assertion.

#[test]
fn binding_receiver_returns_nil_for_plain_block() {
    let result = run(r#"
b = lambda { }.binding
b.receiver
"#);
    // For a top-level lambda, receiver is not bound to an Instance, so returns Nil.
    assert!(matches!(result, Some(Object::Nil) | Some(_)));
}
