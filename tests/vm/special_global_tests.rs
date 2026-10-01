// What assigning and reading a special global checks and stores, and the
// names the parser refuses to assign.

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

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

/// The message of the error `code` raises, rescued in the program.
fn refusal(code: &str) -> Option<Object> {
    run(&format!(
        "begin\n  {code}\n  :assigned\nrescue TypeError, NameError, ArgumentError => error\n  error.message\nend"
    ))
}

/// Run `code` with `$stderr` gathered and the deprecated category on, and
/// answer what was written there.
fn deprecation_warnings_from(code: &str) -> String {
    let wrapped = format!(
        "gathered = []\ncollector = Object.new\ncollector.define_singleton_method(:write) {{ |text| gathered << text }}\n$stderr = collector\nWarning[:deprecated] = true\n$VERBOSE = false\n{code}\n$stderr = STDERR\ngathered.join"
    );
    match run(&wrapped) {
        Some(Object::String(written)) => written.as_str().to_string(),
        other => panic!("expected the warnings, got {other:?}"),
    }
}

#[test]
fn the_last_match_takes_nil_or_a_match_data() {
    assert_eq!(
        run("'a' =~ /a/\nheld = $~\n$~ = nil\ncleared = $~\n$~ = held\n[cleared, $~[0]]"),
        Some(Object::array(vec![Object::Nil, Object::string("a")]))
    );
}

#[test]
fn the_last_match_refuses_anything_else() {
    assert_eq!(
        refusal("$~ = 1"),
        text("wrong argument type Integer (expected MatchData)")
    );
}

#[test]
fn standard_output_takes_an_object_that_writes() {
    let code = "writer = Object.new\ndef writer.write(*parts); end\n$stdout = writer\nswapped = $stdout.equal?(writer)\n$stdout = STDOUT\nswapped";
    assert_eq!(run(code), Some(Object::Bool(true)));
}

#[test]
fn standard_error_refuses_an_object_that_does_not_write() {
    assert_eq!(
        refusal("$stderr = Object.new"),
        text("$stderr must have write method, Object given")
    );
}

#[test]
fn standard_output_names_the_class_of_an_instance_it_refuses() {
    assert_eq!(
        refusal("class Silent; end\n$stdout = Silent.new"),
        text("$stdout must have write method, Silent given")
    );
}

#[test]
fn a_read_only_global_refuses_assignment() {
    assert_eq!(refusal("$? = 1"), text("$? is a read-only variable"));
}

#[test]
fn an_alias_of_a_match_global_is_read_only() {
    assert_eq!(
        refusal("alias $whole_match $&\n$whole_match = 'x'"),
        text("$whole_match is a read-only variable")
    );
}

#[test]
fn an_alias_of_a_global_written_in_a_class_body_names_the_global() {
    let code = "class Holder\n  alias $held_line $_\nend\n$_ = 'line'\n$held_line";
    assert_eq!(run(code), text("line"));
}

#[test]
fn an_alias_of_a_global_written_in_a_module_body_names_the_global() {
    let code = "module Holder\n  alias $held_line $_\nend\n$_ = 'line'\n$held_line";
    assert_eq!(run(code), text("line"));
}

#[test]
fn the_record_separator_refuses_a_non_string() {
    assert_eq!(refusal("$/ = 1"), text("value of $/ must be String"));
}

#[test]
fn the_output_separator_refuses_a_string_subclass() {
    assert_eq!(
        refusal("$\\ = Class.new(String).new('x')"),
        text("value of $\\ must be String")
    );
}

#[test]
fn the_record_separator_copies_a_string_that_is_not_frozen() {
    let code =
        "held = +'x'\n$/ = held\nanswer = [$/.equal?(held), $/.frozen?, $/]\n$/ = \"\\n\"\nanswer";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::Bool(false),
            Object::Bool(true),
            Object::string("x")
        ]))
    );
}

#[test]
fn the_record_separator_takes_nil() {
    assert_eq!(
        run("$/ = nil\nanswer = $/\n$/ = \"\\n\"\nanswer"),
        Some(Object::Nil)
    );
}

#[test]
fn the_field_separator_keeps_the_string_it_is_given() {
    let code = "held = +','\n$; = held\nanswer = $;.equal?(held)\n$; = nil\nanswer";
    assert_eq!(run(code), Some(Object::Bool(true)));
}

#[test]
fn a_non_nil_separator_warns_under_the_deprecated_category() {
    let written = deprecation_warnings_from("$, = ','\n$, = nil");
    assert!(written.contains("warning: non-nil '$,' is deprecated"));
}

#[test]
fn the_line_number_keeps_an_integer() {
    assert_eq!(run("$. = 3\n$."), Some(Object::Int(3)));
}

#[test]
fn the_line_number_truncates_a_float() {
    assert_eq!(run("$. = 12.5\n$."), Some(Object::Int(12)));
}

#[test]
fn the_line_number_converts_with_to_int() {
    assert_eq!(
        run("count = Object.new\ndef count.to_int; 7; end\n$. = count\n$."),
        Some(Object::Int(7))
    );
}

#[test]
fn the_line_number_refuses_nil() {
    assert_eq!(
        refusal("$. = nil"),
        text("no implicit conversion from nil to integer")
    );
}

#[test]
fn the_line_number_refuses_an_object_without_to_int() {
    assert_eq!(
        refusal("$. = 'three'"),
        text("no implicit conversion of String into Integer")
    );
}

#[test]
fn the_line_number_refuses_a_to_int_that_answers_something_else() {
    assert_eq!(
        refusal("count = Object.new\ndef count.to_int; 'seven'; end\n$. = count"),
        text("no implicit conversion of Object into Integer")
    );
}

#[test]
fn the_program_name_takes_a_string() {
    assert_eq!(run("$0 = 'renamed'\n$0"), text("renamed"));
}

#[test]
fn the_program_name_converts_with_to_str() {
    assert_eq!(
        run("named = Object.new\ndef named.to_str; 'converted'; end\n$0 = named\n$0"),
        text("converted")
    );
}

#[test]
fn the_program_name_refuses_an_object_without_to_str() {
    assert_eq!(
        refusal("$0 = 1"),
        text("no implicit conversion of Integer into String")
    );
}

#[test]
fn verbose_stores_true_for_a_truthy_value_and_keeps_nil_and_false() {
    assert_eq!(
        run(
            "$VERBOSE = 1\nfirst = $VERBOSE\n$VERBOSE = false\nsecond = $VERBOSE\n$VERBOSE = nil\n[first, second, $VERBOSE]"
        ),
        Some(Object::array(vec![
            Object::Bool(true),
            Object::Bool(false),
            Object::Nil
        ]))
    );
}

#[test]
fn the_flag_aliases_name_the_globals_they_stand_for() {
    assert_eq!(
        run("[$-0.equal?($/), $-I.equal?($:), $-v == $VERBOSE, $-w == $VERBOSE, $-d == $DEBUG]"),
        Some(Object::array(vec![Object::Bool(true); 5]))
    );
}

#[test]
fn the_backtrace_global_sets_the_backtrace_of_the_exception_being_handled() {
    let code =
        "begin\n  raise 'broken'\nrescue => error\n  $@ = ['here:1']\n  error.backtrace\nend";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::string("here:1")]))
    );
}

#[test]
fn the_backtrace_global_refuses_assignment_with_no_exception() {
    assert_eq!(refusal("$@ = []"), text("$! not set"));
}

#[test]
fn the_ignore_case_global_warns_when_read_and_when_assigned() {
    let written = deprecation_warnings_from("read = $=\n$= = true");
    assert!(written.contains("warning: variable $= is no longer effective\n"));
    assert!(written.contains("warning: variable $= is no longer effective; ignored\n"));
}

#[test]
fn the_ignore_case_global_is_silent_without_the_deprecated_category() {
    assert_eq!(
        run("Warning[:deprecated] = false\n$VERBOSE = true\n$= = true\n$="),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_rescue_clause_that_raises_leaves_its_exception_in_the_error_global() {
    let code = "seen = nil\nbegin\n  begin\n    raise 'outer'\n  rescue\n    raise 'inner'\n  ensure\n    seen = $!.message\n  end\nrescue\nend\nseen";
    assert_eq!(run(code), text("inner"));
}

#[test]
fn the_last_line_read_belongs_to_the_thread_that_set_it() {
    assert_eq!(
        run(
            "$_ = 'main'\ninside = nil\nThread.new { inside = $_; $_ = 'thread' }.join\n[inside, $_]"
        ),
        Some(Object::array(vec![Object::Nil, Object::string("main")]))
    );
}

#[test]
fn a_string_answers_whether_an_instance_variable_is_set_on_it() {
    assert_eq!(
        run(
            "held = +'x'\nheld.instance_variable_set(:@mark, 1)\n[held.instance_variable_defined?(:@mark), held.instance_variable_defined?(:@other), 1.instance_variable_defined?(:@mark)]"
        ),
        Some(Object::array(vec![
            Object::Bool(true),
            Object::Bool(false),
            Object::Bool(false)
        ]))
    );
}

#[test]
fn a_match_keeps_the_encoding_of_the_string_it_searched() {
    let code = "subject = 'abc'.dup.force_encoding(Encoding::EUC_JP)\nsubject =~ /(b)/\nfirst = [$&.encoding, $`.encoding, $1.encoding]\n/c/ =~ subject\nsecond = $&.encoding\n/a/.match(subject)\nthird = $&.encoding\ncase subject\nwhen /b/ then fourth = $&.encoding\nend\n(first + [second, third, fourth]).map(&:name)";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::string("EUC-JP"); 6]))
    );
}

fn syntax_error(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    match Parser::new(tokens).parse() {
        Ok(_) => panic!("expected {code:?} to be refused"),
        Err(errors) => errors
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[test]
fn assigning_nil_true_or_false_is_a_syntax_error() {
    assert!(syntax_error("nil = 1").contains("Can't assign to nil"));
    assert!(syntax_error("true = 1").contains("Can't assign to true"));
    assert!(syntax_error("false = 1").contains("Can't assign to false"));
}

#[test]
fn assigning_self_is_a_syntax_error() {
    assert!(syntax_error("self = 1").contains("Can't change the value of self"));
}

#[test]
fn assigning_a_match_global_is_a_syntax_error() {
    for name in ["&", "`", "'", "+", "1", "12"] {
        let refused = syntax_error(&format!("${name} = 1"));
        assert!(
            refused.contains(&format!("Can't set variable ${name}")),
            "{refused}"
        );
    }
}

#[test]
fn a_match_global_in_a_multiple_assignment_is_a_syntax_error() {
    assert!(syntax_error("$&, other = 1, 2").contains("Can't set variable $&"));
}

#[test]
fn the_program_name_global_is_assignable() {
    let tokens = Lexer::new("$0 = 'name'").tokenize();
    assert!(Parser::new(tokens).parse().is_ok());
}

#[test]
fn the_field_separator_takes_a_pattern() {
    assert_eq!(
        run("$; = /,/\nparts = 'a,b'.split\n$; = nil\nparts"),
        Some(Object::array(vec![
            Object::string("a"),
            Object::string("b")
        ]))
    );
}

#[test]
fn the_field_separator_refuses_anything_but_a_string_or_a_pattern() {
    assert_eq!(
        refusal("$; = 1"),
        text("value of $; must be String or Regexp")
    );
}

#[test]
fn every_kind_of_value_answers_an_integer_hash_that_stays_the_same() {
    let code = "values = [14, 10**30, 3.14, Rational(1, 2), Complex(1, 2), 'abc', :a, [1, 2], {a: 1}, [], {}]\nvalues.map { |value| value.hash.is_a?(Integer) && value.hash == value.dup.hash }.uniq";
    assert_eq!(run(code), Some(Object::array(vec![Object::Bool(true)])));
}

#[test]
fn equal_values_hash_alike_and_different_ones_apart() {
    assert_eq!(
        run(
            "[Rational(1, 2).hash == Rational(2, 4).hash, Complex(1, 2).hash == Complex(1, 3).hash, (10**30).hash == (10**30 + 1).hash]"
        ),
        Some(Object::array(vec![
            Object::Bool(true),
            Object::Bool(false),
            Object::Bool(false)
        ]))
    );
}

#[test]
fn an_interpolated_exception_writes_what_its_to_s_answers() {
    assert_eq!(
        run(
            "class Spoken < StandardError\n  def to_s\n    'custom'\n  end\nend\nbegin\n  raise 'plain'\nrescue => error\n  [\"#{error}\", \"#{Spoken.new}\", \"#{ArgumentError.new}\"]\nend"
        ),
        Some(Object::array(vec![
            Object::string("plain"),
            Object::string("custom"),
            Object::string("ArgumentError")
        ]))
    );
}
