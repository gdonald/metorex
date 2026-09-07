// Coverage tests for StringScanner

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

const HELD: &str = "require 'strscan'\nheld = StringScanner.new(\"This is a test\")\n";

// ── Matching where the cursor stands ───────────────────────────────────────

#[test]
fn scan_takes_what_matches_and_moves_the_cursor() {
    assert_eq!(
        run(&format!("{HELD}held.scan(/\\w+/)")),
        Some(Object::string("This"))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan(/\\w+/)\nheld.pos")),
        Some(Object::Int(4))
    );
    assert_eq!(run(&format!("{HELD}held.scan(/\\d+/)")), Some(Object::Nil));
    assert_eq!(
        run(&format!("{HELD}held.scan(//)")),
        Some(Object::string(""))
    );
}

#[test]
fn scan_takes_a_string_as_the_pattern_itself() {
    assert_eq!(
        run(&format!("{HELD}held.scan(\"This\")")),
        Some(Object::string("This"))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan(\"this\")")),
        Some(Object::Nil)
    );
}

#[test]
fn scan_refuses_a_pattern_it_cannot_read() {
    let err = run_err(&format!("{HELD}held.scan(5)"));
    assert!(err.contains("expected Regexp"), "Error was: {}", err);
}

#[test]
fn check_and_match_look_without_moving() {
    assert_eq!(
        run(&format!("{HELD}held.check(/\\w+/)")),
        Some(Object::string("This"))
    );
    assert_eq!(
        run(&format!("{HELD}held.check(/\\w+/)\nheld.pos")),
        Some(Object::Int(0))
    );
    assert_eq!(
        run(&format!("{HELD}held.match?(/\\w+/)")),
        Some(Object::Int(4))
    );
    assert_eq!(run(&format!("{HELD}held.match?(/\\d/)")), Some(Object::Nil));
}

#[test]
fn skip_moves_the_cursor_and_answers_how_far() {
    assert_eq!(
        run(&format!("{HELD}held.skip(/\\w+/)")),
        Some(Object::Int(4))
    );
    assert_eq!(
        run(&format!("{HELD}held.skip(/\\w+/)\nheld.pos")),
        Some(Object::Int(4))
    );
    assert_eq!(run(&format!("{HELD}held.skip(/\\d/)")), Some(Object::Nil));
}

#[test]
fn an_anchor_reads_from_where_the_cursor_stands() {
    assert_eq!(
        run(&format!(
            "{HELD}held.scan(/\\w+/)\nheld.scan(/^\\d/).inspect"
        )),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan(/\\w+/)\nheld.scan(/\\A\\s/)")),
        Some(Object::string(" "))
    );
}

// ── Searching ahead ────────────────────────────────────────────────────────

#[test]
fn scan_until_takes_everything_through_the_match() {
    assert_eq!(
        run(&format!("{HELD}held.scan_until(/is/)")),
        Some(Object::string("This"))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan_until(/is/)\nheld.pos")),
        Some(Object::Int(4))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan_until(/\\d/)")),
        Some(Object::Nil)
    );
}

#[test]
fn check_until_and_skip_until_and_exist_report_the_same_reach() {
    assert_eq!(
        run(&format!("{HELD}held.check_until(/is/)")),
        Some(Object::string("This"))
    );
    assert_eq!(
        run(&format!("{HELD}held.check_until(/is/)\nheld.pos")),
        Some(Object::Int(0))
    );
    assert_eq!(
        run(&format!("{HELD}held.skip_until(/is/)")),
        Some(Object::Int(4))
    );
    assert_eq!(
        run(&format!("{HELD}held.exist?(/test/)")),
        Some(Object::Int(14))
    );
}

#[test]
fn a_searching_call_refuses_a_string_pattern() {
    let err = run_err(&format!("{HELD}held.scan_until(\"is\")"));
    assert!(err.contains("expected Regexp"), "Error was: {}", err);
    let err = run_err(&format!("{HELD}held.exist?(\"is\")"));
    assert!(err.contains("expected Regexp"), "Error was: {}", err);
}

#[test]
fn the_full_forms_choose_whether_to_move_and_what_to_answer() {
    assert_eq!(
        run(&format!("{HELD}held.scan_full(/\\w+/, true, true)")),
        Some(Object::string("This"))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan_full(/\\w+/, false, false)")),
        Some(Object::Int(4))
    );
    assert_eq!(
        run(&format!(
            "{HELD}held.scan_full(/\\w+/, false, false)\nheld.pos"
        )),
        Some(Object::Int(0))
    );
    assert_eq!(
        run(&format!("{HELD}held.search_full(/is/, true, true)")),
        Some(Object::string("This"))
    );
}

// ── What the last match found ──────────────────────────────────────────────

#[test]
fn the_scanner_reports_what_it_last_matched() {
    let after = format!("{HELD}held.scan(/\\w+/)\n");
    assert_eq!(
        run(&format!("{after}held.matched")),
        Some(Object::string("This"))
    );
    assert_eq!(
        run(&format!("{after}held.matched?")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{after}held.matched_size")),
        Some(Object::Int(4))
    );
    assert_eq!(
        run(&format!("{after}held.pre_match")),
        Some(Object::string(""))
    );
    assert_eq!(
        run(&format!("{after}held.post_match")),
        Some(Object::string(" is a test"))
    );
    assert_eq!(run(&format!("{after}held.size")), Some(Object::Int(1)));
}

#[test]
fn the_scanner_reports_nothing_after_a_match_that_failed() {
    let after = format!("{HELD}held.scan(/\\d/)\n");
    assert_eq!(
        run(&format!("{after}held.matched?")),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!("{after}held.matched.inspect")),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run(&format!("{after}held.pre_match.inspect")),
        Some(Object::string("nil"))
    );
}

#[test]
fn the_scanner_answers_the_groups_the_last_match_held() {
    let dated = "require 'strscan'\nheld = StringScanner.new(\"2024-05\")\nheld.scan(/(?<year>\\d+)-(?<month>\\d+)/)\n";
    assert_eq!(
        run(&format!("{dated}held[0]")),
        Some(Object::string("2024-05"))
    );
    assert_eq!(
        run(&format!("{dated}held[1]")),
        Some(Object::string("2024"))
    );
    assert_eq!(
        run(&format!("{dated}held[:year]")),
        Some(Object::string("2024"))
    );
    assert_eq!(
        run(&format!("{dated}held.captures.inspect")),
        Some(Object::string("[\"2024\", \"05\"]"))
    );
    assert_eq!(
        run(&format!("{dated}held.named_captures.inspect")),
        Some(Object::string(
            "{\"year\" => \"2024\", \"month\" => \"05\"}"
        ))
    );
    assert_eq!(
        run(&format!("{dated}held.values_at(1, 2).inspect")),
        Some(Object::string("[\"2024\", \"05\"]"))
    );
    assert_eq!(run(&format!("{dated}held.size")), Some(Object::Int(3)));
}

#[test]
fn the_scanner_refuses_a_range_as_a_group_index() {
    let err = run_err(&format!("{HELD}held.scan(/\\w+/)\nheld[0..1]"));
    assert!(err.contains("into Integer"), "Error was: {}", err);
}

// ── Where the cursor stands ────────────────────────────────────────────────

#[test]
fn the_cursor_moves_and_reports_where_it_is() {
    assert_eq!(run(&format!("{HELD}held.pos")), Some(Object::Int(0)));
    assert_eq!(
        run(&format!("{HELD}held.pos = 5\nheld.pos")),
        Some(Object::Int(5))
    );
    assert_eq!(
        run(&format!("{HELD}held.pos = -1\nheld.pos")),
        Some(Object::Int(13))
    );
    assert_eq!(
        run(&format!("{HELD}held.pointer = 2\nheld.pointer")),
        Some(Object::Int(2))
    );
    assert_eq!(run(&format!("{HELD}held.charpos")), Some(Object::Int(0)));
}

#[test]
fn the_cursor_refuses_a_place_outside_the_string() {
    let err = run_err(&format!("{HELD}held.pos = 100"));
    assert!(err.contains("out of range"), "Error was: {}", err);
}

#[test]
fn the_scanner_reports_what_is_left_and_whether_it_is_done() {
    assert_eq!(
        run(&format!("{HELD}held.scan(/This /)\nheld.rest")),
        Some(Object::string("is a test"))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan(/This /)\nheld.rest_size")),
        Some(Object::Int(9))
    );
    assert_eq!(run(&format!("{HELD}held.eos?")), Some(Object::Bool(false)));
    assert_eq!(run(&format!("{HELD}held.rest?")), Some(Object::Bool(true)));
    assert_eq!(
        run(&format!("{HELD}held.terminate.eos?")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan(/T/)\nheld.reset.pos")),
        Some(Object::Int(0))
    );
}

#[test]
fn the_scanner_says_whether_it_stands_at_the_start_of_a_line() {
    assert_eq!(
        run(&format!("{HELD}held.beginning_of_line?")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan(/T/)\nheld.bol?")),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("require 'strscan'\nheld = StringScanner.new(\"a\\nb\")\nheld.scan(/a\\n/)\nheld.bol?"),
        Some(Object::Bool(true))
    );
}

#[test]
fn unscan_puts_the_cursor_back_where_the_last_scan_started() {
    assert_eq!(
        run(&format!("{HELD}held.scan(/\\w+/)\nheld.unscan.pos")),
        Some(Object::Int(0))
    );
    let err = run_err(&format!("{HELD}held.unscan"));
    assert!(err.contains("previous match record"), "Error was: {}", err);
}

// ── Reading a piece at a time ──────────────────────────────────────────────

#[test]
fn the_scanner_reads_one_piece_at_a_time() {
    let held = "require 'strscan'\nheld = StringScanner.new(\"abc\")\n";
    assert_eq!(run(&format!("{held}held.getch")), Some(Object::string("a")));
    assert_eq!(
        run(&format!("{held}held.peek(2)")),
        Some(Object::string("ab"))
    );
    assert_eq!(run(&format!("{held}held.peek_byte")), Some(Object::Int(97)));
    assert_eq!(run(&format!("{held}held.scan_byte")), Some(Object::Int(97)));
    assert_eq!(
        run(&format!("{held}held.terminate\nheld.getch.inspect")),
        Some(Object::string("nil"))
    );
}

#[test]
fn peek_refuses_a_negative_length() {
    let err = run_err(&format!("{HELD}held.peek(-1)"));
    assert!(err.contains("negative string size"), "Error was: {}", err);
}

// ── What the scanner holds ─────────────────────────────────────────────────

#[test]
fn the_scanner_takes_on_a_new_string_and_starts_over() {
    assert_eq!(
        run(&format!(
            "{HELD}held.scan(/T/)\nheld.string = \"other\"\nheld.pos"
        )),
        Some(Object::Int(0))
    );
    assert_eq!(
        run(&format!("{HELD}held.string")),
        Some(Object::string("This is a test"))
    );
    assert_eq!(
        run(&format!("{HELD}(held << \" case\").string")),
        Some(Object::string("This is a test case"))
    );
}

#[test]
fn the_scanner_shows_where_it_stands() {
    assert_eq!(
        run(&format!("{HELD}held.inspect")),
        Some(Object::string("#<StringScanner 0/14 @ \"This ...\">"))
    );
    assert_eq!(
        run(&format!("{HELD}held.scan_until(/is/)\nheld.inspect")),
        Some(Object::string(
            "#<StringScanner 4/14 \"This\" @ \" is a...\">"
        ))
    );
    assert_eq!(
        run(&format!("{HELD}held.terminate\nheld.inspect")),
        Some(Object::string("#<StringScanner fin>"))
    );
}

#[test]
fn a_string_subclass_is_scanned_as_a_plain_string() {
    assert_eq!(
        run(
            "require 'strscan'\nclass Held < String\nend\nStringScanner.new(Held.new(\"abc\")).scan(/a/)"
        ),
        Some(Object::string("a"))
    );
}
