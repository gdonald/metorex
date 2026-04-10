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

// ── Module body features ────────────────────────────────────────────────────

#[test]
fn module_body_ivar_assignment() {
    run("module MSpec\n  @exit = nil\n  @abort = nil\nend");
}

#[test]
fn module_body_class_self_attr() {
    run("module Foo\n  class << self\n    attr_reader :bar\n  end\nend");
}

#[test]
fn module_body_ivar_and_method() {
    run("module Config\n  @debug = true\n  def self.debug\n    @debug\n  end\nend");
}

#[test]
fn module_class_self_method_def() {
    run("module Helpers\n  class << self\n    def answer\n      42\n    end\n  end\nend");
}

#[test]
fn module_class_self_attr_writer() {
    run("module Conf\n  class << self\n    attr_writer :debug\n  end\nend");
}

#[test]
fn module_class_self_attr_accessor() {
    run("module Conf\n  class << self\n    attr_accessor :verbose\n  end\nend");
}

#[test]
fn module_class_self_method_and_attr() {
    run(
        "module Helpers\n  class << self\n    attr_reader :count\n    def reset\n      42\n    end\n  end\nend",
    );
}

#[test]
fn module_class_self_attr_accessor_in_module() {
    run(
        "module Tracker\n  class << self\n    attr_accessor :count\n    attr_writer :name\n    attr_reader :id\n    def reset\n      42\n    end\n  end\nend",
    );
}
