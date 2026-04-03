// Remaining coverage tests targeting specific uncovered lines

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

// ── module_function on Class: error paths ───────────────────────────────────

#[test]
fn class_module_function_wrong_args() {
    let err = run_err(
        r#"
class Foo
  def bar
    1
  end
end
Foo.module_function()
"#,
    );
    assert!(err.contains("expected 1"));
}

#[test]
fn class_module_function_type_error() {
    let err = run_err(
        r#"
class Foo
end
Foo.module_function(123)
"#,
    );
    assert!(err.contains("String or Symbol"));
}

#[test]
fn class_module_function_with_symbol() {
    let result = run(r#"
class Foo
  def bar
    99
  end
end
Foo.module_function(:bar)
Foo.bar
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

// ── Hash access ─────────────────────────────────────────────────────────────

#[test]
fn hash_bracket_access_hit() {
    let result = run(r#"
h = {"a" => 1, "b" => 2}
h["a"]
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_bracket_access_miss_errors() {
    let err = run_err(
        r#"
h = {"a" => 1}
h["z"]
"#,
    );
    assert!(err.contains("not found"));
}

// ── Hash methods ────────────────────────────────────────────────────────────

// ── Int times ───────────────────────────────────────────────────────────────

#[test]
fn int_times_iteration() {
    let result = run(r#"
total = 0
5.times do |i|
  total = total + i
end
total
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Range each ──────────────────────────────────────────────────────────────

#[test]
fn range_each_inclusive() {
    let result = run(r#"
total = 0
(1..5).each do |i|
  total = total + i
end
total
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn range_each_exclusive() {
    let result = run(r#"
total = 0
(1...5).each do |i|
  total = total + i
end
total
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Range map ───────────────────────────────────────────────────────────────

#[test]
fn range_map_doubles() {
    let result = run(r#"
(1..3).map do |x|
  x * 2
end
"#);
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 3);
        assert_eq!(arr.borrow()[0], Object::Int(2));
    }
}

// ── Range include? ──────────────────────────────────────────────────────────

#[test]
fn range_include_boundary() {
    let result = run("(1..5).include?(5)");
    assert_eq!(result, Some(Object::Bool(true)));
    let result = run("(1...5).include?(5)");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Set each ────────────────────────────────────────────────────────────────

// ── Set union ───────────────────────────────────────────────────────────────

#[test]
fn set_union_method() {
    let result = run(r#"
a = Set.new([1, 2])
b = Set.new([2, 3])
c = a.union(b)
c.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Set intersection ────────────────────────────────────────────────────────

#[test]
fn set_intersection_method() {
    let result = run(r#"
a = Set.new([1, 2, 3])
b = Set.new([2, 3, 4])
c = a.intersection(b)
c.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── Set difference ──────────────────────────────────────────────────────────

#[test]
fn set_difference_method() {
    let result = run(r#"
a = Set.new([1, 2, 3])
b = Set.new([2, 3])
c = a.difference(b)
c.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Float to_i ──────────────────────────────────────────────────────────────

#[test]
fn float_to_i_method() {
    let result = run("3.14.to_i");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── String equality ─────────────────────────────────────────────────────────

#[test]
fn string_equality() {
    let result = run(r#""hello" == "hello""#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── case/when with ranges ───────────────────────────────────────────────────

#[test]
fn case_when_with_range() {
    let result = run(r#"
case 5
when 1..3
  "low"
when 4..6
  "mid"
when 7..10
  "high"
end
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("mid".to_string())))
    );
}

// ── begin/rescue with ensure ────────────────────────────────────────────────

#[test]
fn begin_rescue_ensure() {
    let result = run(r#"
result = ""
begin
  result = result + "try "
  raise "error"
rescue => e
  result = result + "rescue "
ensure
  result = result + "ensure"
end
result
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "try rescue ensure".to_string()
        )))
    );
}

// ── Method chaining ─────────────────────────────────────────────────────────

#[test]
fn method_chaining() {
    let result = run(r#"
[3, 1, 2].sort.reverse.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── If expression value ─────────────────────────────────────────────────────

#[test]
fn if_expression_value() {
    let result = run(r#"
x = if true then 42 else 0 end
x
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Return from method ──────────────────────────────────────────────────────

#[test]
fn explicit_return() {
    let result = run(r#"
def early_return(x)
  if x > 0
    return "positive"
  end
  "non-positive"
end
early_return(5)
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("positive".to_string())))
    );
}

// ── Break in loop ───────────────────────────────────────────────────────────

#[test]
fn break_in_while_loop() {
    let result = run(r#"
x = 0
while true
  x = x + 1
  if x >= 3
    break
  end
end
x
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Array sort and reverse ──────────────────────────────────────────────────

#[test]
fn array_sort_method() {
    let result = run(r#"
[3, 1, 2].sort
"#);
    if let Some(Object::Array(arr)) = result {
        let vals = arr.borrow();
        assert_eq!(vals[0], Object::Int(1));
        assert_eq!(vals[2], Object::Int(3));
    }
}

// ── Array map ───────────────────────────────────────────────────────────────

#[test]
fn array_map_double() {
    let result = run(r#"
[1, 2, 3].map do |x|
  x * 2
end
"#);
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow()[0], Object::Int(2));
        assert_eq!(arr.borrow()[2], Object::Int(6));
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

// ── Array select ────────────────────────────────────────────────────────────

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

// ── Nil check ───────────────────────────────────────────────────────────────

#[test]
fn nil_question_mark() {
    let result = run("nil.nil?");
    assert_eq!(result, Some(Object::Bool(true)));
    let result = run("42.nil?");
    assert_eq!(result, Some(Object::Bool(false)));
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

// ── instance_variables ──────────────────────────────────────────────────────

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

// ── dup ─────────────────────────────────────────────────────────────────────

#[test]
fn dup_string() {
    let result = run(r#"
"hello".dup
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

// ── instance_variable_get ───────────────────────────────────────────────────

#[test]
fn instance_variable_get() {
    let result = run(r#"
class Foo
  def initialize
    @x = 10
  end
end
Foo.new.instance_variable_get("@x")
"#);
    assert_eq!(result, Some(Object::Int(10)));
}
