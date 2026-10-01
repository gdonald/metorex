// What `super` reaches and the arguments a bare `super` hands on.

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

fn ints(values: &[i64]) -> Object {
    Object::array(values.iter().map(|value| Object::Int(*value)).collect())
}

/// A parent whose `take` answers everything it was handed.
const PARENT: &str =
    "class Parent\n  def take(*values, **options)\n    [values, options]\n  end\nend\n";

fn through_parent(child: &str, call: &str) -> Option<Object> {
    run(&format!(
        "{PARENT}class Child < Parent\n{child}\nend\n{call}"
    ))
}

#[test]
fn a_bare_super_hands_on_a_parameter_the_body_changed() {
    assert_eq!(
        through_parent(
            "  def take(first)\n    first = 9\n    super\n  end",
            "Child.new.take(1)"
        ),
        run("[[9], {}]")
    );
}

#[test]
fn a_bare_super_hands_on_a_default_the_call_left_out() {
    assert_eq!(
        through_parent(
            "  def take(first, second = 5)\n    super\n  end",
            "Child.new.take(1)"
        ),
        run("[[1, 5], {}]")
    );
}

#[test]
fn a_bare_super_spreads_the_rest_parameter_as_it_stands() {
    assert_eq!(
        through_parent(
            "  def take(first, *rest, last)\n    rest << 7\n    super\n  end",
            "Child.new.take(1, 2, 3)"
        ),
        run("[[1, 2, 7, 3], {}]")
    );
}

#[test]
fn a_bare_super_hands_on_a_rest_parameter_given_a_single_value() {
    assert_eq!(
        through_parent(
            "  def take(*rest)\n    rest = :one\n    super\n  end",
            "Child.new.take"
        ),
        run("[[:one], {}]")
    );
}

#[test]
fn a_bare_super_hands_on_keywords_and_the_keyword_rest() {
    assert_eq!(
        through_parent(
            "  def take(flag: :off, **more)\n    super\n  end",
            "Child.new.take(size: 2).inspect"
        ),
        Some(Object::string("[[], {size: 2, flag: :off}]"))
    );
}

#[test]
fn a_bare_super_hands_on_each_value_of_a_repeated_underscore() {
    assert_eq!(
        through_parent(
            "  def take(_, _, _)\n    _ = 0\n    super\n  end",
            "Child.new.take(1, 2, 3)"
        ),
        run("[[0, 2, 3], {}]")
    );
}

#[test]
fn an_underscore_written_twice_reads_the_first_value() {
    assert_eq!(
        run("def first_of(_, _)\n  _\nend\nfirst_of(1, 2)"),
        Some(Object::Int(1))
    );
}

#[test]
fn a_bare_super_from_a_method_that_takes_its_argument_apart_hands_on_the_argument() {
    assert_eq!(
        through_parent(
            "  def take((first, second))\n    super\n  end",
            "Child.new.take([1, 2])"
        ),
        Some(Object::array(vec![
            Object::array(vec![ints(&[1, 2])]),
            run("{}").expect("a hash")
        ]))
    );
}

#[test]
fn a_bare_super_in_a_block_reaches_the_method_the_block_was_written_in() {
    assert_eq!(
        through_parent(
            "  def take(first)\n    [1].map { super }.first\n  end",
            "Child.new.take(4)"
        ),
        run("[[4], {}]")
    );
}

#[test]
fn a_bare_super_from_a_method_built_from_a_block_is_refused() {
    assert_eq!(
        through_parent(
            "  define_method(:take) { |first| super }",
            "begin\n  Child.new.take(1)\nrescue RuntimeError => error\n  error.message.start_with?('implicit argument passing of super')\nend"
        ),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_method_an_ancestor_undefined_sends_super_to_method_missing() {
    let code = "class Hidden\n  undef_method :frozen?\nend\nclass Asks < Hidden\n  def frozen?\n    super\n  end\n  def method_missing(name, *arguments)\n    name\n  end\nend\nAsks.new.frozen?";
    assert_eq!(run(code), Some(Object::symbol("frozen?".to_string())));
}

#[test]
fn a_method_nothing_above_defines_raises_without_method_missing() {
    let code = "class Hidden\n  undef_method :frozen?\nend\nclass Asks < Hidden\n  def frozen?\n    super\n  end\nend\nbegin\n  Asks.new.frozen?\nrescue NoMethodError => error\n  error.message\nend";
    assert_eq!(
        run(code),
        Some(Object::string(
            "super: no superclass method 'frozen?' for an instance of Asks"
        ))
    );
}

#[test]
fn super_from_an_included_module_reaches_what_every_object_answers() {
    let code = "module Sends\n  def __send__(name, *arguments)\n    super\n  end\nend\nclass Sender\n  include Sends\n  def answer\n    42\n  end\nend\nSender.new.__send__(:answer)";
    assert_eq!(run(code), Some(Object::Int(42)));
}

#[test]
fn super_from_an_included_module_with_nothing_above_raises() {
    let code = "module Lone\n  def lonely\n    super\n  end\nend\nclass Holder\n  include Lone\nend\nbegin\n  Holder.new.lonely\nrescue NoMethodError => error\n  error.name\nend";
    assert_eq!(run(code), Some(Object::symbol("lonely".to_string())));
}

#[test]
fn super_from_an_alias_starts_past_the_class_that_wrote_the_body() {
    let code = "class Named\n  def label\n    [:named]\n  end\nend\nclass Relabeled < Named\n  def label\n    [:relabeled] + super\n  end\nend\nclass Copied < Relabeled\n  alias_method :tag, :label\nend\nCopied.new.tag";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("relabeled".to_string()),
            Object::symbol("named".to_string())
        ]))
    );
}

#[test]
fn super_from_an_extended_module_reaches_the_module_extended_before_it() {
    let code = "module Plain\n  def build\n    :plain\n  end\nend\nmodule Wrapped\n  def build\n    [:wrapped, super]\n  end\nend\nclass Factory\n  extend Plain\n  extend Wrapped\nend\nFactory.build";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("wrapped".to_string()),
            Object::symbol("plain".to_string())
        ]))
    );
}

#[test]
fn super_from_an_extended_module_reaches_a_superclass_extension() {
    let code = "module Plain\n  def build\n    :plain\n  end\nend\nmodule Wrapped\n  def build\n    [:wrapped, super]\n  end\nend\nclass Base\n  extend Plain\nend\nclass Factory < Base\n  extend Wrapped\nend\nFactory.build";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("wrapped".to_string()),
            Object::symbol("plain".to_string())
        ]))
    );
}

#[test]
fn a_class_method_super_reaches_the_superclass_class_method() {
    let code = "class Base\n  def self.build(value)\n    [:base, value]\n  end\nend\nclass Factory < Base\n  def self.build(value)\n    value = 3\n    super\n  end\nend\nFactory.build(1)";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("base".to_string()),
            Object::Int(3)
        ]))
    );
}

#[test]
fn a_bare_super_hands_on_a_marked_hash_as_keywords() {
    assert_eq!(
        through_parent(
            "  ruby2_keywords def take(*values)\n    super\n  end",
            "[Child.new.take(size: 2), Child.new.take({size: 2})].inspect"
        ),
        Some(Object::string("[[[], {size: 2}], [[{size: 2}], {}]]"))
    );
}

#[test]
fn a_singleton_method_reaches_the_copy_its_module_extended_itself_with() {
    let code = "module Greeter\n  def greet\n    :module_copy\n  end\n  extend self\n  def self.greet\n    [:own, super]\n  end\nend\nGreeter.greet";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("own".to_string()),
            Object::symbol("module_copy".to_string())
        ]))
    );
}
