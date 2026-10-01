// How a multiple assignment takes its right-hand side apart, where a local
// begins, and the notice for reading a global nothing set.

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

fn inspected(code: &str) -> String {
    match run(&format!("({code}).inspect")) {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

/// The message of the TypeError `code` raises, rescued in the program.
fn type_error(code: &str) -> String {
    match run(&format!(
        "begin\n  {code}\n  nil\nrescue TypeError => error\n  error.message\nend"
    )) {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected a TypeError, got {other:?}"),
    }
}

#[test]
fn a_lone_value_is_taken_apart_through_to_ary() {
    assert_eq!(
        inspected("held = Object.new\ndef held.to_ary; [1, 2]; end\na, b, c = held\n[a, b, c]"),
        "[1, 2, nil]"
    );
}

#[test]
fn a_private_to_ary_is_used() {
    assert_eq!(
        inspected(
            "held = Object.new\nclass << held\n  private def to_ary; [3, 4]; end\nend\na, b = held\n[a, b]"
        ),
        "[3, 4]"
    );
}

#[test]
fn a_value_whose_respond_to_says_no_is_not_converted() {
    assert_eq!(
        inspected(
            "held = Object.new\ndef held.respond_to?(name, all = false); false; end\ndef held.to_ary; [:never]; end\na, b = held\n[a.equal?(held), b]"
        ),
        "[true, nil]"
    );
}

#[test]
fn a_to_ary_answering_nil_leaves_the_value_whole() {
    assert_eq!(
        inspected("held = Object.new\ndef held.to_ary; nil; end\na, b = held\n[a.equal?(held), b]"),
        "[true, nil]"
    );
}

#[test]
fn a_to_ary_answering_something_else_is_refused() {
    assert_eq!(
        type_error("held = Object.new\ndef held.to_ary; 1; end\na, b = held"),
        "can't convert Object to Array (Object#to_ary gives Integer)"
    );
}

#[test]
fn an_array_subclass_is_taken_apart_without_being_asked() {
    assert_eq!(
        inspected(
            "listed = Class.new(Array) { def to_ary; raise 'asked'; end }[5, 6]\na, b = listed\n[a, b]"
        ),
        "[5, 6]"
    );
}

#[test]
fn a_splat_among_several_values_spreads_through_to_a() {
    assert_eq!(
        inspected("held = Object.new\ndef held.to_a; [7, 8]; end\na, b, c = 1, *held\n[a, b, c]"),
        "[1, 7, 8]"
    );
}

#[test]
fn a_splat_of_nil_spreads_nothing() {
    assert_eq!(inspected("a, b = 1, *nil\n[a, b]"), "[1, nil]");
}

#[test]
fn a_splat_of_an_array_subclass_spreads_its_elements() {
    assert_eq!(
        inspected("listed = Class.new(Array)[1, 2]\n*all = *listed\nall"),
        "[1, 2]"
    );
}

#[test]
fn a_lone_splat_builds_a_new_unfrozen_array() {
    assert_eq!(
        inspected(
            "source = [1, 2].freeze\ncopied = *source\n[copied.equal?(source), copied.frozen?]"
        ),
        "[false, false]"
    );
}

#[test]
fn a_splat_whose_to_a_answers_something_else_is_refused() {
    assert_eq!(
        type_error("held = Object.new\ndef held.to_a; 1; end\nsingle = *held"),
        "can't convert Object to Array (Object#to_a gives Integer)"
    );
}

#[test]
fn a_nested_group_takes_its_value_apart_through_to_ary() {
    assert_eq!(
        inspected(
            "held = Object.new\ndef held.to_ary; [2, 3]; end\na, (b, c), d = 1, held, 4\n[a, b, c, d]"
        ),
        "[1, 2, 3, 4]"
    );
}

#[test]
fn a_local_assigned_in_a_branch_that_did_not_run_is_a_local_for_a_later_block() {
    assert_eq!(
        inspected(
            "def check\n  if false\n    later = 1\n  end\n  [1].map { [defined?(later), later] }.first\nend\ncheck"
        ),
        "[\"local-variable\", nil]"
    );
}

#[test]
fn a_local_bound_inside_a_block_is_not_one_after_it() {
    assert_eq!(
        inspected("[1].each { value = 1 }\ndef value(given)\n  given\nend\nvalue []"),
        "[]"
    );
}

#[test]
fn a_local_bound_earlier_in_the_same_scope_is_indexed() {
    assert_eq!(inspected("value = [4]\nvalue [0]"), "4");
}

#[test]
fn a_name_bound_only_later_in_the_file_is_a_call_before_that() {
    assert_eq!(
        inspected(
            "def listed(given)\n  given\nend\nfirst = listed [1]\nlisted = :now_a_local\nfirst"
        ),
        "[1]"
    );
}

/// Run `code` with `$stderr` gathered and `$VERBOSE` on, and answer what
/// was written there.
fn verbose_warnings_from(code: &str) -> String {
    let wrapped = format!(
        "gathered = []\ncollector = Object.new\ncollector.define_singleton_method(:write) {{ |text| gathered << text }}\n$stderr = collector\n$VERBOSE = true\n{code}\n$VERBOSE = false\n$stderr = STDERR\ngathered.join"
    );
    match run(&wrapped) {
        Some(Object::String(written)) => written.as_str().to_string(),
        other => panic!("expected the warnings, got {other:?}"),
    }
}

#[test]
fn reading_a_global_nothing_set_warns_in_verbose_mode() {
    assert!(
        verbose_warnings_from("read = $never_assigned_here")
            .contains("warning: global variable '$never_assigned_here' not initialized")
    );
}

#[test]
fn assigning_a_global_with_or_equals_does_not_warn() {
    assert_eq!(verbose_warnings_from("$lazily_set_here ||= 1"), "");
}

#[test]
fn reading_a_global_nothing_set_is_quiet_outside_verbose_mode() {
    assert_eq!(run("$quiet_unset_global"), Some(Object::Nil));
}

#[test]
fn the_file_argf_reads_is_standard_input_until_one_is_named() {
    assert_eq!(run("$FILENAME"), Some(Object::string("-")));
}
