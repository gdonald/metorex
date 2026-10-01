// Fiddle::Handle: opening a shared library, or the program itself, and
// finding where each symbol in it is loaded.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let source = format!("require 'fiddle'\n({code}).inspect");
    let tokens = Lexer::new(&source).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    match vm.execute_program(&statements).expect("execution failed") {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

/// The class and message of what `code` raises.
fn refusal(code: &str) -> String {
    inspected(&format!(
        "begin\n  {code}\nrescue StandardError => error\n  [error.class, error.message]\nend"
    ))
}

#[test]
fn a_library_that_cannot_be_found_raises_dl_error() {
    assert_eq!(
        inspected(
            "begin\n  Fiddle::Handle.new('doesnotexist.doesnotexist')\nrescue Fiddle::DLError => error\n  error.class.ancestors.take(3)\nend"
        ),
        "[Fiddle::DLError, Fiddle::Error, StandardError]"
    );
}

#[test]
fn the_program_handle_finds_a_c_library_symbol() {
    assert_eq!(
        inspected(
            "held = Fiddle::Handle.new(nil)\n[held.sym('strlen') > 0, held['strlen'] == held.sym('strlen')]"
        ),
        "[true, true]"
    );
}

#[test]
fn the_default_handle_searches_every_loaded_library() {
    assert_eq!(
        inspected("Fiddle::Handle::DEFAULT.sym('strlen') == Fiddle::Handle['strlen']"),
        "true"
    );
}

#[test]
fn an_unknown_symbol_raises_dl_error() {
    assert_eq!(
        refusal("Fiddle::Handle.new(nil).sym('no_such_symbol_here')"),
        "[Fiddle::DLError, \"unknown symbol \\\"no_such_symbol_here\\\"\"]"
    );
}

#[test]
fn a_symbol_name_must_be_a_string() {
    assert_eq!(
        refusal("Fiddle::Handle.new(nil).sym(:strlen)"),
        "[TypeError, \"no implicit conversion of Symbol into String\"]"
    );
}

#[test]
fn a_symbol_name_is_taken_through_to_str() {
    assert_eq!(
        inspected(
            "named = Object.new\ndef named.to_str = 'strlen'\nFiddle::Handle::DEFAULT.sym(named) > 0"
        ),
        "true"
    );
}

#[test]
fn a_null_byte_in_a_library_path_is_refused() {
    assert_eq!(
        refusal("Fiddle::Handle.new(\"a\\0b\")"),
        "[ArgumentError, \"string contains null byte\"]"
    );
}

#[test]
fn a_null_byte_in_a_symbol_name_is_refused() {
    assert_eq!(
        refusal("Fiddle::Handle::DEFAULT.sym(\"a\\0b\")"),
        "[ArgumentError, \"string contains null byte\"]"
    );
}

#[test]
fn closing_twice_and_reading_a_closed_handle_raise() {
    assert_eq!(
        inspected(
            "held = Fiddle::Handle.new(nil)\nfirst = held.close\nsecond = (held.close rescue $!.message)\nthird = (held.sym('strlen') rescue $!.message)\n[first, second, third]"
        ),
        "[0, \"dlclose() called too many times\", \"closed handle\"]"
    );
}

#[test]
fn closing_on_collection_starts_off_and_can_be_switched() {
    assert_eq!(
        inspected(
            "held = Fiddle::Handle.new(nil)\nseen = [held.close_enabled?]\nheld.enable_close\nseen << held.close_enabled?\nheld.disable_close\nseen << held.close_enabled?"
        ),
        "[false, true, false]"
    );
}

#[test]
fn dlopen_answers_a_handle() {
    assert_eq!(
        inspected("held = Fiddle.dlopen(nil)\n[held.class, held.to_i == held.to_ptr]"),
        "[Fiddle::Handle, true]"
    );
}

#[test]
fn the_loader_flags_are_the_platform_numbers() {
    let expected = format!(
        "[{}, {}, {}]",
        libc::RTLD_GLOBAL,
        libc::RTLD_LAZY,
        libc::RTLD_NOW
    );
    assert_eq!(
        inspected("[Fiddle::RTLD_GLOBAL, Fiddle::Handle::RTLD_LAZY, Fiddle::RTLD_NOW]"),
        expected
    );
}
