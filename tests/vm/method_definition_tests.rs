// Where a `def` installs its method, the visibility it gets, and the
// definitions Ruby refuses.

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

fn run_error(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).unwrap_err().to_string()
}

fn parse_error(code: &str) -> String {
    Parser::new(Lexer::new(code).tokenize())
        .parse()
        .expect_err("the code parsed")
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

fn truth(value: bool) -> Option<Object> {
    Some(Object::Bool(value))
}

#[test]
fn a_method_defined_at_the_top_level_is_private() {
    assert_eq!(
        run("def helper\nend\nObject.private_method_defined?(:helper)"),
        truth(true)
    );
}

#[test]
fn a_method_defined_after_a_top_level_public_is_public() {
    assert_eq!(
        run("public\ndef helper\nend\nprivate\nObject.public_method_defined?(:helper)"),
        truth(true)
    );
}

#[test]
fn a_visibility_modifier_given_names_at_the_top_level_marks_them() {
    assert_eq!(
        run("def helper\nend\npublic :helper\nObject.public_method_defined?(:helper)"),
        truth(true)
    );
}

#[test]
fn initialize_is_private_whatever_visibility_is_in_force() {
    let code = "class Held\n  public\n  def initialize\n  end\n  def respond_to_missing?(name, all)\n    false\n  end\nend\n[Held.private_method_defined?(:initialize), Held.private_method_defined?(:respond_to_missing?)]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![Object::Bool(true), Object::Bool(true)]))
    );
}

#[test]
fn a_def_run_from_a_private_method_is_public_on_its_class() {
    let code = "class Account\n  private\n  def open_ledger\n    def entry\n      :entry\n    end\n  end\nend\nAccount.new.send(:open_ledger)\n[Account.public_method_defined?(:entry), Account.new.entry]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::Bool(true),
            Object::symbol("entry".to_string())
        ]))
    );
}

#[test]
fn a_def_run_from_a_top_level_method_installs_publicly_on_object() {
    let code = "def outer\n  def inner\n    :inner\n  end\nend\nouter\n[Object.public_method_defined?(:inner), Object.new.inner]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::Bool(true),
            Object::symbol("inner".to_string())
        ]))
    );
}

#[test]
fn a_def_run_from_a_method_of_class_new_installs_on_that_class() {
    let code = "built = Class.new do\n  def prepare\n    def prepared\n      :prepared\n    end\n  end\nend\nbuilt.new.prepare\n[built.new.prepared, Object.new.respond_to?(:prepared)]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("prepared".to_string()),
            Object::Bool(false)
        ]))
    );
}

#[test]
fn a_def_run_from_a_block_in_a_method_installs_on_its_class() {
    let code = "class Holder\n  def prepare\n    [1].each do\n      def prepared\n        :prepared\n      end\n    end\n  end\nend\nHolder.new.prepare\nHolder.new.prepared";
    assert_eq!(run(code), Some(Object::symbol("prepared".to_string())));
}

#[test]
fn a_def_in_a_block_run_by_instance_exec_is_not_one_of_the_callers_class() {
    let code = "class Runner\n  def run(&block)\n    Object.new.instance_exec(&block)\n  end\nend\nRunner.new.run do\n  [1].each do\n    def defined_in_exec\n      :ran\n    end\n  end\nend\nRunner.method_defined?(:defined_in_exec)";
    assert_eq!(run(code), truth(false));
}

#[test]
fn a_def_in_a_default_value_installs_on_the_methods_class() {
    let code = "class Greeter\n  def greet(x = (def greet; :hello; end; 1))\n    x\n  end\nend\ngreeter = Greeter.new\n[greeter.greet, greeter.greet]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::Int(1),
            Object::symbol("hello".to_string())
        ]))
    );
}

#[test]
fn eval_through_a_binding_taken_in_a_singleton_class_body_defines_there() {
    let code = "class Holder\n  def add\n    class << self\n      eval(\"def added; :added; end\", binding)\n    end\n  end\nend\nheld = Holder.new\nheld.add\n[held.added, Holder.new.respond_to?(:added)]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("added".to_string()),
            Object::Bool(false)
        ]))
    );
}

#[test]
fn a_def_on_a_constant_receiver_in_a_class_body_defines_a_singleton_method() {
    let code = "class Container\n  TARGET = Object.new\n  def TARGET.label\n    :labeled\n  end\nend\nContainer::TARGET.label";
    assert_eq!(run(code), Some(Object::symbol("labeled".to_string())));
}

#[test]
fn a_def_on_a_parenthesized_local_defines_a_singleton_method() {
    let code = "target = Object.new\ndef (target).label\n  :labeled\nend\ndef (other = Object.new).tag\n  :tagged\nend\n[target.label, other.tag]";
    assert_eq!(
        run(code),
        Some(Object::array(vec![
            Object::symbol("labeled".to_string()),
            Object::symbol("tagged".to_string())
        ]))
    );
}

#[test]
fn a_def_in_a_frozen_class_body_raises_frozen_error() {
    assert!(
        run_error("Frozen = Class.new\nFrozen.freeze\nclass Frozen\n  def refused\n  end\nend")
            .contains("can't modify frozen Class")
    );
}

#[test]
fn a_def_in_a_frozen_module_body_raises_frozen_error() {
    assert!(
        run_error("module Sealed\nend\nSealed.freeze\nmodule Sealed\n  def refused\n  end\nend")
            .contains("can't modify frozen Module")
    );
}

#[test]
fn a_def_in_class_eval_of_a_frozen_class_raises_frozen_error() {
    assert!(
        run_error("held = Class.new\nheld.freeze\nheld.class_eval do\n  def refused\n  end\nend")
            .contains("can't modify frozen Class")
    );
}

#[test]
fn a_singleton_def_on_a_class_whose_singleton_class_is_frozen_raises() {
    assert!(
        run_error("held = Class.new\nheld.singleton_class.freeze\ndef held.refused\nend")
            .contains("can't modify frozen Class")
    );
}

#[test]
fn a_singleton_def_on_a_frozen_module_raises() {
    assert!(
        run_error("held = Module.new\nheld.freeze\ndef held.refused\nend")
            .contains("can't modify frozen Module")
    );
}

#[test]
fn a_def_in_the_singleton_class_of_a_frozen_object_raises() {
    assert!(
        run_error("held = Object.new\nheld.freeze\nclass << held\n  def refused\n  end\nend")
            .contains("FrozenError")
            || run_error(
                "held = Object.new\nheld.freeze\nclass << held\n  def refused\n  end\nend"
            )
            .contains("can't modify frozen")
    );
}

#[test]
fn a_bare_call_missing_arguments_raises_argument_error() {
    assert!(
        run_error("def needs(first, second = 2)\nend\nneeds")
            .contains("wrong number of arguments (given 0, expected 1..2)")
    );
}

#[test]
fn a_bare_call_to_a_method_of_self_missing_arguments_raises() {
    assert!(
        run_error("class Holder\n  def needs(first)\n  end\n  def call_it\n    needs\n  end\nend\nHolder.new.call_it")
            .contains("wrong number of arguments (given 0, expected 1)")
    );
}

#[test]
fn a_default_reading_its_own_parameter_reads_nil() {
    assert_eq!(
        run("def reads_itself(value = value)\n  value\nend\nreads_itself"),
        Some(Object::Nil)
    );
}

#[test]
fn a_method_with_two_splat_parameters_is_refused() {
    assert!(parse_error("def two_splats(*first, *second)\nend").contains("unexpected *"));
}
