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
