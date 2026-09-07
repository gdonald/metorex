// Enumerable is written over `each` in the prelude, so its methods have to sit
// behind whatever the receiver's own class supplies.

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

#[test]
fn a_struct_answers_its_own_to_h_rather_than_the_one_from_enumerable() {
    let result = run(r#"
Point = Struct.new(:x, :y)
Point.new(1, 2).to_h.inspect
"#);
    assert_eq!(result, Some(Object::string("{x: 1, y: 2}")));
}

#[test]
fn a_struct_answers_its_own_to_h_when_given_a_block() {
    let result = run(r#"
Point = Struct.new(:x, :y)
Point.new(1, 2).to_h { |name, value| [name.to_s, value * 10] }.inspect
"#);
    assert_eq!(result, Some(Object::string(r#"{"x" => 10, "y" => 20}"#)));
}

#[test]
fn a_class_that_defines_a_method_itself_wins_over_enumerable() {
    let result = run(r#"
class Shouty
  include Enumerable
  def each
    yield 1
    yield 2
  end
  def to_a
    :mine
  end
end
Shouty.new.to_a
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("mine".to_string())))
    );
}

#[test]
fn enumerable_supplies_a_walk_over_a_class_that_only_defines_each() {
    let result = run(r#"
class Numerous
  include Enumerable
  def each
    yield 3
    yield 1
    yield 2
  end
end
Numerous.new.sort.inspect
"#);
    assert_eq!(result, Some(Object::string("[1, 2, 3]")));
}

#[test]
fn an_enumerator_from_a_missing_block_reports_the_receivers_size() {
    let result = run(r#"
class Sized
  include Enumerable
  def each
    yield 1
    yield 2
  end
  def size
    2
  end
end
Sized.new.map.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn an_enumerator_size_is_nil_when_the_receiver_reports_none() {
    let result = run(r#"
class Unsized
  include Enumerable
  def each
    yield 1
  end
end
Unsized.new.map.size.inspect
"#);
    assert_eq!(result, Some(Object::string("nil")));
}

#[test]
fn a_multi_value_yield_reaches_the_block_as_one_packed_array() {
    let result = run(r#"
class Pairs
  include Enumerable
  def each
    yield 1, 2
    yield 3, 4
  end
end
Pairs.new.select { |pair| pair == [3, 4] }.inspect
"#);
    assert_eq!(result, Some(Object::string("[[3, 4]]")));
}

#[test]
fn a_proc_argument_stays_positional_when_the_call_brings_its_own_block() {
    let result = run(r#"
def pick(fallback = nil, &block)
  return fallback.call if fallback && !block.call
  :from_block
end
pick(-> { :from_fallback }) { false }
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new(
            "from_fallback".to_string()
        )))
    );
}

#[test]
fn a_singleton_class_can_alias_a_native_method_of_the_object_it_belongs_to() {
    let result = run(r#"
values = [1, 2, 3]
values.singleton_class.send(:alias_method, :every, :to_a)
values.every.inspect
"#);
    assert_eq!(result, Some(Object::string("[1, 2, 3]")));
}

#[test]
fn a_break_in_a_block_leaves_the_method_that_was_handed_the_block() {
    let result = run(r#"
class Walker
  include Enumerable
  def each
    [4, 3, 2, 1].each { |value| yield value }
  end
end
Walker.new.take_while { |value| break :stopped if value == 3; true }
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("stopped".to_string())))
    );
}

#[test]
fn a_break_inside_a_nested_iteration_still_belongs_to_the_inner_call() {
    let result = run(r#"
answers = []
[1, 2].each do |outer|
  found = [10, 20].map do |inner|
    break :short if inner == 20
    inner
  end
  answers.push(found)
end
answers.inspect
"#);
    assert_eq!(result, Some(Object::string("[:short, :short]")));
}

#[test]
fn nil_true_and_false_answer_to_their_own_classes() {
    let result = run(r#"
[
  NilClass === nil,
  TrueClass === true,
  FalseClass === false,
  nil.is_a?(NilClass),
  true.is_a?(TrueClass),
  false.is_a?(FalseClass),
  NilClass === 1,
  TrueClass === false
].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(
            "[true, true, true, true, true, true, false, false]"
        ))
    );
}

#[test]
fn an_optional_parameter_takes_its_default_when_a_splat_follows_it() {
    let result = run(r#"
def labelled(name = :unnamed, *rest)
  [name, rest]
end
[labelled(), labelled(:given), labelled(:given, 1, 2)].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(
            "[[:unnamed, []], [:given, []], [:given, [1, 2]]]"
        ))
    );
}

#[test]
fn an_enumerator_walked_with_a_block_answers_what_the_underlying_method_answers() {
    let result = run(r#"
[1, 2, 3].map.each { |value| value * 2 }.inspect
"#);
    assert_eq!(result, Some(Object::string("[2, 4, 6]")));
}

#[test]
fn a_singleton_method_can_be_defined_on_a_parenthesized_assignment() {
    let result = run(r#"
class Matchers
  def build
    def (@matcher = Object.new).===(other)
      other.odd?
    end
    @matcher
  end
end
[1, 2, 3, 4].grep(Matchers.new.build).inspect
"#);
    assert_eq!(result, Some(Object::string("[1, 3]")));
}

#[test]
fn inject_folds_through_an_operator_named_by_a_symbol_or_a_string() {
    let result = run(r#"
class Numerous
  include Enumerable
  def each
    yield 1
    yield 2
    yield 3
  end
end
[
  Numerous.new.inject(:+),
  Numerous.new.inject("+"),
  Numerous.new.inject(10, :-),
  [1, 2, 3].inject(:+),
  [1, 2, 3].inject(10, :-)
].inspect
"#);
    assert_eq!(result, Some(Object::string("[6, 6, 4, 6, 4]")));
}

#[test]
fn inject_rejects_an_operator_name_that_is_not_a_symbol_or_a_string() {
    let tokens = metorex::lexer::Lexer::new("[1, 2].inject(0, Object.new)").tokenize();
    let stmts = metorex::parser::Parser::new(tokens)
        .parse()
        .expect("parse failed");
    let mut vm = VirtualMachine::new();
    let error = vm.execute_program(&stmts).unwrap_err().to_string();
    assert!(error.contains("is not a symbol nor a string"), "{}", error);
}

#[test]
fn inject_reaches_elements_a_block_appends_while_it_walks() {
    let result = run(r#"
grown = [1, 2, 3]
seen = []
grown.inject(nil) do |_, value|
  seen.push(value)
  grown.push(value + 10) if value < 4
  nil
end
seen.inspect
"#);
    assert_eq!(result, Some(Object::string("[1, 2, 3, 11, 12, 13]")));
}

#[test]
fn grep_keeps_what_a_pattern_matches_and_grep_v_keeps_the_rest() {
    let result = run(r#"
class Words
  include Enumerable
  def each
    yield "apple"
    yield "banana"
  end
end
[
  Words.new.grep(/an/),
  Words.new.grep_v(/an/),
  Words.new.grep(/an/) { |word| word.upcase }
].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(r#"[["banana"], ["apple"], ["BANANA"]]"#))
    );
}
