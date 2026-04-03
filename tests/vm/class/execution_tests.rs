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
fn module_body_unsupported_statement_error() {
    let err = run_err(
        r#"
module M
  if true
    42
  end
end
"#,
    );
    assert!(err.contains("module") || err.contains("Unsupported") || err.contains("statement"));
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
