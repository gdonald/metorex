// Coverage tests for StringIO, String#index, and the octal string escape

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

const HELD: &str = "require 'stringio'\nheld = StringIO.new(\"example\")\n";

// ── Reading one piece at a time ────────────────────────────────────────────

#[test]
fn a_string_io_reads_characters_and_bytes() {
    assert_eq!(run(&format!("{HELD}held.getc")), Some(Object::string("e")));
    assert_eq!(run(&format!("{HELD}held.getbyte")), Some(Object::Int(101)));
    assert_eq!(
        run(&format!("{HELD}held.readchar")),
        Some(Object::string("e"))
    );
    assert_eq!(run(&format!("{HELD}held.readbyte")), Some(Object::Int(101)));
    assert_eq!(
        run(&format!("{HELD}held.pos = 7\nheld.getc.inspect")),
        Some(Object::string("nil"))
    );
}

#[test]
fn a_string_io_reading_past_the_end_says_so() {
    let err = run_err(&format!("{HELD}held.pos = 7\nheld.readchar"));
    assert!(err.contains("end of file"), "Error was: {}", err);
    let err = run_err(&format!("{HELD}held.pos = 7\nheld.readbyte"));
    assert!(err.contains("end of file"), "Error was: {}", err);
}

#[test]
fn a_string_io_walks_its_characters_bytes_and_codepoints() {
    assert_eq!(
        run("require 'stringio'\nStringIO.new(\"abc\").each_char.to_a.inspect"),
        Some(Object::string("[\"a\", \"b\", \"c\"]"))
    );
    assert_eq!(
        run("require 'stringio'\nStringIO.new(\"abc\").each_byte.to_a.inspect"),
        Some(Object::string("[97, 98, 99]"))
    );
    assert_eq!(
        run("require 'stringio'\nStringIO.new(\"abc\").each_codepoint.to_a.inspect"),
        Some(Object::string("[97, 98, 99]"))
    );
}

#[test]
fn a_string_io_reads_lines_and_counts_them() {
    let lines = "require 'stringio'\nheld = StringIO.new(\"first\\nsecond\\n\")\n";
    assert_eq!(
        run(&format!("{lines}held.gets")),
        Some(Object::string("first\n"))
    );
    assert_eq!(
        run(&format!("{lines}held.gets\nheld.lineno")),
        Some(Object::Int(1))
    );
    assert_eq!(
        run(&format!("{lines}held.readlines.inspect")),
        Some(Object::string("[\"first\\n\", \"second\\n\"]"))
    );
    assert_eq!(
        run(&format!("{lines}held.each_line.to_a.size")),
        Some(Object::Int(2))
    );
    assert_eq!(
        run(&format!("{lines}held.gets\nheld.rewind\nheld.lineno")),
        Some(Object::Int(0))
    );
}

#[test]
fn a_string_io_reads_a_paragraph_for_a_blank_separator() {
    assert_eq!(
        run("require 'stringio'\nStringIO.new(\"a\\n\\nb\\n\").each_line(\"\").to_a.inspect"),
        Some(Object::string("[\"a\\n\\n\", \"b\\n\"]"))
    );
}

#[test]
fn a_string_io_puts_a_character_back() {
    assert_eq!(
        run(&format!("{HELD}held.getc\nheld.ungetc(\"z\")\nheld.string")),
        Some(Object::string("zxample"))
    );
    assert_eq!(
        run(&format!("{HELD}held.getc\nheld.ungetc(\"z\")\nheld.pos")),
        Some(Object::Int(0))
    );
}

// ── Writing ────────────────────────────────────────────────────────────────

#[test]
fn a_string_io_writes_prints_and_puts() {
    let empty = "require 'stringio'\nheld = StringIO.new\n";
    assert_eq!(
        run(&format!("{empty}held.write(\"one\")\nheld.string")),
        Some(Object::string("one"))
    );
    assert_eq!(
        run(&format!("{empty}held.print(\"a\", \"b\")\nheld.string")),
        Some(Object::string("ab"))
    );
    assert_eq!(
        run(&format!("{empty}held.puts(\"a\")\nheld.string")),
        Some(Object::string("a\n"))
    );
    assert_eq!(
        run(&format!("{empty}held.putc(33)\nheld.string")),
        Some(Object::string("!"))
    );
    assert_eq!(
        run(&format!("{empty}(held << \"a\" << \"b\").string")),
        Some(Object::string("ab"))
    );
    assert_eq!(
        run(&format!("{empty}held.printf(\"%d-%d\", 1, 2)\nheld.string")),
        Some(Object::string("1-2"))
    );
}

#[test]
fn a_string_io_pads_with_nulls_when_it_writes_past_the_end() {
    assert_eq!(
        run(&format!(
            "{HELD}held.pos = 10\nheld << \"x\"\nheld.string.bytes.inspect"
        )),
        Some(Object::string(
            "[101, 120, 97, 109, 112, 108, 101, 0, 0, 0, 120]"
        ))
    );
}

#[test]
fn a_string_io_writes_an_array_one_line_per_element() {
    assert_eq!(
        run("require 'stringio'\nheld = StringIO.new\nheld.puts([1, 2])\nheld.string"),
        Some(Object::string("1\n2\n"))
    );
}

#[test]
fn a_string_io_refuses_to_write_when_it_was_opened_for_reading() {
    let err = run_err("require 'stringio'\nStringIO.new(\"x\", \"r\").write(\"y\")");
    assert!(err.contains("not opened for writing"), "Error was: {}", err);
    let err = run_err("require 'stringio'\nStringIO.new(\"x\", \"w\").getc");
    assert!(err.contains("not opened for reading"), "Error was: {}", err);
}

// ── Moving about and reshaping ─────────────────────────────────────────────

#[test]
fn a_string_io_seeks_from_the_start_the_position_or_the_end() {
    assert_eq!(
        run(&format!("{HELD}held.seek(2)\nheld.pos")),
        Some(Object::Int(2))
    );
    assert_eq!(
        run(&format!("{HELD}held.seek(2)\nheld.seek(1, 1)\nheld.pos")),
        Some(Object::Int(3))
    );
    assert_eq!(
        run(&format!("{HELD}held.seek(-1, 2)\nheld.pos")),
        Some(Object::Int(6))
    );
    assert_eq!(run(&format!("{HELD}held.seek(2)")), Some(Object::Int(0)));
}

#[test]
fn a_string_io_refuses_a_seek_it_cannot_read() {
    let err = run_err(&format!("{HELD}held.seek(Object.new)"));
    assert!(err.contains("into Integer"), "Error was: {}", err);
    let err = run_err(&format!("{HELD}held.seek(-1)"));
    assert!(err.contains("Invalid argument"), "Error was: {}", err);
}

#[test]
fn a_string_io_truncates_and_replaces_what_it_holds() {
    assert_eq!(
        run(&format!("{HELD}held.truncate(4)\nheld.string")),
        Some(Object::string("exam"))
    );
    assert_eq!(
        run(&format!(
            "{HELD}held.pos = 3\nheld.string = \"other\"\nheld.pos"
        )),
        Some(Object::Int(0))
    );
}

#[test]
fn a_string_io_refuses_a_truncation_it_cannot_read() {
    let err = run_err(&format!("{HELD}held.truncate(Object.new)"));
    assert!(err.contains("into Integer"), "Error was: {}", err);
    let err = run_err("require 'stringio'\nStringIO.new(\"x\", \"r\").truncate(0)");
    assert!(err.contains("not opened for writing"), "Error was: {}", err);
}

#[test]
fn a_string_io_shows_itself_by_class_and_address_alone() {
    assert_eq!(
        run(&format!("{HELD}held.inspect.include?(\"example\")")),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!("{HELD}held.inspect == held.to_s")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!(
            "{HELD}held.inspect.start_with?(\"#<StringIO:0x\")"
        )),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_string_io_opened_with_a_block_closes_when_the_block_ends() {
    assert_eq!(
        run(
            "require 'stringio'\nheld = nil\nStringIO.open(\"x\") { |io| held = io }\nheld.closed?"
        ),
        Some(Object::Bool(true))
    );
}

// ── Where a substring sits ─────────────────────────────────────────────────

#[test]
fn string_index_answers_where_a_needle_first_sits() {
    assert_eq!(run("\"hello\".index(\"l\")"), Some(Object::Int(2)));
    assert_eq!(run("\"hello\".index(\"l\", 3)"), Some(Object::Int(3)));
    assert_eq!(run("\"hello\".index(\"z\")"), Some(Object::Nil));
    assert_eq!(run("\"hello\".index(/l+/)"), Some(Object::Int(2)));
    assert_eq!(run("\"abc\".index(\"\")"), Some(Object::Int(0)));
    assert_eq!(run("\"héllo\".index(\"l\")"), Some(Object::Int(2)));
}

#[test]
fn string_rindex_answers_where_a_needle_last_sits() {
    assert_eq!(run("\"hello\".rindex(\"l\")"), Some(Object::Int(3)));
    assert_eq!(run("\"hello\".rindex(\"l\", 2)"), Some(Object::Int(2)));
    assert_eq!(run("\"hello\".rindex(\"z\")"), Some(Object::Nil));
}

#[test]
fn string_index_refuses_a_needle_it_cannot_read() {
    let err = run_err("\"hello\".index(5)");
    assert!(err.contains("String"), "Error was: {}", err);
}

// ── An octal escape runs to three digits ───────────────────────────────────

#[test]
fn an_octal_escape_reads_up_to_three_digits() {
    assert_eq!(
        run("\"a\\000b\".bytes.inspect"),
        Some(Object::string("[97, 0, 98]"))
    );
    assert_eq!(run("\"\\101\".bytes.inspect"), Some(Object::string("[65]")));
    assert_eq!(run("\"\\0\".bytes.inspect"), Some(Object::string("[0]")));
    assert_eq!(run("\"\\7\".bytes.inspect"), Some(Object::string("[7]")));
}

// ── An argument may assign ─────────────────────────────────────────────────

#[test]
fn an_argument_may_assign_and_pass_what_it_assigned() {
    assert_eq!(
        run("def take(value)\n  value\nend\ntake(held = 5)\nheld"),
        Some(Object::Int(5))
    );
    assert_eq!(
        run("def take(value)\n  value\nend\ntake(held = 5)"),
        Some(Object::Int(5))
    );
}
