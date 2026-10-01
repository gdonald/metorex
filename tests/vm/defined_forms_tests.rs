// What `defined?` reports for each form it is handed, and what it runs to
// find out.

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

#[test]
fn a_hash_literal_is_defined_when_every_key_and_value_is() {
    assert_eq!(
        run("[defined?({a: Object}), defined?({a: Missing})]"),
        Some(Object::array(vec![
            Object::string("expression"),
            Object::Nil
        ]))
    );
}

#[test]
fn a_splat_in_an_array_literal_is_defined_by_what_it_splats() {
    assert_eq!(
        run("[defined?([*Object]), defined?([*Missing])]"),
        Some(Object::array(vec![
            Object::string("expression"),
            Object::Nil
        ]))
    );
}

#[test]
fn a_bare_call_with_arguments_is_a_method_when_self_answers_it() {
    assert_eq!(
        run(
            "def helper(value)\nend\n[defined?(helper(1)), defined?(missing_helper(1)), defined?(helper(Missing))]"
        ),
        Some(Object::array(vec![
            Object::string("method"),
            Object::Nil,
            Object::Nil
        ]))
    );
}

#[test]
fn a_call_on_self_reaches_a_private_method() {
    let code = "class Holder\n  def check\n    defined?(self.hidden)\n  end\n  private\n  def hidden; end\nend\nHolder.new.check";
    assert_eq!(run(code), text("method"));
}

#[test]
fn a_private_method_called_on_another_receiver_is_not_defined() {
    let code = "class Holder\n  private\n  def hidden; end\nend\ndefined?(Holder.new.hidden)";
    assert_eq!(run(code), Some(Object::Nil));
}

#[test]
fn a_safe_call_asks_about_the_method_behind_it() {
    assert_eq!(
        run("held = 'text'\n[defined?(held&.upcase), defined?(held&.missing)]"),
        Some(Object::array(vec![Object::string("method"), Object::Nil]))
    );
}

#[test]
fn respond_to_missing_answers_for_a_method_a_receiver_lacks() {
    let code = "class Dynamic\n  def respond_to_missing?(name, include_private)\n    name == :anything\n  end\nend\ndefined?(Dynamic.new.anything)";
    assert_eq!(run(code), text("method"));
}

#[test]
fn an_exception_raised_by_the_receiver_makes_the_answer_nil() {
    let code = "def fails\n  raise 'broken'\nend\ndefined?(fails.upcase)";
    assert_eq!(run(code), Some(Object::Nil));
}

#[test]
fn an_element_read_is_a_method_on_the_receiver() {
    assert_eq!(
        run("items = [1]\n[defined?(items[0]), defined?(items[Missing])]"),
        Some(Object::array(vec![Object::string("method"), Object::Nil]))
    );
}

#[test]
fn an_element_assignment_is_a_method_and_a_compound_one_an_assignment() {
    assert_eq!(
        run("items = [1]\n[defined?(items[0] = 2), defined?(items[0] += 2), items]"),
        Some(Object::array(vec![
            Object::string("method"),
            Object::string("assignment"),
            Object::array(vec![Object::Int(1)])
        ]))
    );
}

#[test]
fn a_unary_minus_is_a_method_on_its_operand() {
    assert_eq!(
        run("count = 3\n[defined?(-count), defined?(-missing_value)]"),
        Some(Object::array(vec![Object::string("method"), Object::Nil]))
    );
}

#[test]
fn a_class_variable_is_defined_once_set() {
    let code = "class Holder\n  @@count = 1\n  def self.check\n    [defined?(@@count), defined?(@@missing)]\n  end\nend\nHolder.check";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::string("class variable"),
            Object::Nil
        ]))
    );
}

#[test]
fn an_instance_variable_of_a_class_is_defined_once_set() {
    let code = "class Holder\n  @count = 1\n  def self.check\n    [defined?(@count), defined?(@missing)]\n  end\nend\nHolder.check";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::string("instance-variable"),
            Object::Nil
        ]))
    );
}

#[test]
fn a_global_given_a_second_name_is_defined_through_either() {
    assert_eq!(
        run("$first_name = 1\nalias $second_name $first_name\ndefined?($second_name)"),
        text("global-variable")
    );
}

#[test]
fn a_top_level_constant_is_defined_and_a_private_one_is_not() {
    let code = "class Object\n  HIDDEN_HERE = 1\n  private_constant :HIDDEN_HERE\nend\n[defined?(::String), defined?(::HIDDEN_HERE), defined?(::MISSING_HERE)]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::string("constant"),
            Object::Nil,
            Object::Nil
        ]))
    );
}

#[test]
fn a_scoped_constant_on_an_undefined_namespace_is_not_defined() {
    assert_eq!(
        run("[defined?(Missing::String), defined?(Object::String), defined?(3::String)]"),
        Some(Object::array(vec![
            Object::Nil,
            Object::string("constant"),
            Object::Nil
        ]))
    );
}

#[test]
fn super_from_a_class_method_is_defined_when_the_superclass_has_one() {
    let code = "class Parent\n  def self.build; end\nend\nclass Child < Parent\n  def self.build\n    defined?(super)\n  end\n  def self.other\n    defined?(super)\n  end\nend\n[Child.build, Child.other]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::string("super"), Object::Nil]))
    );
}

#[test]
fn super_outside_a_method_is_not_defined() {
    assert_eq!(run("defined?(super)"), Some(Object::Nil));
}

#[test]
fn a_constant_answers_constant_and_a_missing_one_nil() {
    assert_eq!(
        run("[defined?(String), defined?(MissingConstant), defined?(puts)]"),
        Some(Object::array(vec![
            Object::string("constant"),
            Object::Nil,
            Object::string("method")
        ]))
    );
}

#[test]
fn a_constant_inside_a_basic_object_subclass_is_not_the_top_levels() {
    let code = "class Bare < BasicObject\n  LOCAL = 1\n  def self.check\n    [defined?(String), defined?(LOCAL)]\n  end\nend\nBare.check";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::Nil, Object::string("constant")]))
    );
}

#[test]
fn defined_in_void_context_runs_nothing_and_warns_when_verbose() {
    let code = "$VERBOSE = true\nwarned = []\ncollector = Object.new\ncollector.define_singleton_method(:write) { |text| warned << text }\n$stderr = collector\nran = []\neval(\"defined?(ran << 1); 2\")\n$stderr = STDERR\n[ran, warned.join.include?('possibly useless use of defined? in void context')]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::array(Vec::new()),
            Object::Bool(true)
        ]))
    );
}

#[test]
fn defined_as_the_last_statement_is_evaluated() {
    assert_eq!(run("value = 1\ndefined?(value)"), text("local-variable"));
}

#[test]
fn a_mutable_string_literal_is_an_expression() {
    assert_eq!(
        run("# frozen_string_literal: false\ndefined?('text')"),
        text("expression")
    );
}
