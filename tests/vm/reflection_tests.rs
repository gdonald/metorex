// Tests for reflection and introspection (14.4)

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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── instance_of? ────────────────────────────────────────────────────────

#[test]
fn instance_of_exact_class() {
    let result = run(r#"
class Foo
end
Foo.new.instance_of?(Foo)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_of_parent_class_is_false() {
    let result = run(r#"
class Parent
end
class Child < Parent
end
Child.new.instance_of?(Parent)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn instance_of_on_builtin() {
    let result = run("42.instance_of?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_of_wrong_arg_type() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.instance_of?("Foo")
"#,
    );
    assert!(err.contains("Class"));
}

#[test]
fn instance_of_wrong_arg_count() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.instance_of?()
"#,
    );
    assert!(err.contains("expected 1"));
}

// ── methods ─────────────────────────────────────────────────────────────

#[test]
fn methods_returns_array() {
    let result = run(r#"
class Foo
  def bar
    1
  end
  def baz
    2
  end
end
Foo.new.methods
"#);
    if let Some(Object::Array(arr)) = result {
        let names: Vec<String> = arr.borrow().iter().map(|o| o.to_string()).collect();
        assert!(names.contains(&":bar".to_string()));
        assert!(names.contains(&":baz".to_string()));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn methods_includes_inherited() {
    let result = run(r#"
class Parent
  def greet
    "hi"
  end
end
class Child < Parent
  def wave
    "wave"
  end
end
Child.new.methods
"#);
    if let Some(Object::Array(arr)) = result {
        let names: Vec<String> = arr.borrow().iter().map(|o| o.to_string()).collect();
        assert!(names.contains(&":greet".to_string()));
        assert!(names.contains(&":wave".to_string()));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn methods_on_builtin_returns_array() {
    let result = run(r#"[1, 2].methods"#);
    if let Some(Object::Array(arr)) = result {
        assert!(!arr.borrow().is_empty());
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn methods_wrong_arg_count() {
    // Ruby's `Object#methods` accepts an optional `include_super` Boolean;
    // passing an extra second positional arg is the error case now.
    let err = run_err(
        r#"
class Foo
end
Foo.new.methods(true, "extra")
"#,
    );
    assert!(err.contains("expected 1"));
}

// ── send ────────────────────────────────────────────────────────────────

#[test]
fn send_calls_method_by_string() {
    let result = run(r#"
class Foo
  def bar
    42
  end
end
Foo.new.send("bar")
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn send_calls_method_by_symbol() {
    let result = run(r#"
class Foo
  def bar
    42
  end
end
Foo.new.send(:bar)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn send_passes_arguments() {
    let result = run(r#"
class Calc
  def add(a, b)
    a + b
  end
end
Calc.new.send("add", 3, 4)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn send_on_builtin_string() {
    let result = run(r#""hello".send("upcase")"#);
    assert_eq!(result, Some(Object::String(Rc::new("HELLO".to_string()))));
}

#[test]
fn send_on_builtin_array() {
    let result = run(r#"[1, 2, 3].send("length")"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn send_nonexistent_method() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.send("nonexistent")
"#,
    );
    assert!(err.contains("Undefined method"));
}

#[test]
fn send_wrong_arg_type() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.send(123)
"#,
    );
    assert!(err.contains("not a symbol nor a string"));
}

#[test]
fn send_no_args() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.send()
"#,
    );
    assert!(err.contains("no method name given"));
}

#[test]
fn send_calls_native_to_s() {
    let result = run(r#"42.send("to_s")"#);
    assert_eq!(result, Some(Object::String(Rc::new("42".to_string()))));
}

#[test]
fn send_calls_class_method() {
    let result = run(r#"
class Dog
end
d = Dog.new
d.send("class")
"#);
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Dog");
    } else {
        panic!("Expected Class");
    }
}
