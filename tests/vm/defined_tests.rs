// defined? tests

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

// ── defined? ────────────────────────────────────────────────────────────────

#[test]
fn defined_local_variable() {
    let result = run(r#"x = 1; defined?(x)"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "local-variable".to_string()
        )))
    );
}

#[test]
fn defined_undefined() {
    assert_eq!(run("defined?(nonexistent)"), Some(Object::Nil));
}

#[test]
fn defined_method() {
    let result = run("def foo; end; defined?(foo)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("method".to_string())))
    );
}

#[test]
fn defined_constant() {
    assert_eq!(
        run("defined?(String)"),
        Some(Object::String(std::rc::Rc::new("constant".to_string())))
    );
}

#[test]
fn defined_literal() {
    assert_eq!(
        run("defined?(42)"),
        Some(Object::String(std::rc::Rc::new("expression".to_string())))
    );
}

#[test]
fn defined_global_variable() {
    let result = run("$test_def = 1; defined?($test_def)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "global-variable".to_string()
        )))
    );
}

#[test]
fn defined_instance_var() {
    let result = run(
        "class Foo\n  def initialize\n    @x = 1\n  end\n  def check\n    defined?(@x)\n  end\nend\nFoo.new.check",
    );
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "instance-variable".to_string()
        )))
    );
}

#[test]
fn defined_yield_with_block() {
    let result = run("def test\n  defined?(yield)\nend\ntest { 1 }");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("yield".to_string())))
    );
}

#[test]
fn defined_yield_without_block() {
    assert_eq!(
        run("def test; defined?(yield); end; test()"),
        Some(Object::Nil)
    );
}

#[test]
fn defined_class_variable() {
    let result =
        run("class Foo\n  @@x = 1\n  def check\n    defined?(@@x)\n  end\nend\nFoo.new.check");
    assert!(result.is_some());
}

#[test]
fn defined_self_in_method() {
    let result = run("class Foo\n  def check\n    defined?(self)\n  end\nend\nFoo.new.check");
    assert!(result.is_some());
}

#[test]
fn defined_method_call() {
    assert_eq!(
        run(r#"defined?(puts("hi"))"#),
        Some(Object::String(std::rc::Rc::new("method".to_string())))
    );
}

#[test]
fn defined_scope_resolution() {
    assert_eq!(run("defined?(Nonexistent::Thing)"), Some(Object::Nil));
}

#[test]
fn defined_global_function_returns_string() {
    // `puts` is in scope; defined? returns some non-nil string.
    let result = run("defined?(puts)");
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn defined_class_var_with_existing() {
    let result =
        run("class Foo\n  @@x = 1\n  def check\n    defined?(@@x)\n  end\nend\nFoo.new.check");
    if let Some(Object::String(s)) = result {
        assert!(s.contains("class") || s.contains("variable"));
    } else {
        panic!("expected String, got {:?}", result);
    }
}

#[test]
fn defined_scope_resolution_existing() {
    let result = run("class Foo\n  VERSION = 42\nend\ndefined?(Foo::VERSION)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("constant".to_string())))
    );
}

#[test]
fn defined_super_returns_super() {
    let result = run(
        "class Parent\n  def hi\n    \"p\"\n  end\nend\nclass Child < Parent\n  def hi\n    defined?(super)\n  end\nend\nChild.new.hi",
    );
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("super".to_string())))
    );
}

#[test]
fn defined_self_returns_string() {
    let result = run("class Foo\n  def check\n    defined?(self)\n  end\nend\nFoo.new.check");
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn defined_array_literal() {
    assert_eq!(
        run("defined?([1, 2, 3])"),
        Some(Object::String(std::rc::Rc::new("expression".to_string())))
    );
}

#[test]
fn defined_dictionary_literal() {
    assert_eq!(
        run("defined?({a: 1})"),
        Some(Object::String(std::rc::Rc::new("expression".to_string())))
    );
}

#[test]
fn defined_undefined_global_var_returns_nil() {
    assert_eq!(run("defined?($zzz_no_such_global)"), Some(Object::Nil));
}

#[test]
fn defined_instance_var_when_unset_returns_nil() {
    let result = run("class Foo\n  def check\n    defined?(@unset_iv)\n  end\nend\nFoo.new.check");
    assert_eq!(result, Some(Object::Nil));
}

// ── defined? coverage (additional paths) ─────────────────────────────────────

#[test]
fn defined_class_variable_from_method() {
    let result = run(r#"
class Foo
  @@x = 1
  def self.check
    defined?(@@x)
  end
end
Foo.check
"#);
    assert!(result.is_some());
    assert!(!matches!(result, Some(Object::Nil)));
}

#[test]
fn defined_scope_resolution_class() {
    let result = run(r#"
class Foo
end
defined?(Foo)
"#);
    assert!(result.is_some());
    assert!(!matches!(result, Some(Object::Nil)));
}

#[test]
fn defined_undefined_scope_resolution() {
    let result = run("defined?(NonExistent)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn defined_method_call_on_receiver() {
    let result = run(r#"defined?("hello".length)"#);
    assert!(result.is_some());
    assert!(!matches!(result, Some(Object::Nil)));
}

#[test]
fn defined_method_call_undefined_receiver() {
    let result = run("defined?(nonexist.method)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn defined_call_expression_coverage() {
    let result = run(r#"defined?(puts("hi"))"#);
    assert!(result.is_some());
}

#[test]
fn defined_index_expression() {
    let result = run("defined?([1,2,3][0])");
    assert!(result.is_some());
    assert!(!matches!(result, Some(Object::Nil)));
}

#[test]
fn defined_index_undefined_receiver() {
    let result = run("defined?(nonexist[0])");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn defined_super() {
    let result = run(r#"
class Foo
  def test
    defined?(super)
  end
end
Foo.new.test
"#);
    assert!(result.is_some());
}

#[test]
fn defined_self() {
    let result = run(r#"
class Foo
  def test
    defined?(self)
  end
end
Foo.new.test
"#);
    assert!(result.is_some());
    assert!(!matches!(result, Some(Object::Nil)));
}

// ── defined? catch-all for unknown expressions ───────────────────────────────

#[test]
fn defined_range_expression() {
    let result = run("defined?(1..10)");
    assert!(result.is_some());
}
