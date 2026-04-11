// Class/define_method/file error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// ══════════════════════════════════════════════════════════════════════════════
// Native methods mod.rs - dispatch fallthrough (lines 160-162, 224)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn file_write_non_string_content() {
    let result = run(r#"
File.write("/tmp/metorex_test_err_cov.txt", 42)
"#);
    // File.write formats non-string content, returns byte count
    assert!(result.is_some());
    let _ = std::fs::remove_file("/tmp/metorex_test_err_cov.txt");
}

#[test]
fn define_method_with_block_on_class() {
    let result = run(r#"
class Dyn
  define_method(:greet) do
    "hello"
  end
end
Dyn.new.greet
"#);
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

#[test]
fn define_method_with_symbol_name() {
    let result = run(r#"
class Sym
  define_method(:calc) do
    100
  end
end
Sym.new.calc
"#);
    assert_eq!(result, Some(Object::Int(100)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Class native methods - name, superclass, ancestors
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn class_name_method() {
    let result = run(r#"
class Animal
end
Animal.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Animal".to_string()))));
}

#[test]
fn class_superclass_method() {
    let result = run(r#"
class Base
end
class Child < Base
end
Child.superclass.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Base".to_string()))));
}

#[test]
fn class_ancestors_method() {
    let result = run(r#"
class A
end
class B < A
end
B.ancestors.length
"#);
    // B and A in ancestors
    assert!(result.is_some());
    if let Some(Object::Int(n)) = result {
        assert!(n >= 2, "Expected at least 2 ancestors, got {}", n);
    }
}

#[test]
fn class_superclass_nil_when_no_parent() {
    let result = run(r#"
class Solo
end
Solo.superclass
"#);
    assert_eq!(result, Some(Object::Nil));
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Class.const_defined? ───────────────────────────────────────────────────

#[test]
fn const_defined_true_for_existing_global() {
    let result = run(r#"
Foo = 42
Object.const_defined?(:Foo)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn const_defined_false_for_missing() {
    let result = run(r#"Object.const_defined?(:NopeNotHere)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn const_defined_with_string_arg() {
    let result = run(r#"
Bar = 99
Object.const_defined?("Bar")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn const_defined_with_non_symbol_returns_false() {
    let result = run(r#"Object.const_defined?(42)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn const_defined_no_args_errors() {
    let err = run_err("Object.const_defined?");
    assert!(err.contains("argument"));
}

// ── File.expand_path error paths ──────────────────────────────────────────

#[test]
fn file_expand_path_no_args_errors() {
    let err = run_err("File.expand_path");
    assert!(err.contains("argument"));
}

#[test]
fn file_expand_path_too_many_args_errors() {
    let err = run_err(r#"File.expand_path("a", "b", "c")"#);
    assert!(err.contains("argument"));
}

#[test]
fn file_expand_path_non_string_arg_errors() {
    let err = run_err("File.expand_path(42)");
    assert!(err.contains("String"));
}

#[test]
fn file_expand_path_non_string_base_errors() {
    let err = run_err(r#"File.expand_path("foo", 42)"#);
    assert!(err.contains("String"));
}

#[test]
fn file_expand_path_with_absolute_base() {
    let result = run(r#"File.expand_path("a.txt", "/tmp")"#);
    if let Some(Object::String(s)) = result {
        assert!(s.contains("a.txt"));
    } else {
        panic!("expected string, got {:?}", result);
    }
}

// ── module_function explicit-name form ─────────────────────────────────────

#[test]
fn module_function_with_symbol_name() {
    // Make `helper` callable on the module itself.
    let result = run(r#"
module Util
  def helper
    "ok"
  end
  module_function :helper
end
Util.helper
"#);
    assert_eq!(result, Some(Object::string("ok")));
}

#[test]
fn module_function_no_args_errors() {
    let err = run_err(
        r#"
class Mod
end
Mod.send(:module_function)
"#,
    );
    assert!(err.contains("argument") || err.contains("undefined"));
}

#[test]
fn module_function_undefined_method_errors() {
    // Send the message directly to bypass the visibility-stub fallthrough.
    let err = run_err(
        r#"
module Util2
end
Util2.send(:module_function, :nonexistent_method)
"#,
    );
    assert!(err.contains("undefined") || err.contains("nonexistent"));
}

#[test]
fn module_function_non_string_arg_errors() {
    let err = run_err(
        r#"
module Util3
end
Util3.send(:module_function, 42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol"));
}

// ── Signal.trap stub ─────────────────────────────────────────────────────

#[test]
fn signal_trap_with_block_is_noop() {
    let result = run(r#"
Signal.trap("INT") { puts "intercepted" }
"#);
    // The block is silently discarded; no output, return nil.
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn signal_trap_returns_nil() {
    let result = run(r#"Signal.trap("INT")"#);
    assert_eq!(result, Some(Object::Nil));
}
