// A local named like a method hides the method only where the name stands
// alone. Followed by arguments, the name is a call.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let source = format!("__answered__ = begin\n{code}\nend\n__answered__.inspect");
    let tokens = Lexer::new(&source).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    match vm.execute_program(&statements).expect("execution failed") {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

#[test]
fn a_kernel_function_is_called_while_a_local_of_its_name_exists() {
    assert_eq!(
        inspected("format = 5\n[format('%03d', 7), format]"),
        "[\"007\", 5]"
    );
}

#[test]
fn a_spaced_parenthesis_after_a_local_name_opens_the_first_argument() {
    assert_eq!(
        inspected(
            "def shown(*held) = held\nshown = 3\nn = 12\nresult = shown (n * 2) % 5 == 4, 7\nresult"
        ),
        "[true, 7]"
    );
}

#[test]
fn a_pending_autoload_in_an_enclosing_scope_comes_ahead_of_an_outer_constant() {
    assert_eq!(
        inspected(
            "module Outer\n  class Item; def self.kind = :outer; end\n  module Inner\n    autoload :Item, '/nonexistent/item_file_for_this_test.rb'\n    def self.kind = (Item.kind rescue $!.class)\n  end\nend\nOuter::Inner.kind"
        ),
        "LoadError"
    );
}
