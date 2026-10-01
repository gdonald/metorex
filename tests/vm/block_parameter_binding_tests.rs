// How a block binds what it is handed, and the parameter lists the parser
// refuses.

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

fn parse_error(code: &str) -> String {
    let errors = Parser::new(Lexer::new(code).tokenize())
        .parse()
        .expect_err("the code parsed");
    errors
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

const HAND: &str = "def hand(value)\n  yield value\nend\n";

#[test]
fn post_parameters_take_the_values_after_the_required_ones_before_them() {
    let code = format!("{HAND}hand([1, 2]) {{ |a, *b, c, d| [a, b, c, d] }} == [1, [], 2, nil]");
    assert_eq!(run(&code), Some(Object::Bool(true)));
}

#[test]
fn optional_parameters_take_what_the_required_ones_leave() {
    let code = format!(
        "{HAND}hand([1, 2, 3, 4]) {{ |a, b = 5, c = 6, d, e| [a, b, c, d, e] }} == [1, 2, 6, 3, 4]"
    );
    assert_eq!(run(&code), Some(Object::Bool(true)));
}

#[test]
fn extra_values_are_dropped_when_there_is_no_splat() {
    let code = format!(
        "{HAND}hand([1, 2, 3, 4, 5, 6]) {{ |a, b = 5, c = 6, d, e| [a, b, c, d, e] }} == [1, 2, 3, 4, 5]"
    );
    assert_eq!(run(&code), Some(Object::Bool(true)));
}

#[test]
fn a_lone_object_answering_to_ary_spreads_across_the_parameters() {
    let code = format!(
        "{HAND}class Pair\n  def to_ary\n    [1, 2]\n  end\nend\nhand(Pair.new) {{ |a, b, c| [a, b, c] }} == [1, 2, nil]"
    );
    assert_eq!(run(&code), Some(Object::Bool(true)));
}

#[test]
fn a_lone_object_whose_to_ary_answers_nil_stays_whole() {
    let code = format!(
        "{HAND}class Declines\n  def to_ary\n    nil\n  end\nend\nheld = Declines.new\nhand(held) {{ |a, b| a.equal?(held) && b.nil? }}"
    );
    assert_eq!(run(&code), Some(Object::Bool(true)));
}

#[test]
fn a_lone_object_asks_its_own_respond_to_before_spreading() {
    let code = format!(
        "{HAND}class Refuses\n  def respond_to?(name, include_all = false)\n    false\n  end\n  def to_ary\n    [1, 2]\n  end\nend\nhand(Refuses.new) {{ |a, b| b }}"
    );
    assert_eq!(run(&code), Some(Object::Nil));
}

#[test]
fn a_lone_object_answering_to_ary_through_method_missing_spreads() {
    let code = format!(
        "{HAND}class Dynamic\n  def method_missing(name, *arguments)\n    name == :to_ary ? [3, 4] : super\n  end\n  def respond_to_missing?(name, include_private)\n    name == :to_ary\n  end\nend\nhand(Dynamic.new) {{ |a, b| a * b }}"
    );
    assert_eq!(run(&code), Some(Object::Int(12)));
}

#[test]
fn a_lone_object_without_respond_to_missing_stays_whole() {
    let code = format!("{HAND}hand(4) {{ |a, b| [a, b] }} == [4, nil]");
    assert_eq!(run(&code), Some(Object::Bool(true)));
}

#[test]
fn a_to_ary_answering_something_other_than_an_array_raises_type_error() {
    let code = format!(
        "{HAND}class Wrong\n  def to_ary\n    7\n  end\nend\nbegin\n  hand(Wrong.new) {{ |a, b| a }}\nrescue TypeError => error\n  error.message\nend"
    );
    assert_eq!(
        run(&code),
        Some(Object::string(
            "can't convert Wrong to Array (Wrong#to_ary gives Integer)"
        ))
    );
}

#[test]
fn a_repeated_underscore_parameter_takes_the_first_value() {
    let code = format!("{HAND}hand([1, 2]) {{ |_, _| _ }}");
    assert_eq!(run(&code), Some(Object::Int(1)));
}

#[test]
fn a_default_reading_its_own_parameter_reads_nil() {
    assert_eq!(run("proc { |same = same| same }.call"), Some(Object::Nil));
}

#[test]
fn a_block_local_written_after_a_leading_semicolon_shadows_the_outer_name() {
    let code = "outer = :outer\n[1].each { |; outer| outer = :inner }\nouter";
    assert_eq!(run(code), Some(Object::symbol("outer".to_string())));
}

#[test]
fn a_block_local_starts_as_nil_in_a_block_run_by_each() {
    assert_eq!(
        run("seen = :unset\n[1].each { |value; kept| seen = kept }\nseen"),
        Some(Object::Nil)
    );
}

#[test]
fn a_repeated_parameter_name_is_refused() {
    assert!(parse_error("[1].each { |x, x| }").contains("duplicated argument name"));
}

#[test]
fn a_repeated_lambda_parameter_name_is_refused() {
    assert!(parse_error("-> (x, x) {}").contains("duplicated argument name"));
}

#[test]
fn a_repeated_parameter_name_without_parentheses_is_refused() {
    assert!(parse_error("-> x, x { x }").contains("duplicated argument name"));
}

#[test]
fn a_second_semicolon_in_a_parameter_list_is_refused() {
    assert!(parse_error("[1].each { |a; b; c| }").contains("unexpected ';'"));
}

#[test]
fn a_bare_ampersand_without_an_anonymous_block_parameter_is_refused() {
    assert!(parse_error("def lone\n  hand(1, &)\nend").contains("no anonymous block parameter"));
}

#[test]
fn an_endless_method_forwards_its_anonymous_block() {
    let code = format!("{HAND}def forwards(&) = hand(3, &)\nforwards {{ |value| value + 1 }}");
    assert_eq!(run(&code), Some(Object::Int(4)));
}

#[test]
fn an_interpolated_percent_word_forwards_the_anonymous_block() {
    let code =
        format!("{HAND}def words(&)\n  %W[#{{hand(2,&)}}]\nend\nwords {{ |value| value * 5 }}");
    assert_eq!(run(&code), Some(Object::array(vec![Object::string("10")])));
}

#[test]
fn an_interpolated_symbol_forwards_the_anonymous_block() {
    let code =
        format!("{HAND}def named(&)\n  :\"held_#{{hand(2, &)}}\"\nend\nnamed {{ |value| value }}");
    assert_eq!(run(&code), Some(Object::symbol("held_2".to_string())));
}
