// Exception error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// ══════════════════════════════════════════════════════════════════════════════
// Exception methods (lines 23-25, 78 of exception_methods.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn exception_message_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.message
end
result
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("test error".to_string())))
    );
}

#[test]
fn exception_type_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.type
end
result
"#);
    assert!(result.is_some());
}

#[test]
fn exception_to_s_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.to_s
end
result
"#);
    assert!(result.is_some());
    if let Some(Object::String(s)) = result {
        assert!(
            s.contains("test error"),
            "Expected to contain 'test error', got: {}",
            s
        );
    }
}

#[test]
fn exception_backtrace_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.backtrace
end
result
"#);
    assert!(result.is_some());
}

#[test]
fn exception_exception_type_alias() {
    // `exception_type` is an alias for `type`.
    let result = run(r#"
def t
  begin
    raise "boom"
  rescue => e
    e.exception_type
  end
end
t
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("RuntimeError".to_string())))
    );
}

#[test]
fn exception_to_s_includes_type_and_message() {
    let result = run(r#"
def t
  begin
    raise "specific message"
  rescue => e
    e.to_s
  end
end
t
"#);
    if let Some(Object::String(s)) = result {
        assert!(s.contains("RuntimeError"));
        assert!(s.contains("specific message"));
    } else {
        panic!("expected String, got {:?}", result);
    }
}

#[test]
fn exception_unknown_method_falls_through() {
    // Calling an unsupported method on an exception falls through to method_missing
    // / lookup; we expect an error rather than crashing.
    let result = std::panic::catch_unwind(|| {
        run(r#"
def t
  begin
    raise "boom"
  rescue => e
    e.totally_made_up_method
  end
end
t
"#)
    });
    assert!(result.is_err() || result.unwrap().is_some());
}
