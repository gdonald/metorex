// Tests for built-in Object methods (class, to_s, respond_to?) on all types

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

// ── .class on built-in types returns a Class object ─────────────────────

#[test]
fn array_class() {
    let result = run("[1, 2, 3].class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Array");
    }
}

#[test]
fn string_class() {
    let result = run(r#""hello".class"#);
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "String");
    }
}

#[test]
fn integer_class() {
    let result = run("42.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Integer");
    }
}

#[test]
fn float_class() {
    let result = run("3.14.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Float");
    }
}

#[test]
fn bool_class() {
    let result = run("true.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "TrueClass");
    }
}

#[test]
fn nil_class() {
    let result = run("nil.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "NilClass");
    }
}

#[test]
fn range_class() {
    let result = run("(1..5).class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Range");
    }
}

#[test]
fn hash_class() {
    let result = run(r#"{"a" => 1}.class"#);
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Hash");
    }
}

#[test]
fn set_class() {
    let result = run("Set.new.class");
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Set");
    }
}

#[test]
fn user_instance_class() {
    let result = run(r#"
class Dog
end
Dog.new.class
"#);
    assert!(matches!(result, Some(Object::Class(_))));
    if let Some(Object::Class(c)) = result {
        assert_eq!(c.name(), "Dog");
    }
}

// ── .to_s on built-in types ─────────────────────────────────────────────

#[test]
fn array_to_s() {
    let result = run("[1, 2, 3].to_s");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[1, 2, 3]".to_string())))
    );
}

#[test]
fn integer_to_s() {
    let result = run("42.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("42".to_string()))));
}

#[test]
fn nil_to_s() {
    // Ruby semantics: nil.to_s == "". Use nil.inspect for the literal "nil".
    let result = run("nil.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("".to_string()))));
}

#[test]
fn bool_to_s() {
    let result = run("true.to_s");
    assert_eq!(result, Some(Object::String(Rc::new("true".to_string()))));
}

// ── .respond_to? on built-in types ──────────────────────────────────────

#[test]
fn array_respond_to_length() {
    let result = run(r#"[1, 2].respond_to?("length")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_respond_to_upcase() {
    let result = run(r#""hello".respond_to?("upcase")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn integer_respond_to_class() {
    let result = run(r#"42.respond_to?("class")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── instance_variable_set/get ───────────────────────────────────────────────

#[test]
fn instance_variable_set_on_instance() {
    let result = run(r#"
class Foo; end
f = Foo.new
f.instance_variable_set(:@x, 42)
f.instance_variable_get(:@x)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn instance_variable_set_on_class() {
    let result = run(r#"
class Foo
  def self.setup
    instance_variable_set(:@val, 99)
  end
  def self.val
    instance_variable_get(:@val)
  end
end
Foo.setup
Foo.val
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

fn run_err(code: &str) -> String {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    use metorex::vm::VirtualMachine;
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

#[test]
fn instance_variable_set_with_string_name() {
    let result = run(r#"
class Foo; end
f = Foo.new
f.instance_variable_set("@x", 7)
f.instance_variable_get("@x")
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_set_wrong_arg_count_errors() {
    let err = run_err(
        r#"
class Foo; end
Foo.new.instance_variable_set(:@x)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_set_non_string_name_errors() {
    let err = run_err(
        r#"
class Foo; end
Foo.new.instance_variable_set(42, "bad")
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol"));
}

#[test]
fn instance_variable_set_on_module() {
    let result = run(r#"
module M
end
M.instance_variable_set(:@v, 11)
M.instance_variable_get(:@v)
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn instance_variable_set_on_true_raises_frozen_error() {
    let err = run_err(r#"true.instance_variable_set(:@x, 1)"#);
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("TrueClass"));
}

#[test]
fn instance_variable_set_on_false_raises_frozen_error() {
    let err = run_err(r#"false.instance_variable_set(:@x, 1)"#);
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("FalseClass"));
}

#[test]
fn instance_variable_set_on_nil_raises_frozen_error() {
    let err = run_err(r#"nil.instance_variable_set(:@x, 1)"#);
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("NilClass"));
}

#[test]
fn instance_variable_set_on_integer_raises_frozen_error() {
    let err = run_err(r#"42.instance_variable_set(:@x, 1)"#);
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("Integer"));
}

#[test]
fn instance_variable_set_on_symbol_raises_frozen_error() {
    let err = run_err(r#":foo.instance_variable_set(:@x, 1)"#);
    assert!(err.contains("can't modify frozen"));
}

#[test]
fn instance_variable_set_on_immediate_raises_runtime_error_subclass() {
    let result = run(r#"
result = begin
  true.instance_variable_set(:@x, 1)
  "no error"
rescue RuntimeError => e
  "caught: #{e.class}"
end
result
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("caught: FrozenError".to_string())))
    );
}

#[test]
fn instance_variable_set_on_frozen_instance_raises_frozen_error() {
    let err = run_err(
        r#"
class Foo; end
f = Foo.new
f.freeze
f.instance_variable_set(:@x, 1)
"#,
    );
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("Foo"));
}

// ── dup / clone ───────────────────────────────────────────────────────────

#[test]
fn array_dup_creates_independent_array() {
    let result = run(r#"
a = [1, 2, 3]
b = a.dup
b << 4
a.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_clone_creates_independent_array() {
    let result = run(r#"
a = [1, 2, 3]
b = a.clone
b << 4
a.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn dict_dup_creates_independent_dict() {
    let result = run(r#"
h = {"a" => 1}
g = h.dup
g["b"] = 2
h["b"]
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn dup_on_integer_returns_same() {
    let result = run("42.dup");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn instance_dup_creates_independent_instance() {
    let result = run(r#"
class Foo
  def initialize(v)
    @v = v
  end
  def v
    @v
  end
  def v=(x)
    @v = x
  end
end
a = Foo.new(1)
b = a.dup
b.v = 99
a.v
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_with_args_errors() {
    let err = run_err("[1, 2].dup(3)");
    assert!(err.contains("argument"));
}

// ── object_methods.rs: to_s with arguments error (lines 44-49) ─────────────

#[test]
fn object_to_s_with_args_error() {
    let err = run_err("42.to_s(10)");
    assert!(err.contains("argument"));
}

// ── object_methods.rs: respond_to? with non-String/Symbol arg error (line 78) ─

#[test]
fn object_respond_to_non_string_arg_error() {
    let err = run_err(r#"42.respond_to?(123)"#);
    assert!(err.contains("123 is not a symbol nor a string"));
}

// ── object_methods.rs: instance_variable_set on immediates raises FrozenError ─

#[test]
fn instance_variable_set_on_plain_object_raises_frozen_error() {
    let err = run_err(r#"42.instance_variable_set(:@x, 99)"#);
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("Integer"));
}

// ── object_methods.rs: methods on instance with superclass (lines 308, 315) ─

#[test]
fn object_methods_includes_inherited_methods() {
    let result = run(r#"
class Animal
  def speak
    "..."
  end
end
class Dog < Animal
  def bark
    "woof"
  end
end
d = Dog.new
m = d.methods
m.include?(:speak)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_methods_returns_array() {
    let result = run(r#"42.methods"#);
    assert!(matches!(result, Some(Object::Array(_))));
}

// ── object_methods.rs: send with non-String/Symbol method name error (lines 368) ─

#[test]
fn object_send_non_string_method_name_error() {
    let err = run_err(r#"42.send(123)"#);
    assert!(err.contains("123 is not a symbol nor a string"));
}

// ── object_methods.rs: send dispatching native method (lines 370-376) ─────

#[test]
fn object_send_dispatches_native_method() {
    let result = run(r#"[1, 2, 3].send("length")"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn object_send_dispatches_object_method() {
    let result = run(r#"42.send("to_s")"#);
    assert_eq!(result, Some(Object::String(Rc::new("42".to_string()))));
}

#[test]
fn object_send_undefined_method_error() {
    let err = run_err(r#"42.send("totally_nonexistent_xyz")"#);
    assert!(err.contains("method") || err.contains("undefined") || err.contains("nonexistent"));
}

// ── object_methods.rs: dup/clone on Array via object_methods (lines 399-402) ─

#[test]
fn object_dup_on_set_returns_copy() {
    let result = run(r#"
s = Set.new([1, 2, 3])
t = s.dup
t.size
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── object_methods.rs: =~ on non-string/regex pair returns Nil (line 418) ──

#[test]
fn object_match_operator_non_string_returns_nil() {
    let result = run(r#"42 =~ 99"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── object_methods.rs: =~ with invalid regex returns Nil (line 437) ────────

#[test]
fn object_match_operator_invalid_regex_returns_nil() {
    // An invalid regex pattern should return nil rather than error
    let result = run(r#""hello" =~ /[invalid(/"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── object_methods.rs: !~ on non-string/regex pair returns true (lines 440, 452) ─

#[test]
fn object_not_match_operator_without_a_match_method_raises() {
    // Ruby has no `Object#=~`, so an Integer has no `!~` either.
    let error = run_err(r#"42 !~ 99"#);
    assert!(error.contains("undefined method '=~' for an instance of Integer"));
}

// ── object_methods.rs: !~ with invalid regex returns true (lines 445-448) ──

#[test]
fn object_not_match_operator_invalid_regex_returns_true() {
    let result = run(r#""hello" !~ /[invalid(/"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── object_methods.rs: instance_exec with block (lines 466-471) ─────────────

#[test]
fn object_instance_exec_with_block() {
    let result = run(r#"
x = 42
result = x.instance_exec { 100 }
result
"#);
    assert_eq!(result, Some(Object::Int(100)));
}

#[test]
fn object_instance_eval_with_block() {
    let result = run(r#"
x = 42
result = x.instance_eval { 200 }
result
"#);
    assert_eq!(result, Some(Object::Int(200)));
}

// ── object_methods.rs: instance_exec without block error (lines 480-482) ───

#[test]
fn object_instance_exec_without_block_error() {
    let err = run_err(r#"42.instance_exec"#);
    assert!(err.contains("block") || err.contains("requires"));
}

#[test]
fn object_instance_eval_without_block_or_source_error() {
    // With neither a block nor a source string there is nothing to run, so
    // Ruby reports the missing argument.
    let err = run_err(r#"42.instance_eval"#);
    assert!(
        err.contains("wrong number of arguments (given 0, expected 1..3)"),
        "unexpected error: {err}"
    );
}

// ── !~ dispatches =~ ─────────────────────────────────────────────────────────

#[test]
fn not_match_negates_a_user_defined_match_method() {
    let result = run(r#"
class Matcher
  def =~(other)
    other == :expected
  end
end
Matcher.new !~ :expected
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn not_match_is_true_when_the_match_method_says_no() {
    let result = run(r#"
class Matcher
  def =~(other)
    false
  end
end
Matcher.new !~ :anything
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn not_match_can_be_overridden_directly() {
    let result = run(r#"
class Overridden
  def !~(other)
    :custom
  end
end
(Overridden.new !~ :anything).inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(":custom".to_string())))
    );
}

#[test]
fn not_match_raises_for_a_receiver_without_a_match_method() {
    let error = run_err("Object.new !~ :foo");
    assert!(error.contains("undefined method '=~' for an instance of Object"));
}
