// Coverage tests for CSV

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// Loading and running the CSV library nests deeper than the stack a test
/// thread is given, so each program runs on a thread sized like the one the
/// binary itself uses.
fn on_a_deep_stack(work: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(work)
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

/// The value a program answers, written the way `inspect` writes it.
fn shown(code: &str) -> String {
    let held = format!("__answered__ = begin\n{code}\nend\n__answered__.inspect");
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        match vm.execute_program(&stmts).expect("execution failed") {
            Some(Object::String(written)) => written.to_string(),
            other => panic!("expected a string, got {other:?}"),
        }
    })
}

fn run_err(code: &str) -> String {
    let held = code.to_string();
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        vm.execute_program(&stmts).unwrap_err().to_string()
    })
}

// ── Reading ────────────────────────────────────────────────────────────────

#[test]
fn text_is_read_into_rows_of_fields() {
    assert_eq!(shown("require 'csv'\nCSV.parse('')"), "[]");
    assert_eq!(shown("require 'csv'\nCSV.parse('foo')"), "[[\"foo\"]]");
    assert_eq!(
        shown("require 'csv'\nCSV.parse('foo,bar,baz')"),
        "[[\"foo\", \"bar\", \"baz\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.parse(\"foo,bar\\nbaz,quz\")"),
        "[[\"foo\", \"bar\"], [\"baz\", \"quz\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.parse_line('a,b')"),
        "[\"a\", \"b\"]"
    );
    assert_eq!(shown("require 'csv'\nCSV.parse_line('')"), "nil");
}

#[test]
fn an_empty_field_is_nil_and_a_quoted_empty_one_is_a_string() {
    assert_eq!(
        shown("require 'csv'\nCSV.parse('foo,,baz')"),
        "[[\"foo\", nil, \"baz\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.parse('foo,\"\",baz')"),
        "[[\"foo\", \"\", \"baz\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.parse('foo,')"),
        "[[\"foo\", nil]]"
    );
}

#[test]
fn a_blank_line_is_a_row_with_no_fields() {
    assert_eq!(shown("require 'csv'\nCSV.parse(\"\\n\")"), "[[]]");
    assert_eq!(
        shown("require 'csv'\nCSV.parse(\"\\n\\nbar\")"),
        "[[], [], [\"bar\"]]"
    );
    assert_eq!(shown("require 'csv'\nCSV.parse(\"foo\\n\")"), "[[\"foo\"]]");
    assert_eq!(
        shown("require 'csv'\nCSV.parse(\"\\nfoo\")"),
        "[[], [\"foo\"]]"
    );
}

#[test]
fn a_quoted_field_may_carry_the_separator_and_its_own_quotes() {
    assert_eq!(
        shown("require 'csv'\nCSV.parse('\"Johnson, Dwayne\",actor')"),
        "[[\"Johnson, Dwayne\", \"actor\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.parse('\"say \"\"hi\"\"\"')"),
        "[[\"say \\\"hi\\\"\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.parse(\"\\\"one\\ntwo\\\"\")"),
        "[[\"one\\ntwo\"]]"
    );
}

#[test]
fn another_separator_may_be_named() {
    assert_eq!(
        shown("require 'csv'\nCSV.parse('foo;bar', col_sep: ';')"),
        "[[\"foo\", \"bar\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.parse(\"foo;bar\\nbaz;quz\", col_sep: ';')"),
        "[[\"foo\", \"bar\"], [\"baz\", \"quz\"]]"
    );
}

#[test]
fn text_a_reader_cannot_make_sense_of_is_refused() {
    assert!(
        run_err("require 'csv'\nCSV.parse('\"quoted\" field')")
            .contains("Any value after quoted field")
    );
    assert!(run_err("require 'csv'\nCSV.parse('a\"b')").contains("Illegal quoting"));
    assert!(run_err("require 'csv'\nCSV.parse('\"unclosed')").contains("Unclosed quoted field"));
}

#[test]
fn liberal_parsing_takes_the_text_as_it_stands() {
    assert_eq!(
        shown(
            "require 'csv'\nCSV.parse('\"Johnson, Dwayne\",Dwayne \"The Rock\" Johnson', liberal_parsing: true)"
        ),
        "[[\"Johnson, Dwayne\", \"Dwayne \\\"The Rock\\\" Johnson\"]]"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.new('', liberal_parsing: true).liberal_parsing?"),
        "true"
    );
    assert_eq!(
        shown("require 'csv'\nCSV.new('').liberal_parsing?"),
        "false"
    );
}

// ── Writing ────────────────────────────────────────────────────────────────

#[test]
fn a_row_is_written_back_out_as_a_line() {
    assert_eq!(
        shown("require 'csv'\nCSV.generate_line(['foo', 'bar'])"),
        "\"foo,bar\\n\""
    );
    assert_eq!(shown("require 'csv'\nCSV.generate_line([])"), "\"\\n\"");
    assert_eq!(
        shown("require 'csv'\nCSV.generate_line(['foo', nil, 'bar'])"),
        "\"foo,,bar\\n\""
    );
    assert_eq!(
        shown("require 'csv'\nCSV.generate_line(['foo', 'bar'], col_sep: ';')"),
        "\"foo;bar\\n\""
    );
}

#[test]
fn a_field_that_would_not_read_back_is_quoted() {
    assert_eq!(
        shown("require 'csv'\nCSV.generate_line(['a,b'])"),
        "\"\\\"a,b\\\"\\n\""
    );
    assert_eq!(
        shown("require 'csv'\nCSV.generate_line(['say \"hi\"'])"),
        "\"\\\"say \\\"\\\"hi\\\"\\\"\\\"\\n\""
    );
    assert_eq!(
        shown("require 'csv'\nCSV.generate_line([\"one\\ntwo\"])"),
        "\"\\\"one\\ntwo\\\"\\n\""
    );
}

#[test]
fn rows_pushed_onto_a_sheet_are_read_back_off_it() {
    const SHEET: &str =
        "require 'csv'\nsheet = CSV.new('')\nsheet << ['a', 1]\nsheet.add_row(['b', 2])\n";
    assert_eq!(shown(&format!("{SHEET}sheet.string")), "\"a,1\\nb,2\\n\"");
    assert_eq!(
        shown(&format!("{SHEET}sheet.read")),
        "[[\"a\", \"1\"], [\"b\", \"2\"]]"
    );
    assert_eq!(
        shown(
            "require 'csv'\nfound = []\nCSV.new(\"x,1\\ny,2\").each { |row| found.push(row) }\nfound"
        ),
        "[[\"x\", \"1\"], [\"y\", \"2\"]]"
    );
    assert_eq!(shown("require 'csv'\nCSV.new('').col_sep"), "\",\"");
    assert_eq!(
        shown("require 'csv'\nCSV.new(\"a,b\\n\").shift"),
        "[\"a\", \"b\"]"
    );
    assert_eq!(shown("require 'csv'\nCSV.new('').shift"), "nil");
}
