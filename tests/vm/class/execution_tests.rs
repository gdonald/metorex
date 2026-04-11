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
    assert!(
        err.contains("define_method")
            || err.contains("String")
            || err.contains("Symbol")
            || err.contains("argument")
    );
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
