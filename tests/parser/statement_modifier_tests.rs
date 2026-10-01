// Modifiers written after a statement: several in a row apply left to
// right, and a `rescue` modifier's fallback may be a command where the
// rescued code is a statement or a command of its own.

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

fn parse_fails(code: &str) -> bool {
    Parser::new(Lexer::new(code).tokenize()).parse().is_err()
}

#[test]
fn two_conditional_modifiers_apply_left_to_right() {
    assert_eq!(
        inspected("seen = []\nseen << 1 if true unless false\nseen << 2 if false if true\nseen"),
        "[1]"
    );
}

#[test]
fn a_while_modifier_inside_an_if_modifier_loops() {
    assert_eq!(
        inspected("count = 0\ncount += 1 while count < 3 if true\ncount"),
        "3"
    );
}

#[test]
fn a_rescue_after_a_modifier_condition_rescues_the_whole_statement() {
    assert_eq!(
        inspected("seen = []\nraise if true rescue seen.push 1\nseen"),
        "[1]"
    );
}

#[test]
fn a_rescue_modifier_after_a_command_takes_a_command_fallback() {
    assert_eq!(
        inspected("def shout(*) = raise\nseen = []\nshout 1 rescue seen.push 2\nseen"),
        "[2]"
    );
}

#[test]
fn a_rescue_modifier_after_an_assigned_command_assigns_the_fallback() {
    assert_eq!(
        inspected("def shout(*) = raise\ndef three(*) = 3\nvalue = shout 1 rescue three 4\nvalue"),
        "3"
    );
}

#[test]
fn a_rescue_after_a_paren_less_argument_rescues_the_statement() {
    assert_eq!(
        inspected("seen = []\nseen.push raise rescue seen.push :fallback\nseen"),
        "[:fallback]"
    );
}

#[test]
fn a_command_fallback_after_an_assigned_expression_is_refused() {
    assert!(parse_fails("value = raise rescue p 2"));
}

#[test]
fn a_rescue_modifier_after_a_modifier_rescue_chains() {
    assert_eq!(
        inspected("seen = []\nseen.push(3) rescue seen.push(4) if true\nseen"),
        "[3]"
    );
}

#[test]
fn an_elsif_in_a_statement_takes_then_before_its_branch() {
    assert_eq!(
        inspected(
            "[0, 5, 500].map do |value|\n  if value.zero? then 'zero'\n  elsif value < 10 then 'small'\n  else 'large'\n  end\nend"
        ),
        "[\"zero\", \"small\", \"large\"]"
    );
}

#[test]
fn a_constant_of_an_enclosing_scope_hides_a_top_level_one() {
    assert_eq!(
        inspected(
            "module Library\n  class NameError < StandardError; end\n  String = :ours\n  class Reader\n    def failure = [NameError, String]\n  end\nend\nLibrary::Reader.new.failure"
        ),
        "[Library::NameError, :ours]"
    );
}
