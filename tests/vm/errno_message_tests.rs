// An Errno raised with a message of its own names the reason once, ahead
// of what was being done.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let source = format!("({code}).inspect");
    let tokens = Lexer::new(&source).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    match vm.execute_program(&statements).expect("execution failed") {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

#[test]
fn a_string_io_seek_before_the_start_names_the_reason_once() {
    assert_eq!(
        inspected(
            "require 'stringio'\nio = StringIO.new('abc')\n[(io.seek(-1) rescue $!.message), (io.truncate(-1) rescue $!.message)]"
        ),
        "[\"Invalid argument\", \"Invalid argument - negative length\"]"
    );
}

#[test]
fn opening_a_directory_that_is_not_there_names_the_call_and_the_path() {
    assert_eq!(
        inspected(
            "begin\n  Dir.new('/no/such/dir/here')\nrescue SystemCallError => error\n  error.message\nend"
        ),
        "\"No such file or directory @ dir_initialize - /no/such/dir/here\""
    );
}
