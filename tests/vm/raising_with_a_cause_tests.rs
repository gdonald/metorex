// What `raise` does with `cause:`, and where an exception's message comes
// from when a subclass writes its own `initialize`.

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

/// The class and message of whatever the code raises, as one string.
fn refused(code: &str) -> String {
    let source = format!(
        "begin\n{code}\n  \"nothing was raised\"\nrescue Exception => refused\n  \"#{{refused.class}}: #{{refused.message}}\"\nend\n"
    );
    let answered = run(&source);
    match answered {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("Got: {:?}", other),
    }
}

#[test]
fn a_cause_on_its_own_names_nothing_to_raise() {
    assert_eq!(
        refused("  raise(cause: StandardError.new(\"held\"))"),
        "ArgumentError: only cause is given with no arguments"
    );
}

#[test]
fn a_cause_that_is_not_an_exception_is_refused() {
    assert_eq!(
        refused("  raise(\"the new one\", cause: Object.new)"),
        "TypeError: exception object expected"
    );
}

#[test]
fn a_message_followed_by_anything_else_names_no_exception() {
    assert_eq!(
        refused("  raise(\"the new one\", {cause: RuntimeError.new})"),
        "TypeError: exception class/object expected"
    );
}

#[test]
fn a_chain_of_causes_that_runs_back_on_itself_is_refused() {
    assert_eq!(
        refused(
            r#"
  begin
    raise "one"
  rescue => first
    begin
      raise "two"
    rescue
      begin
        raise "three"
      rescue => third
        raise(first, cause: third)
      end
    end
  end
"#
        ),
        "ArgumentError: circular causes"
    );
}

#[test]
fn an_exception_is_not_its_own_cause() {
    let answered = run(r#"
held = StandardError.new("itself")
begin
  raise(held, cause: held)
rescue StandardError => refused
  refused.cause.nil?
end
"#);
    assert_eq!(answered, Some(Object::Bool(true)));
}

#[test]
fn a_named_cause_stands_in_for_the_one_being_handled() {
    let answered = run(r#"
begin
  begin
    raise "being handled"
  rescue
    raise("the new one", cause: StandardError.new("named instead"))
  end
rescue RuntimeError => refused
  refused.cause.message
end
"#);
    assert_eq!(answered, Some(Object::string("named instead")));
}

#[test]
fn a_cause_of_nil_asks_for_none_at_all() {
    let answered = run(r#"
begin
  begin
    raise "being handled"
  rescue
    raise("the new one", cause: nil)
  end
rescue RuntimeError => refused
  refused.cause.nil?
end
"#);
    assert_eq!(answered, Some(Object::Bool(true)));
}

#[test]
fn an_exception_raised_again_keeps_the_cause_it_was_raised_with() {
    let answered = run(r#"
begin
  begin
    raise "one"
  rescue => first
    begin
      raise "two"
    rescue
      raise first
    end
  end
rescue RuntimeError => refused
  refused.cause.nil?
end
"#);
    assert_eq!(answered, Some(Object::Bool(true)));
}

#[test]
fn a_keyword_the_exception_takes_reaches_its_own_constructor() {
    let answered = run(r#"
held = Class.new(StandardError) do
  attr_reader :data
  def initialize(data)
    @data = data
  end
end
begin
  raise(held, data: 42)
rescue => refused
  [refused.data[:data], refused.message == held.to_s]
end
"#);
    let Some(Object::Array(answered)) = answered else {
        panic!("Got something other than a pair")
    };
    assert_eq!(answered.borrow()[0], Object::Int(42));
    assert_eq!(answered.borrow()[1], Object::Bool(true));
}

#[test]
fn a_subclass_takes_its_message_from_what_it_hands_to_super() {
    let answered = run(r#"
class SaidSo < StandardError
  def initialize(named)
    super("from super")
    @named = named
  end
end
class SaidNothing < StandardError
  def initialize(named)
    @named = named
  end
end
[SaidSo.new(1).message, SaidNothing.new("text").message]
"#);
    let Some(Object::Array(answered)) = answered else {
        panic!("Got something other than a pair")
    };
    assert_eq!(answered.borrow()[0], Object::string("from super"));
    assert_eq!(answered.borrow()[1], Object::string("SaidNothing"));
}
