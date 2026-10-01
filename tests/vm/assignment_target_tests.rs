// Assignment targets written on a receiver: the receiver, subscripts, and
// namespace are evaluated once, ahead of the value.

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

fn parses(code: &str) -> bool {
    Parser::new(Lexer::new(code).tokenize()).parse().is_ok()
}

const BOX_CLASS: &str = r#"
class Box
  attr_accessor :count

  def initialize
    @slots = {}
  end

  def [](*keys)
    @slots[keys]
  end

  def []=(*keys, value)
    @slots[keys[0...-1] + [keys.last]] = value
  end
end
"#;

#[test]
fn splat_subscript_is_evaluated_once_and_spread_across_the_subscripts() {
    let code = format!(
        "{BOX_CLASS}
calls = 0
keys = Object.new
keys.define_singleton_method(:to_a) {{ calls += 1; [:a, :b] }}
box = Box.new
box[*keys] = 5
calls * 100 + box[:a, :b]"
    );
    assert_eq!(run(&code), Some(Object::Int(105)));
}

#[test]
fn several_subscripts_are_written_through_one_call() {
    let code = format!(
        "{BOX_CLASS}
box = Box.new
box[:a, :b] = 3
box[:a, :b] += 4
box[:a, :b]"
    );
    assert_eq!(run(&code), Some(Object::Int(7)));
}

#[test]
fn and_assign_on_a_falsy_reader_writes_nothing() {
    let code = format!(
        "{BOX_CLASS}
writes = 0
box = Box.new
box.define_singleton_method(:count=) {{ |value| writes += 1 }}
box.count &&= 9
writes"
    );
    assert_eq!(run(&code), Some(Object::Int(0)));
}

#[test]
fn or_assign_defines_a_constant_the_namespace_lacks() {
    let code = "module Limits\nend\nLimits::DEPTH ||= 4\nLimits::DEPTH";
    assert_eq!(run(code), Some(Object::Int(4)));
}

#[test]
fn or_assign_keeps_a_constant_the_namespace_holds() {
    let code = "module Limits\n  DEPTH = 2\nend\nLimits::DEPTH ||= 4\nLimits::DEPTH";
    assert_eq!(run(code), Some(Object::Int(2)));
}

#[test]
fn compound_assignment_on_a_scoped_constant_evaluates_the_namespace_once() {
    let code = "module Limits\n  WIDTH = 1\nend\n$VERBOSE = nil\nlooked = 0\n(looked += 1; Limits)::WIDTH += 2\nlooked * 10 + Limits::WIDTH";
    assert_eq!(run(code), Some(Object::Int(13)));
}

#[test]
fn or_assign_on_an_undefined_bare_constant_defines_it() {
    assert_eq!(run("LEVEL ||= 3\nLEVEL"), Some(Object::Int(3)));
}

#[test]
fn a_constant_bound_on_object_reads_at_the_top_level() {
    assert_eq!(run("Object::SHARED = 6\nSHARED"), Some(Object::Int(6)));
}

#[test]
fn a_scoped_constant_on_a_value_that_is_not_a_module_raises_type_error() {
    let code = "begin\n  (:plain)::A = 1\nrescue TypeError => error\n  error.message\nend";
    assert_eq!(
        run(code),
        Some(Object::string(":plain is not a class/module"))
    );
}

#[test]
fn a_private_reader_refuses_an_explicit_receiver_in_a_compound_assignment() {
    let code = r#"
class Guarded
  attr_writer :amount
  private
  attr_reader :amount
end
begin
  Guarded.new.amount += 1
rescue NoMethodError => error
  error.message
end"#;
    assert_eq!(
        run(code),
        Some(Object::string(
            "private method 'amount' called for an instance of Guarded"
        ))
    );
}

#[test]
fn a_target_list_with_an_unclosed_group_does_not_parse() {
    assert!(!parses("first, (second"));
}

#[test]
fn a_target_list_with_a_dot_naming_nothing_after_a_group_does_not_parse() {
    assert!(!parses("first, (second). = 1, 2"));
}

#[test]
fn a_target_list_with_a_call_and_subscript_parses() {
    let code = "store = {}\nsource = [store]\nsource.fetch(0)[:a], other = 1, 2\nstore[:a] + other";
    assert_eq!(run(code), Some(Object::Int(3)));
}

#[test]
fn and_assign_on_a_scoped_constant_that_is_not_there_raises_name_error() {
    let code = "begin\n  Object::MissingHere &&= 1\nrescue NameError => error\n  error.class\nend";
    assert_eq!(
        run(code).map(|class| class.to_string()),
        Some("NameError".to_string())
    );
}

#[test]
fn or_assign_on_a_top_level_constant_defines_it() {
    assert_eq!(
        run("::TopUndefinedHere ||= 5\nTopUndefinedHere"),
        Some(Object::Int(5))
    );
}
