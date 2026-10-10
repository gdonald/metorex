// The method lookup cache: a name is looked up through a class's ancestry
// once, and again only after a method table or a mixin chain changes.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// The VM after running `code`, with the classes it defined still in hand,
/// and what the program answered.
fn ran_answering(code: &str) -> (VirtualMachine, Option<Object>) {
    let statements = Parser::new(Lexer::new(code).tokenize())
        .parse()
        .expect("parse failed");
    let mut vm = VirtualMachine::new();
    let answer = vm.execute_program(&statements).expect("execution failed");
    (vm, answer)
}

fn ran(code: &str) -> VirtualMachine {
    ran_answering(code).0
}

/// How many times the class named `class_name` walked its ancestry for
/// `method_name`.
fn walks(vm: &VirtualMachine, class_name: &str, method_name: &str) -> u64 {
    match vm.globals().get(class_name) {
        Some(Object::Class(class)) => class.method_lookup_walks(method_name),
        other => panic!("expected a class named {class_name}, got {other:?}"),
    }
}

const COUNTER: &str = "
module Doubling
  def double(amount)
    amount * 2
  end
end

class Counter
  include Doubling

  def initialize
    @total = 0
  end

  def add(amount)
    @total += double(amount)
  end
end

counter = Counter.new
";

#[test]
fn a_thousand_calls_of_one_method_walk_the_ancestry_once() {
    let vm = ran(&format!(
        "{COUNTER}1000.times {{ |step| counter.add(step) }}\n"
    ));
    assert_eq!(walks(&vm, "Counter", "add"), 1);
}

#[test]
fn a_method_found_through_a_module_is_walked_for_once() {
    let vm = ran(&format!(
        "{COUNTER}1000.times {{ |step| counter.add(step) }}\n"
    ));
    assert_eq!(walks(&vm, "Counter", "double"), 1);
}

#[test]
fn defining_a_method_anywhere_makes_the_next_call_walk_again() {
    let vm = ran(&format!(
        "{COUNTER}counter.add(1)\nclass Unrelated\n  def other; end\nend\ncounter.add(2)\n"
    ));
    assert_eq!(walks(&vm, "Counter", "add"), 2);
}

#[test]
fn including_a_module_makes_the_next_call_walk_again() {
    let vm = ran(&format!(
        "{COUNTER}counter.add(1)\nmodule Louder\nend\nclass Counter\n  include Louder\nend\ncounter.add(2)\n"
    ));
    assert_eq!(walks(&vm, "Counter", "add"), 2);
}

#[test]
fn a_method_redefined_in_a_module_is_walked_for_again() {
    let vm = ran(&format!(
        "{COUNTER}counter.add(1)\nmodule Doubling\n  def double(amount)\n    amount * 10\n  end\nend\ncounter.add(1)\n"
    ));
    assert_eq!(walks(&vm, "Counter", "double"), 2);
}

#[test]
fn a_method_redefined_in_a_module_is_the_one_called_next() {
    let (_, answer) = ran_answering(&format!(
        "{COUNTER}counter.add(1)\nmodule Doubling\n  def double(amount)\n    amount * 10\n  end\nend\ncounter.add(1)\ncounter.instance_variable_get(:@total)\n"
    ));
    assert_eq!(answer, Some(Object::Int(12)));
}

#[test]
fn removing_a_method_is_seen_by_the_next_lookup() {
    let (_, answer) = ran_answering(&format!(
        "{COUNTER}counter.add(1)\nclass Counter\n  remove_method :add\nend\ncounter.respond_to?(:add)\n"
    ));
    assert_eq!(answer, Some(Object::Bool(false)));
}

#[test]
fn an_alias_written_over_a_method_is_the_one_called_next() {
    let (_, answer) = ran_answering(&format!(
        "{COUNTER}class Counter\n  def total = @total\n  def none = 0\nend\ncounter.add(3)\nfirst = counter.total\nclass Counter\n  alias total none\nend\n[first, counter.total]\n"
    ));
    assert_eq!(
        answer.map(|held| held.to_string()),
        Some("[6, 0]".to_string())
    );
}
