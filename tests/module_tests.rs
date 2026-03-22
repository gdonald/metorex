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

#[test]
fn module_include_adds_instance_methods() {
    let result = run("
module Greetable
  def greet
    42
  end
end
class Foo
  include Greetable
end
Foo.new.greet
");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn module_extend_adds_class_methods() {
    let result = run("
module ClassOps
  def answer
    99
  end
end
class Bar
  extend ClassOps
end
Bar.answer
");
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn module_methods_can_access_instance_vars() {
    let result = run("
module Named
  def get_name
    @name
  end
end
class Person
  include Named
  def initialize(name)
    @name = name
  end
end
Person.new(\"Bob\").get_name
");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Bob".to_string())))
    );
}

#[test]
fn multiple_modules_can_be_included() {
    let result = run("
module A
  def a_val
    1
  end
end
module B
  def b_val
    2
  end
end
class C
  include A
  include B
end
c = C.new
c.a_val + c.b_val
");
    assert_eq!(result, Some(Object::Int(3)));
}
