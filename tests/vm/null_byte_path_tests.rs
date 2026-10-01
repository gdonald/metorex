// A path holding a NUL byte is refused before anything is looked up.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn refusal(code: &str) -> Option<Object> {
    let wrapped = format!(
        "begin\n  {code}\n  :accepted\nrescue ArgumentError => error\n  error.message\nend"
    );
    let tokens = Lexer::new(&wrapped).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("execution failed")
}

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

#[test]
fn reading_a_directory_refuses_a_nul_byte() {
    for call in [
        "Dir.entries(\".\\0\")",
        "Dir.children(\".\\0\")",
        "Dir.empty?(\".\\0\")",
        "Dir.foreach(\".\\0\").to_a",
    ] {
        assert_eq!(
            refusal(call),
            text("path name contains null byte"),
            "{call}"
        );
    }
}

#[test]
fn a_list_of_glob_patterns_refuses_a_nul_byte_as_a_path() {
    assert_eq!(
        refusal("Dir.glob([\"a\\0b\"])"),
        text("path name contains null byte")
    );
    assert_eq!(
        refusal("Dir[\"a\", \"b\\0c\"]"),
        text("path name contains null byte")
    );
}

#[test]
fn a_single_glob_pattern_refuses_a_nul_byte_as_a_separator() {
    assert_eq!(
        refusal("Dir.glob(\"a\\0b\")"),
        text("nul-separated glob pattern is deprecated")
    );
}

#[test]
fn fnmatch_refuses_a_nul_byte_in_the_pattern_as_a_string() {
    assert_eq!(
        refusal("File.fnmatch(\"a\\0\", \"a\")"),
        text("string contains null byte")
    );
}

#[test]
fn fnmatch_refuses_a_nul_byte_in_the_path_as_a_path() {
    assert_eq!(
        refusal("File.fnmatch(\"a\", \"a\\0\")"),
        text("path name contains null byte")
    );
}
