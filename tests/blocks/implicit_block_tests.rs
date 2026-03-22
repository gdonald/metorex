// Tests for implicit block capture (trailing blocks passed to methods)

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn eval(source: &str) -> Option<Object> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("Execution failed")
}

#[test]
fn test_trailing_do_end_block_with_block_param() {
    let source = r#"
def run(&block)
  block.call(10)
end

run do |x|
  x * 2
end
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn test_trailing_brace_block_with_block_param() {
    let source = r#"
def run(&block)
  block.call(5)
end

run { |x| x + 3 }
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn test_block_given_returns_true_with_block() {
    let source = r#"
def check
  block_given?
end

check { 42 }
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn test_block_given_returns_false_without_block() {
    let source = r#"
def check
  block_given?
end

check
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn test_block_given_conditional_execution() {
    let source = r#"
def maybe_block(&block)
  if block_given?
    block.call(7)
  else
    0
  end
end

a = maybe_block { |x| x * 3 }
b = maybe_block()
a + b
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(21)));
}

#[test]
fn test_trailing_block_with_explicit_args() {
    let source = r#"
def apply(x, &block)
  block.call(x)
end

apply(4) do |n|
  n * n
end
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(16)));
}

#[test]
fn test_trailing_brace_block_with_explicit_args() {
    let source = r#"
def apply(x, &block)
  block.call(x)
end

apply(6) { |n| n + 10 }
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(16)));
}

#[test]
fn test_array_each_with_trailing_do_end_block() {
    let source = r#"
sum = 0
[1, 2, 3].each do |n|
  sum = sum + n
end
sum
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn test_array_each_with_trailing_brace_block() {
    let source = r#"
sum = 0
[10, 20, 30].each { |n| sum = sum + n }
sum
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(60)));
}

#[test]
fn test_array_map_with_trailing_brace_block() {
    let source = r#"
[1, 2, 3].map { |n| n * 2 }
"#;
    let result = eval(source);
    match result {
        Some(Object::Array(arr)) => {
            let arr = arr.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Object::Int(2));
            assert_eq!(arr[1], Object::Int(4));
            assert_eq!(arr[2], Object::Int(6));
        }
        other => panic!("Expected Array, got {:?}", other),
    }
}

#[test]
fn test_array_map_with_trailing_do_end_block() {
    let source = r#"
[1, 2, 3].map do |n|
  n + 10
end
"#;
    let result = eval(source);
    match result {
        Some(Object::Array(arr)) => {
            let arr = arr.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Object::Int(11));
            assert_eq!(arr[1], Object::Int(12));
            assert_eq!(arr[2], Object::Int(13));
        }
        other => panic!("Expected Array, got {:?}", other),
    }
}

#[test]
fn test_range_each_with_trailing_do_end_block() {
    let source = r#"
sum = 0
(1..4).each do |i|
  sum = sum + i
end
sum
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn test_range_map_with_trailing_brace_block() {
    let source = r#"
(1..3).map { |i| i * i }
"#;
    let result = eval(source);
    match result {
        Some(Object::Array(arr)) => {
            let arr = arr.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Object::Int(1));
            assert_eq!(arr[1], Object::Int(4));
            assert_eq!(arr[2], Object::Int(9));
        }
        other => panic!("Expected Array, got {:?}", other),
    }
}

#[test]
fn test_block_captures_outer_variable() {
    let source = r#"
factor = 3
result = [1, 2, 3].map { |n| n * factor }
result
"#;
    let result = eval(source);
    match result {
        Some(Object::Array(arr)) => {
            let arr = arr.borrow();
            assert_eq!(arr[0], Object::Int(3));
            assert_eq!(arr[1], Object::Int(6));
            assert_eq!(arr[2], Object::Int(9));
        }
        other => panic!("Expected Array, got {:?}", other),
    }
}

#[test]
fn test_no_block_param_block_given_check() {
    let source = r#"
def greet(name)
  if block_given?
    "block given"
  else
    "no block"
  end
end

greet("Alice")
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("no block")));
}

#[test]
fn test_if_else_as_last_statement_in_function() {
    let source = r#"
def classify(n)
  if n > 0
    "positive"
  elsif n < 0
    "negative"
  else
    "zero"
  end
end

classify(5)
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("positive")));
}

#[test]
fn test_trailing_block_without_parens() {
    let source = r#"
def run_twice(&block)
  a = block.call(1)
  b = block.call(2)
  a + b
end

run_twice { |x| x * 10 }
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn test_select_with_trailing_brace_block() {
    let source = r#"
[1, 2, 3, 4, 5, 6].select { |n| n % 2 == 0 }
"#;
    let result = eval(source);
    match result {
        Some(Object::Array(arr)) => {
            let arr = arr.borrow();
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Object::Int(2));
            assert_eq!(arr[1], Object::Int(4));
            assert_eq!(arr[2], Object::Int(6));
        }
        other => panic!("Expected Array, got {:?}", other),
    }
}
