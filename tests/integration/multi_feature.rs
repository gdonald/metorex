// 10.3.4 — Multi-Feature Integration Tests
//
// Tests that combine multiple language features together in single programs,
// verifying that different subsystems interoperate correctly.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run_env(code: &str) -> VirtualMachine {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed");
    vm
}

// ── Classes + Inheritance + Blocks ──

#[test]
fn class_with_block_iteration() {
    let code = r#"class NumberSet
  def initialize
    @numbers = []
  end
  def add(n)
    @numbers.push(n)
  end
  def sum
    total = 0
    @numbers.each do |n|
      total += n
    end
    total
  end
end
ns = NumberSet.new
ns.add(10)
ns.add(20)
ns.add(30)
result = ns.sum"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(60)));
}

// ── Functions + Closures + Arrays ──

#[test]
fn higher_order_function_with_closure() {
    let code = r#"def make_adder(n)
  lambda do |x| x + n end
end
add5 = make_adder(5)
add10 = make_adder(10)
result = add5.call(3) + add10.call(3)"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(21)));
}

// ── Control Flow + String Interpolation + Arrays ──

#[test]
fn control_flow_with_string_building() {
    let code = "results = []\nfor i in 1..5\n  if i % 2 == 0\n    results.push(\"even\")\n  else\n    results.push(\"odd\")\n  end\nend\nresult = results.length";
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(5)));
}

// ── Classes + Inheritance + Method Override + Super ──

#[test]
fn inheritance_chain_with_super() {
    let code = r#"class Base
  def value
    10
  end
end
class Middle < Base
  def value
    super + 20
  end
end
class Top < Middle
  def value
    super + 30
  end
end
result = Top.new.value"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(60)));
}

// ── Hashes + Iteration + Conditional Logic ──

#[test]
fn hash_processing_pipeline() {
    let code = r#"scores = {"alice" => 85, "bob" => 92, "charlie" => 78}
count = 0
scores.each do |name, score|
  if score >= 80
    count += 1
  end
end
result = count"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(2)));
}

// ── Exception Handling + Classes ──

#[test]
fn exception_handling_with_raise_and_rescue() {
    let code = r#"class Validator
  def validate(value)
    result = nil
    begin
      if value < 0
        raise "negative value"
      end
      result = value * 2
    rescue => e
      result = -1
    end
    result
  end
end
v = Validator.new
r1 = v.validate(5)
r2 = v.validate(-1)
result = r1 + r2"#;
    let vm = run_env(code);
    // 5*2 = 10, -1 rescued as -1 => 10 + (-1) = 9
    assert_eq!(vm.environment().get("result"), Some(Object::Int(9)));
}

// ── define_method + Blocks + Classes ──

#[test]
fn dynamic_method_definition_in_class() {
    let code = r#"class Multiplier
end
Multiplier.define_method(:double) { |n| n * 2 }
Multiplier.define_method(:triple) { |n| n * 3 }
m = Multiplier.new
result = m.double(5) + m.triple(5)"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(25)));
}

// ── Nested Blocks + Map + Filter ──

#[test]
fn chained_array_operations() {
    let code = r#"numbers = [1, 2, 3, 4, 5, 6]
evens = numbers.filter { |n| n % 2 == 0 }
doubled = evens.map { |n| n * 2 }
result = doubled.length"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(3)));
}

// ── Range + For + While + Break ──

#[test]
fn loop_with_break() {
    let code = r#"result = 0
for i in 1..100
  if i > 10
    break
  end
  result += i
end"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("result"), Some(Object::Int(55)));
}

// ── String Methods + Array + Map ──

#[test]
fn string_processing_pipeline() {
    let code = r#"words = ["hello", "world", "foo"]
upper = words.map { |w| w.upcase }
result = upper[0]"#;
    let vm = run_env(code);
    assert_eq!(
        vm.environment().get("result"),
        Some(Object::String(Rc::new("HELLO".to_string())))
    );
}

// ── Pattern Matching + Classes ──

#[test]
fn case_with_multiple_types() {
    let code = r#"def classify(x)
  case x
  when 0
    "zero"
  when 1
    "one"
  else
    "other"
  end
end
r1 = classify(0)
r2 = classify(1)
r3 = classify(99)"#;
    let vm = run_env(code);
    assert_eq!(
        vm.environment().get("r1"),
        Some(Object::String(Rc::new("zero".to_string())))
    );
    assert_eq!(
        vm.environment().get("r2"),
        Some(Object::String(Rc::new("one".to_string())))
    );
    assert_eq!(
        vm.environment().get("r3"),
        Some(Object::String(Rc::new("other".to_string())))
    );
}

// ── Full Program: Class + Methods + Blocks + Collections + Control Flow ──

#[test]
fn full_program_student_grades() {
    let code = r#"class Student
  def initialize(name, grade)
    @name = name
    @grade = grade
  end
  def name
    @name
  end
  def grade
    @grade
  end
  def passing?
    @grade >= 60
  end
end

students = [
  Student.new("Alice", 95),
  Student.new("Bob", 55),
  Student.new("Charlie", 78)
]

passing_count = 0
total_grade = 0
students.each do |s|
  total_grade += s.grade
  if s.passing?
    passing_count += 1
  end
end

average = total_grade / students.length"#;
    let vm = run_env(code);
    assert_eq!(vm.environment().get("passing_count"), Some(Object::Int(2)));
    assert_eq!(vm.environment().get("total_grade"), Some(Object::Int(228)));
}
