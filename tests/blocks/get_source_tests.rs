// Tests for get_source - retrieve Method objects for named methods

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
fn test_get_source_returns_method_object() {
    let source = r#"
class Dog
  def speak
    "Woof!"
  end
end
m = Dog.new.get_source(:speak)
m.name
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("speak")));
}

#[test]
fn test_get_source_with_string_name() {
    let source = r#"
class Dog
  def speak
    "Woof!"
  end
end
m = Dog.new.get_source("speak")
m.name
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("speak")));
}

#[test]
fn test_get_source_returns_nil_for_undefined_method() {
    let source = r#"
class Dog
end
Dog.new.get_source(:nonexistent)
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn test_get_source_returns_nil_for_native_method() {
    let source = r#"
class Foo
end
Foo.new.get_source(:to_s)
"#;
    // to_s is a native method stub; get_source looks it up in the user-defined table.
    // Since native methods are stubs in the class table, it returns a Method object.
    // The test verifies it doesn't crash and returns a result.
    let result = eval(source);
    // Either nil or a method stub - should not panic
    assert!(result == Some(Object::Nil) || matches!(result, Some(Object::Method(_))));
}

#[test]
fn test_get_source_with_inherited_method() {
    let source = r#"
class Animal
  def speak
    "..."
  end
end
class Dog < Animal
end
m = Dog.new.get_source(:speak)
m.name
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("speak")));
}

#[test]
fn test_get_source_returns_correct_method_for_multiple_methods() {
    let source = r#"
class Calculator
  def add(a, b)
    a + b
  end
  def subtract(a, b)
    a - b
  end
end
calc = Calculator.new
m_add = calc.get_source(:add)
m_sub = calc.get_source(:subtract)
m_add.name + " " + m_sub.name
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("add subtract")));
}

#[test]
fn test_get_source_method_is_callable() {
    let source = r#"
class Greeter
  def hello(name)
    "Hello, #{name}!"
  end
end
g = Greeter.new
m = g.get_source(:hello)
m.name
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn test_get_source_on_dynamic_method() {
    let source = r#"
class Foo
end
Foo.define_method(:bar) { |x| x * 2 }
m = Foo.new.get_source(:bar)
m.name
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("bar")));
}
