// `instance_eval` handed a String: where it reads constants from, what it
// does with the caller's locals, and the file and line it is counted from.

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

#[test]
fn a_constant_reads_from_the_receivers_singleton_class_first() {
    let answered = run(r#"
module ReceiverScope
  class Receiver
    FOO = :receiver
    def initialize
      singleton_class.const_set(:FOO, :singleton_class)
    end
  end
end
module CallerScope
  class Caller
    FOO = :caller
    def read(receiver)
      receiver.instance_eval("FOO")
    end
  end
end
CallerScope::Caller.new.read(ReceiverScope::Receiver.new)
"#);
    assert_eq!(
        answered,
        Some(Object::symbol("singleton_class".to_string()))
    );
}

#[test]
fn a_constant_reads_from_the_callers_scopes_before_the_receivers_parents() {
    let answered = run(r#"
module ReceiverScope
  class Parent
    FOO = :parent
  end
  class Receiver < Parent
  end
end
module CallerScope
  class Caller
    FOO = :caller
    def read(receiver)
      receiver.instance_eval("FOO")
    end
  end
end
CallerScope::Caller.new.read(ReceiverScope::Receiver.new)
"#);
    assert_eq!(answered, Some(Object::symbol("caller".to_string())));
}

#[test]
fn a_name_the_code_assigns_is_the_callers_own() {
    let answered = run(r#"
held = nil
Object.new.instance_eval("held = :assigned")
held
"#);
    assert_eq!(answered, Some(Object::symbol("assigned".to_string())));
}

#[test]
fn the_file_and_line_it_is_counted_from_are_the_ones_it_was_given() {
    let answered = run(r#"
begin
  Object.new.instance_eval("raise 'from the string'", "a_file", 10)
rescue RuntimeError => refused
  refused.backtrace.first
end
"#);
    assert_eq!(answered, Some(Object::string("a_file:10:in '<main>'")));
}

#[test]
fn a_line_counted_from_below_one_is_still_counted_from_there() {
    let answered = run(r#"
begin
  Object.new.instance_eval("\n\nraise 'from the string'\n", "b_file", -100)
rescue RuntimeError => refused
  refused.backtrace.first
end
"#);
    let Some(Object::String(held)) = answered else {
        panic!("Got something other than a backtrace line")
    };
    assert!(
        held.as_str().starts_with("b_file:-98"),
        "Got: {}",
        held.as_str()
    );
}

#[test]
fn code_with_no_file_named_for_it_reads_as_written_where_the_call_was_made() {
    let answered = run(r#"Object.new.instance_eval("__FILE__")"#);
    let Some(Object::String(held)) = answered else {
        panic!("Got something other than a file name")
    };
    assert!(
        held.as_str().starts_with("(eval at "),
        "Got: {}",
        held.as_str()
    );
}

#[test]
fn the_file_and_the_line_are_taken_the_way_any_other_argument_is() {
    let answered = run(r#"
class NamesAFile
  def to_str
    "from_to_str"
  end
end
class CountsFromALine
  def to_int
    7
  end
end
begin
  Object.new.instance_eval("raise 'held'", NamesAFile.new, CountsFromALine.new)
rescue RuntimeError => refused
  refused.backtrace.first
end
"#);
    assert_eq!(answered, Some(Object::string("from_to_str:7:in '<main>'")));
}

#[test]
fn a_file_that_is_not_a_string_is_refused() {
    let tokens = Lexer::new(r#"Object.new.instance_eval("1", 42)"#).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    let refused = vm.execute_program(&statements).expect_err("should refuse");
    assert!(
        refused
            .to_string()
            .contains("no implicit conversion of Integer into String"),
        "Got: {}",
        refused
    );
}
