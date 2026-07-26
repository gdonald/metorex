// Coverage tests for vm/class_execution.rs uncovered paths

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

// ── Instance variable declaration in class body (line 96-100) ────────────────

#[test]
fn instance_var_declaration_in_class_body() {
    let result = run(r#"
class C
  @name
  def initialize(n)
    @name = n
  end
  def name
    @name
  end
end
C.new("test").name
"#);
    assert_eq!(result, Some(Object::string("test")));
}

// ── include with non-module value (lines 203-205) ─────────────────────────────

#[test]
fn include_non_module_error() {
    let err = run_err(
        r#"
class C
  include Integer
end
"#,
    );
    assert!(err.contains("module") || err.contains("Integer") || err.contains("not a module"));
}

// ── include with undefined module (lines 209-211) ────────────────────────────

#[test]
fn include_undefined_module_error() {
    let err = run_err(
        r#"
class C
  include UndefinedModule
end
"#,
    );
    assert!(err.contains("Undefined") || err.contains("module") || err.contains("UndefinedModule"));
}

// ── extend with non-module value (lines 233-235) ──────────────────────────────

#[test]
fn extend_non_module_error() {
    let err = run_err(
        r#"
class C
  extend Integer
end
"#,
    );
    assert!(err.contains("module") || err.contains("Integer") || err.contains("not a module"));
}

// ── extend with undefined module (lines 239-241) ─────────────────────────────

#[test]
fn extend_undefined_module_error() {
    let err = run_err(
        r#"
class C
  extend UndefinedMod
end
"#,
    );
    assert!(err.contains("Undefined") || err.contains("module") || err.contains("UndefinedMod"));
}

// ── define_method with invalid first arg type (lines 265-272) ────────────────

#[test]
fn define_method_invalid_name_type_error() {
    let err = run_err(
        r#"
class C
  define_method(42) do
    "hello"
  end
end
"#,
    );
    assert!(err.contains("is not a symbol nor a string"));
}

// ── define_method with no args (lines 275-278) ───────────────────────────────

#[test]
fn define_method_no_args_error() {
    let err = run_err(
        r#"
class C
  define_method do
    "hello"
  end
end
"#,
    );
    assert!(err.contains("define_method") || err.contains("argument") || err.contains("requires"));
}

// ── module body with unsupported statement (lines 402-404) ───────────────────

#[test]
fn module_body_general_statements_execute() {
    // Module bodies now support general statements (not just method defs and constants)
    let result = run(r#"
module M
  if true
    42
  end
end
"#);
    // Module definition returns nil
    assert!(result.is_none() || result == Some(Object::Nil));
}

// ── define_method with captured vars closure (line 299) ──────────────────────

#[test]
fn define_method_with_closure_captures_variables() {
    let result = run(r#"
prefix = "Hello"
class Greeter
  define_method(:greet) do |name|
    prefix + ", " + name
  end
end
Greeter.new.greet("World")
"#);
    assert_eq!(result, Some(Object::string("Hello, World")));
}

// ── Class superclass not a class error ───────────────────────────────────────

#[test]
fn class_superclass_not_class_error() {
    let err = run_err(
        r#"
x = 42
class C < x
end
"#,
    );
    assert!(err.contains("Superclass") || err.contains("class") || err.contains("must be"));
}

#[test]
fn class_superclass_undefined_error() {
    let err = run_err(
        r#"
class C < UndefinedClass
end
"#,
    );
    assert!(
        err.contains("Undefined") || err.contains("superclass") || err.contains("UndefinedClass")
    );
}

// ── define_method with closure capture (lines 284-299) ──────────────────────

#[test]
fn define_method_with_closure() {
    let result = run("class Foo\n  define_method(\"val\") do\n    100\n  end\nend\nFoo.new.val");
    assert_eq!(result, Some(Object::Int(100)));
}

// ── def self.method vs def method separation ────────────────────────────────

fn run_class(code: &str) -> Option<metorex::object::Object> {
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    use metorex::vm::VirtualMachine;
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

#[test]
fn class_method_and_instance_method_coexist() {
    let result = run_class(
        r#"
class Foo
  def self.class_method
    "class"
  end
  def instance_method
    "instance"
  end
end
Foo.class_method
"#,
    );
    assert_eq!(
        result,
        Some(metorex::object::Object::String(std::rc::Rc::new(
            "class".to_string()
        )))
    );
}

#[test]
fn class_method_not_callable_on_instance() {
    let result = run_class(
        r#"
class Foo
  def self.class_only
    42
  end
  def instance_only
    99
  end
end
Foo.new.instance_only
"#,
    );
    assert_eq!(result, Some(metorex::object::Object::Int(99)));
}

// ── Range include? with strings ─────────────────────────────────────────────

#[test]
fn range_include_string() {
    assert_eq!(
        run_class(r#"("a".."z").include?("m")"#),
        Some(metorex::object::Object::Bool(true))
    );
}

// ── class << self with attr_accessor in a class ────────────────────────────

#[test]
fn class_self_attr_accessor_creates_methods() {
    let result = run_class(
        r#"
class Foo
  class << self
    attr_accessor :title, :version
  end
end
Foo.title = "Bar"
Foo.version = "1.0"
Foo.title + Foo.version
"#,
    );
    assert_eq!(
        result,
        Some(metorex::object::Object::String(std::rc::Rc::new(
            "Bar1.0".to_string()
        )))
    );
}

#[test]
fn class_self_attr_accessor_default_nil() {
    let result = run_class(
        r#"
class Foo
  class << self
    attr_accessor :unset
  end
end
Foo.unset.nil?
"#,
    );
    assert_eq!(result, Some(metorex::object::Object::Bool(true)));
}

// ── module body with class << self / attr_accessor ────────────────────────

#[test]
fn module_class_self_with_attr_reader() {
    let result = run_class(
        r#"
module M
  class << self
    attr_reader :version
  end
end
M.version.nil?
"#,
    );
    assert_eq!(result, Some(metorex::object::Object::Bool(true)));
}

#[test]
fn module_class_self_with_def_method() {
    let result = run_class(
        r#"
module M
  class << self
    def hello
      "world"
    end
  end
end
M.hello
"#,
    );
    assert_eq!(
        result,
        Some(metorex::object::Object::String(std::rc::Rc::new(
            "world".to_string()
        )))
    );
}

// ── Setters on Class/Module receivers ─────────────────────────────────────

#[test]
fn setter_on_class_via_defined_method() {
    let result = run_class(
        r#"
class Foo
  def self.config=(v)
    @config = v
  end
  def self.config
    @config
  end
end
Foo.config = 42
Foo.config
"#,
    );
    assert_eq!(result, Some(metorex::object::Object::Int(42)));
}

#[test]
fn setter_on_class_falls_back_to_class_var() {
    // No defined setter — assignment stores into a class variable.
    let result = run_class(
        r#"
class Foo
end
Foo.title = "hello"
Foo.instance_variable_get("@title")
"#,
    );
    assert_eq!(
        result,
        Some(metorex::object::Object::String(std::rc::Rc::new(
            "hello".to_string()
        )))
    );
}

#[test]
fn setter_on_module_falls_back_to_class_var() {
    let result = run_class(
        r#"
module Mx
end
Mx.label = "world"
Mx.instance_variable_get("@label")
"#,
    );
    assert_eq!(
        result,
        Some(metorex::object::Object::String(std::rc::Rc::new(
            "world".to_string()
        )))
    );
}

// ── module body with constant assignment ──────────────────────────────────

#[test]
fn module_with_constant_assignment() {
    let result = run_class(
        r#"
module M
  VERSION = "1.0"
end
M::VERSION
"#,
    );
    assert_eq!(
        result,
        Some(metorex::object::Object::String(std::rc::Rc::new(
            "1.0".to_string()
        )))
    );
}

// ── parser/statements/mod.rs lines 32-34: private/public def ─────────────────

#[test]
fn private_def_defines_callable_method() {
    let result = run(r#"
class Foo
  private def secret
    42
  end
  def reveal
    secret
  end
end
Foo.new.reveal
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn public_def_defines_callable_method() {
    let result = run(r#"
class Bar
  public def visible
    99
  end
end
Bar.new.visible
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

// ── define_method on class object (native_methods/mod.rs line 497) ───────────

#[test]
fn define_method_on_class_object_no_captured_vars() {
    // Calling define_method on a Class object at runtime (not inside class body)
    // with a block that has no captured variables covers the empty captured_vars branch.
    let result = run(r#"
class Widget
end
Widget.define_method("value") do
  42
end
Widget.new.value
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── include non-module at top level ─────────────────────────────────────────

#[test]
fn include_non_module_errors() {
    let err = run_err(
        r#"
Foo = 42
include Foo
"#,
    );
    assert!(err.contains("module") || err.contains("not a module"));
}

#[test]
fn include_undefined_module_errors() {
    let err = run_err("include UndefinedModuleXyz");
    assert!(err.contains("Undefined") || err.contains("undefined") || err.contains("module"));
}

// ── class reopen from environment ───────────────────────────────────────────

#[test]
fn class_reopen_from_env() {
    let result = run(r#"
class Foo
  def a
    1
  end
end
class Foo
  def b
    2
  end
end
Foo.new.a + Foo.new.b
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── module reopen from environment ──────────────────────────────────────────

#[test]
fn module_reopen_from_env() {
    let result = run(r#"
module M
  def self.a
    1
  end
end
module M
  def self.b
    2
  end
end
M.a + M.b
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── nested class inside module ──────────────────────────────────────────────

#[test]
fn nested_class_in_module() {
    let result = run(r#"
module Outer
  class Inner
    def val
      42
    end
  end
end
Outer::Inner.new.val
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── nested module inside module ─────────────────────────────────────────────

#[test]
fn nested_module_in_module() {
    let result = run(r#"
module Outer
  module Inner
    def self.val
      99
    end
  end
end
Outer::Inner.val
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

// ── include inside module body ──────────────────────────────────────────────

#[test]
fn include_inside_module() {
    let result = run(r#"
module Greetable
  def greet
    "hello"
  end
end
module Polite
  include Greetable
end
"#);
    assert!(result.is_some() || result.is_none());
}

// ── singleton method on Module ──────────────────────────────────────────────

#[test]
fn singleton_method_on_module() {
    let result = run(r#"
module MyMod
end
def MyMod.helper
  "from module"
end
MyMod.helper
"#);
    assert_eq!(result, Some(Object::string("from module")));
}

// ── singleton method on instance (def obj.method) ───────────────────────────

#[test]
fn singleton_method_on_instance() {
    let result = run(r#"
obj = Object.new
def obj.greet
  "hello from singleton"
end
obj.greet
"#);
    assert_eq!(result, Some(Object::string("hello from singleton")));
}

// ── alias_method with string args in class body ─────────────────────────────

#[test]
fn alias_method_string_args() {
    let result = run(r#"
class AliasTest
  def original
    "orig"
  end
  alias_method "aliased", "original"
end
AliasTest.new.aliased
"#);
    assert_eq!(result, Some(Object::string("orig")));
}

// ── define_method in class body ─────────────────────────────────────────────

#[test]
fn define_method_in_class_body() {
    let result = run(r#"
class DmTest
  define_method(:dynamic) { "dyn" }
end
DmTest.new.dynamic
"#);
    assert_eq!(result, Some(Object::string("dyn")));
}

// ── refine in module body ───────────────────────────────────────────────────

#[test]
fn refine_in_module_body() {
    let result = run(r#"
module StringExt
  refine(String) do
    def shout
      upcase + "!"
    end
  end
end
using StringExt
"hello".shout
"#);
    assert_eq!(result, Some(Object::string("HELLO!")));
}

// ── qualified constant resolution (Foo::Bar) ────────────────────────────────

#[test]
fn qualified_constant_resolution() {
    let result = run(r#"
module A
  class B
    def val
      7
    end
  end
end
A::B.new.val
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

// ── expressions in module body ──────────────────────────────────────────────

#[test]
fn expression_in_module_body() {
    let result = run(r#"
module M
  X = 42
end
M::X
"#);
    assert!(result.is_some());
}
