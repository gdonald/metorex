// Edge-case coverage tests for native method error paths

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

// ── Array [] operator on custom class ───────────────────────────────────────

#[test]
fn array_bracket_operator_custom() {
    let result = run(r#"
class MyArr
  def initialize
    @data = [10, 20, 30]
  end
  def [](idx)
    @data[idx]
  end
end
a = MyArr.new
a[1]
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

// ── Hash edge cases ────────────────────────────────────────────────────────

#[test]
fn hash_each_with_block() {
    let result = run(r#"
result = []
h = {"a" => 1, "b" => 2}
h.each do |k, v|
  result.push(k)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_merge_method() {
    let result = run(r#"
a = {"x" => 1}
b = {"y" => 2}
c = a.merge(b)
c.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── Operator edge cases ─────────────────────────────────────────────────────

#[test]
fn modulo_operator() {
    let result = run("10 % 3");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn float_modulo_operator() {
    let result = run("10.5 % 3.0");
    if let Some(Object::Float(f)) = result {
        assert!((f - 1.5).abs() < 1e-9);
    } else {
        panic!("Expected Float");
    }
}

// ── Method invocation: undefined method check ───────────────────────────────

#[test]
fn undef_method_error_via_invoke() {
    let err = run_err(
        r#"
class Foo
  def bar
    42
  end
end
Foo.undef_method("bar")
Foo.new.bar
"#,
    );
    assert!(err.contains("Undefined method 'bar'"));
}

// ── VM core: string interpolation ───────────────────────────────────────────

#[test]
fn string_interpolation_with_expression() {
    let result = run(r#"
x = 5
"result is #{x + 3}"
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("result is 8".to_string())))
    );
}

// ── Unless expression ───────────────────────────────────────────────────────

#[test]
fn unless_expression_value() {
    let result = run(r#"
x = unless false
  42
end
x
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── For loop with range ─────────────────────────────────────────────────────

#[test]
fn for_loop_range_sum() {
    let result = run(r#"
total = 0
for i in 1..5
  total = total + i
end
total
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── While loop ──────────────────────────────────────────────────────────────

#[test]
fn while_loop_basic() {
    let result = run(r#"
x = 0
while x < 5
  x = x + 1
end
x
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── Calling non-callable errors ─────────────────────────────────────────────

#[test]
fn call_non_callable_errors() {
    let err = run_err(
        r#"
x = 42
x.call
"#,
    );
    assert!(err.contains("Undefined method 'call'"));
}

// ── Divide by zero ──────────────────────────────────────────────────────────

#[test]
fn divide_by_zero_int() {
    let err = run_err("1 / 0");
    assert!(err.contains("Division by zero") || err.contains("divide"));
}

#[test]
fn divide_by_zero_float() {
    // Float division by zero should error
    let err = run_err("1.0 / 0.0");
    assert!(
        err.contains("Division by zero") || err.contains("zero"),
        "err was: {}",
        err
    );
}

// ── Array sort ──────────────────────────────────────────────────────────────

#[test]
fn array_sort_method() {
    let result = run(r#"
[3, 1, 2].sort
"#);
    if let Some(Object::Array(arr)) = result {
        let vals = arr.borrow();
        assert_eq!(vals[0], Object::Int(1));
        assert_eq!(vals[1], Object::Int(2));
        assert_eq!(vals[2], Object::Int(3));
    }
}

// ── Array flat_map ──────────────────────────────────────────────────────────

#[test]
fn array_map_double() {
    let result = run(r#"
[1, 2, 3].map do |x|
  x * 2
end
"#);
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 3);
        assert_eq!(arr.borrow()[0], Object::Int(2));
    }
}

// ── Array select/filter ─────────────────────────────────────────────────────

#[test]
fn array_select_filter() {
    let result = run(r#"
[1, 2, 3, 4, 5].select do |x|
  x > 3
end
"#);
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    }
}

// ── Array reduce ────────────────────────────────────────────────────────────

#[test]
fn array_reduce_sum() {
    let result = run(r#"
[1, 2, 3, 4].reduce(0) do |sum, x|
  sum + x
end
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Range to_a ──────────────────────────────────────────────────────────────

#[test]
fn range_to_a_inclusive() {
    let result = run("(1..3).to_a");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 3);
    }
}

#[test]
fn range_to_a_exclusive() {
    let result = run("(1...3).to_a");
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    }
}

// ── Type introspection methods ──────────────────────────────────────────────

#[test]
fn nil_question_mark() {
    let result = run("nil.nil?");
    assert_eq!(result, Some(Object::Bool(true)));
    let result = run("42.nil?");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn is_a_method() {
    let result = run(r#"
class Animal
end
class Dog < Animal
end
d = Dog.new
d.is_a?(Animal)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── respond_to? ─────────────────────────────────────────────────────────────

#[test]
fn respond_to_defined_method() {
    let result = run(r#"
class Foo
  def bar
    42
  end
end
Foo.new.respond_to?("bar")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_undefined_method() {
    let result = run(r#"
class Foo
end
Foo.new.respond_to?("nonexistent")
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── instance_variables method ───────────────────────────────────────────────

#[test]
fn instance_variables_method() {
    let result = run(r#"
class Foo
  def initialize
    @x = 1
    @y = 2
  end
end
f = Foo.new
f.instance_variables
"#);
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    }
}

// ── dup/clone ───────────────────────────────────────────────────────────────

#[test]
fn dup_string() {
    let result = run(r#"
s = "hello"
s.dup
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

// ── instance_variable_get/set ───────────────────────────────────────────────

#[test]
fn instance_variable_get() {
    let result = run(r#"
class Foo
  def initialize
    @x = 10
  end
end
f = Foo.new
f.instance_variable_get("@x")
"#);
    assert_eq!(result, Some(Object::Int(10)));
}
