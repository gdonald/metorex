// Tests for define_method - dynamically defining methods on classes

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
fn test_define_method_on_class_with_brace_block() {
    let source = r#"
class Greeter
end
Greeter.define_method(:hello) { |name| "Hello, #{name}!" }
g = Greeter.new
g.hello("World")
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("Hello, World!")));
}

#[test]
fn test_define_method_on_class_with_do_end_block() {
    let source = r#"
class Greeter
end
Greeter.define_method(:hello) do |name|
  "Hello, #{name}!"
end
g = Greeter.new
g.hello("Alice")
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("Hello, Alice!")));
}

#[test]
fn test_define_method_inside_class_body() {
    let source = r#"
class Calculator
  define_method(:double) { |n| n * 2 }
end
calc = Calculator.new
calc.double(5)
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn test_define_method_multiple_inside_class_body() {
    let source = r#"
class Calculator
  define_method(:double) { |n| n * 2 }
  define_method(:triple) { |n| n * 3 }
end
calc = Calculator.new
calc.double(3) + calc.triple(2)
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(12)));
}

#[test]
fn test_define_method_no_parameters() {
    let source = r#"
class Greeter
  define_method(:greet) { "Hi there!" }
end
Greeter.new.greet
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("Hi there!")));
}

#[test]
fn test_define_method_with_string_name() {
    let source = r#"
class Foo
end
Foo.define_method("bar") { |x| x + 1 }
Foo.new.bar(41)
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn test_define_method_is_callable_on_instance() {
    let source = r#"
class Animal
end
Animal.define_method(:speak) { "..." }
a = Animal.new
a.speak
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("...")));
}

#[test]
fn test_define_method_reflects_in_respond_to() {
    let source = r#"
class Robot
end
Robot.define_method(:beep) { "beep!" }
r = Robot.new
r.respond_to?("beep")
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn test_define_method_dynamic_loop() {
    let source = r#"
class NumberMethods
end
["zero", "one", "two"].each do |word|
  NumberMethods.define_method(word) { word }
end
nm = NumberMethods.new
nm.zero
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("zero")));
}

#[test]
fn test_define_method_replaces_existing_method() {
    let source = r#"
class Foo
  def greet
    "old"
  end
end
Foo.define_method(:greet) { "new" }
Foo.new.greet
"#;
    let result = eval(source);
    assert_eq!(result, Some(Object::string("new")));
}
