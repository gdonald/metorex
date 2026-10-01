// The code inside #{...} is a run of statements, so modifiers, semicolons,
// and line breaks there mean what they mean anywhere else.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("execution failed")
}

#[test]
fn a_modifier_inside_an_interpolation_applies_to_its_statement() {
    assert_eq!(
        run("count = 1\n\"keyword#{'s' if count > 1}|#{3 unless true}\""),
        Some(Object::string("keyword|"))
    );
}

#[test]
fn the_last_statement_inside_an_interpolation_gives_its_value() {
    assert_eq!(
        run("count = 1\n\"#{count += 1; count * 10}\""),
        Some(Object::string("20"))
    );
}

#[test]
fn an_interpolation_may_run_across_lines() {
    assert_eq!(
        run("\"total: #{\n  doubled = 2 * 2\n  doubled + 1\n}\""),
        Some(Object::string("total: 5"))
    );
}

#[test]
fn an_empty_statement_list_inside_an_interpolation_gives_nothing() {
    assert_eq!(run("\"a#{;}b\""), Some(Object::string("ab")));
}
