// The block a call is handed, and the names a method reads inside one.

use super::*;

#[test]
fn symbol_to_proc_in_map() {
    let result = run("[1, 2, 3].map(&:to_s)");
    if let Some(Object::Array(arr)) = &result {
        let items: Vec<_> = arr.borrow().iter().map(|o| format!("{}", o)).collect();
        assert_eq!(items, vec!["1", "2", "3"]);
    } else {
        panic!("expected array");
    }
}

// ── block_arg nil is dropped ─────────────────────────────────────────────────

#[test]
fn block_arg_nil_dropped() {
    let result = run(r#"
def foo(&block)
  block_given?
end
b = nil
foo(&b)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── block_arg with block object ──────────────────────────────────────────────

#[test]
fn block_arg_with_block() {
    let result = run(r#"
def foo(&block)
  block.call
end
b = lambda { 42 }
foo(&b)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── __method__ inside a method with class prefix (lines 64-65) ───────────────

#[test]
fn method_name_inside_method_returns_short_name() {
    let result = run(r#"
class MyClass
  def greet
    __method__()
  end
end
MyClass.new.greet
"#);
    assert_eq!(result, Some(Object::symbol("greet".to_string())));
}

// ── rand with non-Int argument (line 89) ──────────────────────────────────────

#[test]
fn rand_with_a_float_bound_above_one_gives_an_integer() {
    let result = run("rand(3.14).is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_with_an_uncoercible_argument_raises_type_error() {
    let error = run_err(r#"rand("hello")"#);
    assert!(error.contains("no implicit conversion of String into Integer"));
}
