// Coverage tests for misc native methods (puts, file, define_method, bool)

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

// ── puts with Instance that has no to_s method ───────────────────────────────

#[test]
fn puts_with_instance_no_to_s() {
    // Instance with no to_s - falls back to Display format
    let result = run(r#"
class Nameless
end
obj = Nameless.new
puts obj
"#);
    // Should not error - just prints the default representation
    assert!(result == Some(Object::Nil) || result.is_none());
}

// ── Object::Symbol Display (display.rs line 16) ───────────────────────────────
// puts on a Symbol triggers Display::fmt which formats it as ":name"

#[test]
fn puts_symbol_triggers_display() {
    let result = run(r#"
x = :hello
puts x
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Object::Module Display (display.rs line 44) ───────────────────────────────

#[test]
fn puts_module_triggers_display() {
    let result = run(r#"
module Greetings
end
puts Greetings
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Object::Method Display (display.rs line 45) ───────────────────────────────
// method(:name) returns the Method object; puts on it calls Display

#[test]
fn puts_method_object_triggers_display() {
    let result = run(r#"
def greet
  "hello"
end
puts method(:greet)
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Object::Block Display (display.rs line 46) ────────────────────────────────

#[test]
fn puts_block_object_triggers_display() {
    let result = run(r#"
b = lambda do |x|
  x + 1
end
puts b
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── File.write (native_methods/mod.rs lines 160-162) ────────────────────

#[test]
fn file_write_and_read() {
    let result = run(
        "File.write(\"/tmp/metorex_test_coverage.txt\", \"hello\")\nFile.read(\"/tmp/metorex_test_coverage.txt\")",
    );
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
    let _ = std::fs::remove_file("/tmp/metorex_test_coverage.txt");
}

// ── define_method closure capture (native_methods/mod.rs line 224) ──────

#[test]
fn define_method_closure_capture_in_class() {
    let result =
        run("class Foo\n  define_method(:get_val) do\n    42\n  end\nend\nFoo.new.get_val");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Various dict key types (utils.rs line 29) ───────────────────────────

#[test]
fn int_as_hash_key() {
    let result = run("h = {}\nh[1] = \"one\"\nh[1]");
    assert_eq!(result, Some(Object::String(Rc::new("one".to_string()))));
}

#[test]
fn bool_as_hash_key() {
    let result = run("h = {}\nh[true] = \"yes\"\nh[true]");
    assert_eq!(result, Some(Object::String(Rc::new("yes".to_string()))));
}
