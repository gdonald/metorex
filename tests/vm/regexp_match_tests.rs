// A regexp match answers a MatchData, and records it as the last match that
// `$~`, `$1`, and `Regexp.last_match` read.

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

#[test]
fn a_match_answers_a_match_data_carrying_the_captures() {
    let result = run(r#"
found = /(\w+)\s+(\w+)/.match("hello there world")
[found.class.to_s, found[0], found[1], found.captures, found.pre_match, found.post_match].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(
            r#"["MatchData", "hello there", "hello", ["hello", "there"], "", " world"]"#
        ))
    );
}

#[test]
fn a_match_reports_where_each_group_sat() {
    let result = run(r#"
found = /(\w+)\s+(\w+)/.match("hello there")
[found.begin(0), found.end(0), found.offset(2), found.size].inspect
"#);
    assert_eq!(result, Some(Object::string("[0, 11, [6, 11], 3]")));
}

#[test]
fn a_named_group_stops_the_unnamed_ones_from_capturing() {
    let result = run(r#"
found = /(?<greeting>\w+)\s+(\w+)/.match("hello there")
[found[:greeting], found.names, found.captures, found.named_captures].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(
            r#"["hello", ["greeting"], ["hello"], {"greeting" => "hello"}]"#
        ))
    );
}

#[test]
fn begin_refuses_a_subscript_past_the_end() {
    let error = run_err(r#"/(a)/.match("a").begin(9)"#);
    assert!(error.contains("index 9 out of matches"), "{}", error);
}

#[test]
fn a_subscript_past_the_end_answers_nil() {
    let result = run(r#"/(a)/.match("a")[9].inspect"#);
    assert_eq!(result, Some(Object::string("nil")));
}

#[test]
fn the_last_match_is_what_the_capture_globals_read() {
    let result = run(r#"
"the quick fox" =~ /the (\w+)/
[$~[0], $1, $&, $~.pre_match, Regexp.last_match(1)].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(
            r#"["the quick", "quick", "the quick", "", "quick"]"#
        ))
    );
}

#[test]
fn a_pattern_that_does_not_match_clears_the_last_match() {
    let result = run(r#"
"hello" =~ /hello/
"nothing here" =~ /zzz/
[$~, $1].inspect
"#);
    assert_eq!(result, Some(Object::string("[nil, nil]")));
}

#[test]
fn grep_without_a_block_leaves_the_last_match_alone() {
    let result = run(r#"
"z" =~ /z/
["abc", "def"].grep(/b/)
[$&, Regexp.last_match[0]].inspect
"#);
    assert_eq!(result, Some(Object::string(r#"["z", "z"]"#)));
}

#[test]
fn a_regexp_answers_its_source_options_and_class() {
    let result = run(r#"
[/ab+c/i.source, /ab+c/i.options, /ab+c/i.casefold?, /a/.class.to_s, /a/ == /a/].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(r#"["ab+c", 1, true, "Regexp", true]"#))
    );
}

#[test]
fn regexp_builds_from_a_string_and_unions_its_arguments() {
    let result = run(r#"
[Regexp.new("ab").match("xaby")[0], Regexp.union("cat", "dog").source].inspect
"#);
    assert_eq!(result, Some(Object::string(r#"["ab", "cat|dog"]"#)));
}

#[test]
fn case_equality_covers_a_range_and_coerces_through_to_str() {
    let result = run(r#"
subject = Object.new
def subject.to_str
  "hello"
end
[(3..7) === 5, (3..7) === 9, /ll/ === subject].inspect
"#);
    assert_eq!(result, Some(Object::string("[true, false, true]")));
}

#[test]
fn a_string_subclass_matches_the_same_as_a_string() {
    let result = run(r#"
class Wrapped < String
end
/e(l+)/.match(Wrapped.new("hello"))[1]
"#);
    assert_eq!(result, Some(Object::string("ll")));
}

#[test]
fn a_group_name_may_be_written_more_than_once() {
    let result = run(r#"
found = /(?<a>.)(.)(?<a>\d+)(\d)/.match("THX1138.")
[found["a"], found.begin("a"), found.end("a"), found.names, found.named_captures].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(r#"["113", 3, 6, ["a"], {"a" => "113"}]"#))
    );
}

#[test]
fn a_multi_byte_group_name_counts_in_characters() {
    let result = run(r#"
found = /(?<æ>.)(.)(?<b>\d+)(\d)/.match("THX1138.")
[found["æ"], found.begin("æ"), found.names].inspect
"#);
    assert_eq!(result, Some(Object::string(r#"["H", 1, ["æ", "b"]]"#)));
}

#[test]
fn a_pattern_reports_the_group_numbers_each_name_was_written_on() {
    let result =
        run(r#"[/(?<a>.)(?<b>.)(?<a>.)/.names, /(?<a>.)(?<b>.)(?<a>.)/.named_captures].inspect"#);
    assert_eq!(
        result,
        Some(Object::string(
            r#"[["a", "b"], {"a" => [1, 3], "b" => [2]}]"#
        ))
    );
}

#[test]
fn two_patterns_differing_only_in_the_encoding_option_are_the_same() {
    let result = run(r#"[/x/n == /x/, /x/n.hash == /x/.hash, /x/i == /x/].inspect"#);
    assert_eq!(result, Some(Object::string("[true, true, false]")));
}

#[test]
fn sub_and_gsub_record_the_match_for_either_kind_of_pattern() {
    let result = run(r#"
"he[[o".gsub("[", "]")
literal = [$~.regexp.source, $~[0]]
"hello world".sub(/o (w)/, "O \\1")
(literal + [$~[0], $1]).inspect
"#);
    assert_eq!(result, Some(Object::string(r#"["\\[", "[", "o w", "w"]"#)));
}
