// Method object error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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
// AST methods - method body access (lines 43-47 of mod.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn method_object_body_access() {
    let result = run(r#"
def greet
  "hello"
end
m = method(:greet)
m.body
"#);
    assert!(result.is_some());
}

#[test]
fn method_object_name_access() {
    let result = run(r#"
def greet
  "hello"
end
m = method(:greet)
m.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("greet".to_string()))));
}

#[test]
fn method_object_arity_access() {
    let result = run(r#"
def add(a, b)
  a + b
end
m = method(:add)
m.arity
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn method_object_parameters_access() {
    let result = run(r#"
def add(a, b)
  a + b
end
m = method(:add)
m.parameters
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Method object - owner and source_location
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn method_object_owner_access() {
    let result = run(r#"
def my_func
  1
end
m = method(:my_func)
m.owner
"#);
    assert!(result.is_some());
}

#[test]
fn method_object_source_location_access() {
    let result = run(r#"
def my_func
  1
end
m = method(:my_func)
m.source_location
"#);
    assert!(result.is_some());
}
