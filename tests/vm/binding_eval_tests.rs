// Code run through a binding runs where the binding was taken: in its frame,
// defining methods where a `def` written there would, against its self.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let tokens = Lexer::new(&format!("({code}).inspect")).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    match vm.execute_program(&statements).expect("execution failed") {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

#[test]
fn a_def_run_from_a_method_through_a_top_level_binding_is_a_private_method_of_object() {
    assert_eq!(
        inspected(
            "def runner(held) = held.eval('def via_runner = 1')\nrunner(binding)\n[Object.private_method_defined?(:via_runner), via_runner]"
        ),
        "[true, 1]"
    );
}

#[test]
fn a_def_run_through_a_binding_taken_in_an_instance_method_lands_on_its_class() {
    assert_eq!(
        inspected(
            "class Shelf\n  def held = binding\nend\nShelf.new.held.eval('def added = 2')\nShelf.public_method_defined?(:added)"
        ),
        "true"
    );
}

#[test]
fn a_def_in_a_block_at_the_top_level_is_private() {
    assert_eq!(
        inspected("1.times { def in_block; end }\nObject.private_method_defined?(:in_block)"),
        "true"
    );
}

#[test]
fn a_def_run_inside_a_method_is_public() {
    assert_eq!(
        inspected("def outer\n  def inner; end\nend\nouter\nObject.public_method_defined?(:inner)"),
        "true"
    );
}

#[test]
fn an_error_raised_through_a_binding_names_the_frame_the_binding_was_taken_in() {
    assert_eq!(
        inspected(
            "def runner(held)\n  [1].each { held.eval(\"raise 'x'\", 'written', 4) }\nrescue => error\n  error.backtrace.first\nend\nrunner(binding)"
        ),
        "\"written:4:in '<main>'\""
    );
}

#[test]
fn an_error_raised_through_a_binding_taken_in_a_method_names_that_method() {
    assert_eq!(
        inspected(
            "def taken = binding\nbegin\n  taken.eval(\"raise 'x'\", 'written', 2)\nrescue => error\n  error.backtrace.first\nend"
        ),
        "\"written:2:in 'Object#taken'\""
    );
}

#[test]
fn a_missing_name_looked_up_through_a_top_level_binding_is_reported_for_main() {
    assert_eq!(
        inspected(
            "def asker(held)\n  held.eval('missing_there')\nrescue NameError => error\n  error.message\nend\nasker(binding)"
        ),
        "\"undefined local variable or method 'missing_there' for main\""
    );
}

#[test]
fn a_binding_copied_with_dup_keeps_its_frame() {
    assert_eq!(
        inspected(
            "def taken = binding\nbegin\n  taken.dup.eval(\"raise 'x'\", 'written', 3)\nrescue => error\n  error.backtrace.first\nend"
        ),
        "\"written:3:in 'Object#taken'\""
    );
}
