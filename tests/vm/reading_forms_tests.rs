// Coverage tests for the forms the parser reads and the libraries metorex
// carries

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

// ── A control-flow form read for its value ─────────────────────────────────

#[test]
fn a_case_read_for_its_value_chains_onto_what_it_answered() {
    assert_eq!(
        run("case 1\nwhen 1 then :matched\nend.to_s"),
        Some(Object::string("matched"))
    );
}

#[test]
fn an_unless_read_for_its_value_chains_onto_what_it_answered() {
    assert_eq!(
        run("unless false\n  :kept\nend.to_s"),
        Some(Object::string("kept"))
    );
}

#[test]
fn a_begin_read_for_its_value_chains_onto_what_it_answered() {
    assert_eq!(
        run("begin\n  :held\nrescue\n  :caught\nend.to_s"),
        Some(Object::string("held"))
    );
}

#[test]
fn a_loop_answers_nil_and_may_be_chained_onto() {
    assert_eq!(
        run("counted = 0\nwhile counted < 3\n  counted += 1\nend.inspect"),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run("counted = 0\nuntil counted > 2\n  counted += 1\nend.inspect"),
        Some(Object::string("nil"))
    );
}

#[test]
fn then_may_hold_its_body_on_the_next_line() {
    assert_eq!(run("if true then\n  1\nend"), Some(Object::Int(1)));
    assert_eq!(run("unless false then\n  2\nend"), Some(Object::Int(2)));
}

// ── Empty and multi-statement groups ───────────────────────────────────────

#[test]
fn an_empty_pair_of_parentheses_answers_nil() {
    assert_eq!(run("()"), Some(Object::Nil));
    assert_eq!(
        run("[0, (), 2].inspect"),
        Some(Object::string("[0, nil, 2]"))
    );
    assert_eq!(
        run("{() => ()}.inspect"),
        Some(Object::string("{nil => nil}"))
    );
    assert_eq!(run("if ()\n  1\nelse\n  2\nend"), Some(Object::Int(2)));
}

#[test]
fn a_group_of_statements_answers_the_last_one() {
    assert_eq!(run("(1; 2; 3)"), Some(Object::Int(3)));
    assert_eq!(run("held = (4; 5)\nheld"), Some(Object::Int(5)));
}

// ── Percent literals ───────────────────────────────────────────────────────

#[test]
fn a_capital_w_list_fills_in_its_words() {
    assert_eq!(
        run("count = 3\nfilled = %W(a #{count} c)\nfilled.inspect"),
        Some(Object::string("[\"a\", \"3\", \"c\"]"))
    );
}

#[test]
fn a_capital_i_list_fills_in_its_symbols() {
    assert_eq!(
        run("count = 3\nfilled = %I(a b#{count})\nfilled.inspect"),
        Some(Object::string("[:a, :b3]"))
    );
}

#[test]
fn a_lowercase_list_leaves_its_words_alone() {
    assert_eq!(
        run("plain = %w(x y)\nplain.inspect"),
        Some(Object::string("[\"x\", \"y\"]"))
    );
    assert_eq!(
        run("plain = %i(x y)\nplain.inspect"),
        Some(Object::string("[:x, :y]"))
    );
}

#[test]
fn a_lowercase_q_string_reads_no_interpolation() {
    assert_eq!(run("raw = %q{a#{1}b}\nraw"), Some(Object::string("a#{1}b")));
    assert_eq!(run("raw = %q(a\\(b)\nraw"), Some(Object::string("a(b")));
    assert_eq!(run("raw = %q[a\\\\b]\nraw"), Some(Object::string("a\\b")));
}

// ── A chain written with the dot leading the line ──────────────────────────

#[test]
fn a_chain_may_lead_the_next_line_with_its_dot() {
    assert_eq!(
        run("[1, 2, 3]\n  .map { |n| n * 2 }\n  .first"),
        Some(Object::Int(2))
    );
}

#[test]
fn a_comment_may_sit_between_a_chain_and_its_next_call() {
    assert_eq!(
        run("\"ab\"\n  # what follows is the call\n  .upcase"),
        Some(Object::string("AB"))
    );
}

// ── A call that steps over a nil receiver ──────────────────────────────────

#[test]
fn a_safe_call_answers_nil_for_a_nil_receiver() {
    assert_eq!(run("nil&.length"), Some(Object::Nil));
    assert_eq!(run("nil&.upcase"), Some(Object::Nil));
    assert_eq!(run("nil&.map { |n| n }"), Some(Object::Nil));
    assert_eq!(run("[][10]&.length"), Some(Object::Nil));
}

#[test]
fn a_safe_call_runs_the_method_for_anything_else() {
    assert_eq!(run("\"hello\"&.length"), Some(Object::Int(5)));
    assert_eq!(
        run("[1, 2]&.map { |n| n + 1 }.inspect"),
        Some(Object::string("[2, 3]"))
    );
    assert_eq!(run("\"hi\"&.sub(\"h\", \"H\")"), Some(Object::string("Hi")));
}

// ── A quoted symbol names a method to alias ────────────────────────────────

#[test]
fn alias_takes_a_quoted_symbol_for_either_name() {
    assert_eq!(
        run("class Held\n  def one\n    1\n  end\n  alias :'two' :'one'\nend\nHeld.new.two"),
        Some(Object::Int(1))
    );
}

// ── The libraries metorex carries ──────────────────────────────────────────

#[test]
fn base64_encodes_and_decodes() {
    assert_eq!(
        run("require 'base64'\nBase64.strict_encode64(\"hello\")"),
        Some(Object::string("aGVsbG8="))
    );
    assert_eq!(
        run("require 'base64'\nBase64.strict_decode64(\"aGVsbG8=\")"),
        Some(Object::string("hello"))
    );
    assert_eq!(
        run("require 'base64'\nBase64.encode64(\"a\" * 50)"),
        Some(Object::string(
            "YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFh\nYWFhYWE=\n"
        ))
    );
    assert_eq!(
        run("require 'base64'\nBase64.urlsafe_encode64(\"hello?\", padding: false)"),
        Some(Object::string("aGVsbG8_"))
    );
}

#[test]
fn requiring_a_carried_library_twice_runs_it_once() {
    assert_eq!(
        run("require 'base64'\nrequire 'base64'"),
        Some(Object::Bool(false))
    );
}

#[test]
fn shellwords_splits_and_escapes() {
    assert_eq!(
        run("require 'shellwords'\nShellwords.shellsplit(\"a 'b c' d\").inspect"),
        Some(Object::string("[\"a\", \"b c\", \"d\"]"))
    );
    assert_eq!(
        run("require 'shellwords'\nShellwords.shellescape(\"a b\")"),
        Some(Object::string("a\\ b"))
    );
    assert_eq!(
        run("require 'shellwords'\nShellwords.shellescape(\"\")"),
        Some(Object::string("''"))
    );
}

#[test]
fn abbrev_answers_the_unambiguous_beginnings() {
    assert_eq!(
        run("require 'abbrev'\nAbbrev.abbrev([\"ruby\"]).keys.inspect"),
        Some(Object::string("[\"r\", \"ru\", \"rub\", \"ruby\"]"))
    );
}

#[test]
fn a_singleton_class_has_one_instance() {
    let held = "require 'singleton'\nclass Held\n  include Singleton\nend\n";
    assert_eq!(
        run(&format!("{held}Held.instance.equal?(Held.instance)")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!(
            "{held}begin\n  Held.new\nrescue NoMethodError\n  :refused\nend"
        )),
        Some(Object::symbol("refused".to_string()))
    );
    assert_eq!(
        run(&format!(
            "{held}begin\n  Held.instance.dup\nrescue TypeError\n  :refused\nend"
        )),
        Some(Object::symbol("refused".to_string()))
    );
}

// ── The `not` keyword ──────────────────────────────────────────────────────

#[test]
fn not_takes_what_the_parentheses_hold_and_nothing_more() {
    assert_eq!(run("not(true).to_s"), Some(Object::string("false")));
    assert_eq!(run("not(false).to_s"), Some(Object::string("true")));
    assert_eq!(run("(not true)"), Some(Object::Bool(false)));
    assert_eq!(run("not true"), Some(Object::Bool(false)));
}

// ── A private method is not one the object answers to ──────────────────────

#[test]
fn methods_leaves_out_the_private_ones() {
    let held = "class Held\n  def shown; end\n  private\n  def hidden; end\nend\n";
    assert_eq!(
        run(&format!("{held}Held.new.methods.include?(:hidden)")),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!("{held}Held.new.methods.include?(:shown)")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!(
            "{held}Held.private_instance_methods(false).inspect"
        )),
        Some(Object::string("[:hidden]"))
    );
}

// ── Percent literals with other delimiters ─────────────────────────────────

#[test]
fn a_percent_string_takes_any_punctuation_as_its_delimiter() {
    assert_eq!(run("held = %!hey!\nheld"), Some(Object::string("hey")));
    assert_eq!(run("held = %@hey@\nheld"), Some(Object::string("hey")));
    assert_eq!(run("held = %#hey#\nheld"), Some(Object::string("hey")));
    assert_eq!(run("held = %_hey_\nheld"), Some(Object::string("hey")));
}

#[test]
fn a_percent_string_fills_in_its_interpolation() {
    assert_eq!(
        run("count = 2\nheld = %(a #{count} b)\nheld"),
        Some(Object::string("a 2 b"))
    );
    assert_eq!(
        run("count = 2\nheld = %Q{x#{count}y}\nheld"),
        Some(Object::string("x2y"))
    );
}

#[test]
fn a_percent_still_divides_where_a_value_precedes_it() {
    assert_eq!(run("7 % 3"), Some(Object::Int(1)));
    assert_eq!(run("held = \"%d\"\nheld % 5"), Some(Object::string("5")));
}

#[test]
fn a_percent_x_literal_runs_its_text_as_a_command() {
    assert_eq!(
        run("held = %x(echo hi)\nheld"),
        Some(Object::string("hi\n"))
    );
}

// ── A group holding one expression with a modifier ─────────────────────────

#[test]
fn a_group_may_hold_an_expression_with_a_trailing_modifier() {
    assert_eq!(run("(123 if true)"), Some(Object::Int(123)));
    assert_eq!(run("(123 if false).inspect"), Some(Object::string("nil")));
    assert_eq!(
        run("count = 0\n(count += 1 until count > 4)\ncount"),
        Some(Object::Int(5))
    );
}

// ── Code named after where it was written ──────────────────────────────────

#[test]
fn eval_names_its_code_after_the_place_it_was_written() {
    assert_eq!(
        run("eval(\"__FILE__\").start_with?(\"(eval at \")"),
        Some(Object::Bool(true))
    );
}

// ── More libraries metorex carries ─────────────────────────────────────────

#[test]
fn an_observable_tells_its_observers_when_it_changed() {
    let held = "require 'observer'\nclass Ticker\n  include Observable\n  def tick(value)\n    changed\n    notify_observers(value)\n  end\nend\nclass Watcher\n  attr_reader :seen\n  def update(value)\n    @seen = value\n  end\nend\nticker = Ticker.new\nwatcher = Watcher.new\n";
    assert_eq!(
        run(&format!("{held}ticker.count_observers")),
        Some(Object::Int(0))
    );
    assert_eq!(
        run(&format!(
            "{held}ticker.add_observer(watcher)\nticker.tick(:moved)\nwatcher.seen.to_s"
        )),
        Some(Object::string("moved"))
    );
    assert_eq!(
        run(&format!(
            "{held}ticker.add_observer(watcher)\nticker.delete_observers\nticker.tick(:moved)\nwatcher.seen.inspect"
        )),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run(&format!(
            "{held}2.times {{ ticker.add_observer(watcher) }}\nticker.count_observers"
        )),
        Some(Object::Int(1))
    );
}

#[test]
fn an_observable_says_nothing_until_it_has_changed() {
    let held = "require 'observer'\nclass Quiet\n  include Observable\nend\nheld = Quiet.new\n";
    assert_eq!(
        run(&format!("{held}held.changed?")),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!("{held}held.changed\nheld.changed?")),
        Some(Object::Bool(true))
    );
}

#[test]
fn securerandom_draws_values_of_the_asked_for_length() {
    assert_eq!(
        run("require 'securerandom'\nSecureRandom.hex.length"),
        Some(Object::Int(32))
    );
    assert_eq!(
        run("require 'securerandom'\nSecureRandom.hex(5).length"),
        Some(Object::Int(10))
    );
    assert_eq!(
        run("require 'securerandom'\nSecureRandom.hex(0)"),
        Some(Object::string(""))
    );
    assert_eq!(
        run("require 'securerandom'\nSecureRandom.uuid.length"),
        Some(Object::Int(36))
    );
    assert_eq!(
        run("require 'securerandom'\nSecureRandom.random_number(10).class.name"),
        Some(Object::string("Integer"))
    );
    assert_eq!(
        run("require 'securerandom'\nSecureRandom.random_number.class.name"),
        Some(Object::string("Float"))
    );
}

#[test]
fn securerandom_refuses_a_negative_length() {
    let err = run_err("require 'securerandom'\nSecureRandom.hex(-1)");
    assert!(err.contains("negative string size"), "Error was: {}", err);
}

// ── What a StringIO reports about itself ───────────────────────────────────

#[test]
fn a_string_io_reports_its_length_and_position() {
    let held = "require 'stringio'\nheld = StringIO.new(\"example\")\n";
    assert_eq!(run(&format!("{held}held.size")), Some(Object::Int(7)));
    assert_eq!(run(&format!("{held}held.length")), Some(Object::Int(7)));
    assert_eq!(run(&format!("{held}held.getc")), Some(Object::string("e")));
    assert_eq!(
        run(&format!("{held}held.getc\nheld.pos")),
        Some(Object::Int(1))
    );
    assert_eq!(
        run(&format!("{held}held.read(7)\nheld.tell")),
        Some(Object::Int(7))
    );
    assert_eq!(run(&format!("{held}held.eof?")), Some(Object::Bool(false)));
    assert_eq!(
        run(&format!("{held}held.read(7)\nheld.eof?")),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_string_io_stands_in_for_a_file_without_being_one() {
    let held = "require 'stringio'\nheld = StringIO.new(\"example\")\n";
    assert_eq!(
        run(&format!("{held}held.fileno.inspect")),
        Some(Object::string("nil"))
    );
    assert_eq!(
        run(&format!("{held}held.pid.inspect")),
        Some(Object::string("nil"))
    );
    assert_eq!(run(&format!("{held}held.tty?")), Some(Object::Bool(false)));
    assert_eq!(
        run(&format!("{held}held.isatty")),
        Some(Object::Bool(false))
    );
    assert_eq!(run(&format!("{held}held.sync")), Some(Object::Bool(true)));
    assert_eq!(run(&format!("{held}held.fsync")), Some(Object::Int(0)));
    assert_eq!(
        run(&format!("{held}held.flush.equal?(held)")),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_string_io_closes_its_two_sides_apart() {
    let held = "require 'stringio'\nheld = StringIO.new(\"example\")\n";
    assert_eq!(
        run(&format!("{held}held.close_read\nheld.closed?")),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!(
            "{held}held.close_read\nheld.close_write\nheld.closed?"
        )),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{held}held.close\nheld.closed?")),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!("{held}held.close_write\nheld.closed_write?")),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_string_io_truncates_and_pads_its_text() {
    let held = "require 'stringio'\nheld = StringIO.new(\"123456789\")\n";
    assert_eq!(
        run(&format!("{held}held.truncate(4)\nheld.string")),
        Some(Object::string("1234"))
    );
    assert_eq!(
        run(&format!("{held}held.truncate(4)")),
        Some(Object::Int(0))
    );
    assert_eq!(
        run(&format!("{held}held.string = \"other\"\nheld.pos")),
        Some(Object::Int(0))
    );
}

#[test]
fn a_string_io_refuses_a_negative_truncation() {
    let err = run_err("require 'stringio'\nStringIO.new(\"abc\").truncate(-1)");
    assert!(err.contains("Invalid argument"), "Error was: {}", err);
}

#[test]
fn a_string_io_has_no_file_control() {
    let err = run_err("require 'stringio'\nStringIO.new(\"abc\").fcntl");
    assert!(err.contains("unimplemented"), "Error was: {}", err);
}
