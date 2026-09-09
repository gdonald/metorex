// Coverage tests for GetoptLong, Find, and Timeout

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// Loading and running these libraries nests deeper than the stack a test
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

const OPTIONS: &str = "require 'getoptlong'\nARGV.replace([\"--size\", \"10k\", \"-v\", \"a.txt\"])\nopts = GetoptLong.new([\"--size\", \"-s\", GetoptLong::REQUIRED_ARGUMENT], [\"--verbose\", \"-v\", GetoptLong::NO_ARGUMENT])\nopts.quiet = true\n";

// ── Reading options ────────────────────────────────────────────────────────

#[test]
fn each_option_is_answered_with_the_word_it_carries() {
    assert_eq!(
        shown(&format!("{OPTIONS}opts.get")),
        "[\"--size\", \"10k\"]"
    );
    assert_eq!(
        shown(&format!("{OPTIONS}opts.get\nopts.get")),
        "[\"--verbose\", \"\"]"
    );
    assert_eq!(
        shown(&format!("{OPTIONS}opts.get\nopts.get\nopts.get")),
        "nil"
    );
    assert_eq!(
        shown(&format!(
            "{OPTIONS}found = []\nopts.each {{ |name, held| found.push([name, held]) }}\nfound"
        )),
        "[[\"--size\", \"10k\"], [\"--verbose\", \"\"]]"
    );
}

#[test]
fn one_letter_names_answer_the_name_written_first() {
    assert_eq!(
        shown(
            "require 'getoptlong'\nARGV.replace([\"-s\", \"2\"])\nopts = GetoptLong.new([\"--size\", \"-s\", GetoptLong::REQUIRED_ARGUMENT])\nopts.quiet = true\nopts.get"
        ),
        "[\"--size\", \"2\"]"
    );
    assert_eq!(
        shown(
            "require 'getoptlong'\nARGV.replace([\"-s\", \"2\"])\nopts = GetoptLong.new([\"-s\", \"--size\", GetoptLong::REQUIRED_ARGUMENT])\nopts.quiet = true\nopts.get"
        ),
        "[\"-s\", \"2\"]"
    );
    assert_eq!(
        shown(
            "require 'getoptlong'\nARGV.replace([\"-vc\"])\nopts = GetoptLong.new([\"--verbose\", \"-v\", GetoptLong::NO_ARGUMENT], [\"--check\", \"-c\", GetoptLong::NO_ARGUMENT])\nopts.quiet = true\n[opts.get, opts.get]"
        ),
        "[[\"--verbose\", \"\"], [\"--check\", \"\"]]"
    );
}

#[test]
fn a_word_may_be_written_after_an_equals_sign_or_as_a_shortened_name() {
    assert_eq!(
        shown(
            "require 'getoptlong'\nARGV.replace([\"--size=4k\"])\nopts = GetoptLong.new([\"--size\", GetoptLong::REQUIRED_ARGUMENT])\nopts.quiet = true\nopts.get"
        ),
        "[\"--size\", \"4k\"]"
    );
    assert_eq!(
        shown(
            "require 'getoptlong'\nARGV.replace([\"--siz=4k\"])\nopts = GetoptLong.new([\"--size\", GetoptLong::REQUIRED_ARGUMENT])\nopts.quiet = true\nopts.get"
        ),
        "[\"--size\", \"4k\"]"
    );
}

#[test]
fn the_words_that_are_not_options_are_left_in_argv() {
    assert_eq!(
        shown(&format!("{OPTIONS}opts.get\nopts.get\nopts.get\nARGV")),
        "[\"a.txt\"]"
    );
    assert_eq!(
        shown(
            "require 'getoptlong'\nARGV.replace([\"-v\", \"--\", \"-c\"])\nopts = GetoptLong.new([\"--verbose\", \"-v\", GetoptLong::NO_ARGUMENT], [\"--check\", \"-c\", GetoptLong::NO_ARGUMENT])\nopts.quiet = true\nopts.get\nopts.get\nARGV"
        ),
        "[\"-c\"]"
    );
}

#[test]
fn terminating_stops_the_reading_and_answers_self_only_once() {
    assert_eq!(shown(&format!("{OPTIONS}opts.terminated?")), "false");
    assert_eq!(
        shown(&format!(
            "{OPTIONS}opts.get\nopts.get\nopts.get\nopts.terminated?"
        )),
        "true"
    );
    assert_eq!(shown(&format!("{OPTIONS}opts.terminate\nopts.get")), "nil");
    assert_eq!(
        shown(&format!("{OPTIONS}opts.terminate\nopts.terminate")),
        "nil"
    );
}

// ── Refusing what cannot be read ───────────────────────────────────────────

#[test]
fn an_option_list_that_names_nothing_readable_is_refused() {
    const EMPTY: &str = "require 'getoptlong'\nARGV.replace([])\nopts = GetoptLong.new\n";
    assert!(
        run_err(&format!("{EMPTY}opts.set_options([\"--size\"])")).contains("no argument-flag")
    );
    assert!(
        run_err(&format!("{EMPTY}opts.set_options([\"--size\", GetoptLong::NO_ARGUMENT, GetoptLong::REQUIRED_ARGUMENT])"))
            .contains("too many argument-flags")
    );
    assert!(run_err(&format!("{EMPTY}opts.set_options(\"test\")")).contains("non-Array"));
    assert!(
        run_err(&format!(
            "{EMPTY}opts.set_options([\"-size\", GetoptLong::NO_ARGUMENT])"
        ))
        .contains("invalid option")
    );
    assert!(
        run_err(&format!("{EMPTY}opts.set_options([\"--size\", GetoptLong::NO_ARGUMENT], [\"--size\", GetoptLong::NO_ARGUMENT])"))
            .contains("option redefined")
    );
    assert!(
        run_err(&format!("{EMPTY}opts.get\nopts.set_options()"))
            .contains("option processing has already started")
    );
}

#[test]
fn an_option_missing_the_word_it_needs_is_refused() {
    const MISSING: &str = "require 'getoptlong'\nARGV.replace([\"--size\"])\nopts = GetoptLong.new([\"--size\", GetoptLong::REQUIRED_ARGUMENT])\nopts.quiet = true\n";
    assert!(run_err(&format!("{MISSING}opts.get")).contains("requires an argument"));
    assert_eq!(
        shown(&format!(
            "{MISSING}begin\n  opts.get\nrescue GetoptLong::MissingArgument\nend\nopts.error_message"
        )),
        "\"option `--size' requires an argument\""
    );
}

#[test]
fn the_ordering_may_not_change_once_reading_has_begun() {
    assert!(
        run_err("require 'getoptlong'\nopts = GetoptLong.new\nopts.ordering = 12345")
            .contains("invalid ordering")
    );
    assert!(
        run_err(&format!(
            "{OPTIONS}opts.get\nopts.ordering = GetoptLong::PERMUTE"
        ))
        .contains("argument error")
    );
    assert_eq!(
        shown("require 'getoptlong'\nGetoptLong.new.ordering == GetoptLong::PERMUTE"),
        "true"
    );
}

// ── Walking a directory ────────────────────────────────────────────────────

#[test]
fn find_hands_over_every_path_beneath_the_one_it_is_given() {
    assert_eq!(
        shown(
            "require 'find'\nfound = []\nFind.find('src/lexer') { |path| found.push(path) }\nfound.include?('src/lexer/percent.rs')"
        ),
        "true"
    );
    assert_eq!(
        shown("require 'find'\nFind.find('src/lexer').to_a.include?('src/lexer')"),
        "true"
    );
    assert_eq!(
        shown(
            "require 'find'\nfound = []\nFind.find('src') { |path| found.push(path)\n  Find.prune if path == 'src/lexer' }\nfound.any? { |path| path.start_with?('src/lexer/') }"
        ),
        "false"
    );
}

// ── A limit on how long a block runs ───────────────────────────────────────

#[test]
fn a_timed_block_answers_what_it_answers() {
    assert_eq!(shown("require 'timeout'\nTimeout.timeout(1) { 42 }"), "42");
    assert_eq!(shown("require 'timeout'\nTimeout.timeout(nil) { 7 }"), "7");
    assert_eq!(
        shown(
            "require 'timeout'\nRuntimeError.ancestors.include?(StandardError) && Timeout::Error.ancestors.include?(RuntimeError)"
        ),
        "true"
    );
    assert!(
        run_err("require 'timeout'\nTimeout.timeout(-1) { 1 }")
            .contains("Timeout sec must be a non-negative number")
    );
}
