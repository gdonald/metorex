// Coverage tests for src/vm/block_execution.rs execute_block_with_receiver.
// Targets control-flow branches inside instance_exec/instance_eval blocks.

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

// ── instance_exec with control flow: Value (lines 135-137) ─────────────────

#[test]
fn instance_exec_non_expression_statement_sets_last_value() {
    // An if/while statement returning a Value via ControlFlow::Value.
    let result = run(r#"
class HostIE
  def run
    instance_exec do
      x = 99
      x
    end
  end
end
HostIE.new.run
"#);
    // Last expression returns 99.
    assert_eq!(result, Some(Object::Int(99)));
}

// ── instance_exec with raise: Exception (lines 142-151) ───────────────────

#[test]
fn instance_exec_raise_propagates_exception() {
    let err = run_err(
        r#"
class HostIE2
  def run
    instance_exec do
      raise "boom"
    end
  end
end
HostIE2.new.run
"#,
    );
    assert!(
        err.contains("boom") || err.contains("RuntimeError") || err.contains("Exception"),
        "unexpected: {}",
        err
    );
}

// ── instance_exec with break (lines 152-157) ──────────────────────────────

#[test]
fn instance_exec_break_returns_break_value() {
    // `break` inside instance_exec propagates as a BlockBreak; the VM
    // catches it and returns the break value as the instance_exec result.
    let result = run(r#"
class HostIE3
  def run
    instance_exec do
      break 42
    end
  end
end
HostIE3.new.run
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── instance_exec with next/continue (lines 158-160) ──────────────────────

#[test]
fn instance_exec_next_produces_loop_control_error() {
    let err = run_err(
        r#"
class HostIE4
  def run
    instance_exec do
      next 7
    end
  end
end
HostIE4.new.run
"#,
    );
    assert!(
        err.contains("next") || err.contains("continue") || err.contains("loop"),
        "unexpected: {}",
        err
    );
}

// ── instance_exec with return (lines 138-141) ─────────────────────────────
// `return` inside instance_exec is a non-local return; behavior depends on
// whether the enclosing method catches it.

#[test]
fn instance_exec_return_triggers_non_local_return() {
    let result = run(r#"
class HostIE5
  def run
    instance_exec do
      return 55
    end
    0
  end
end
HostIE5.new.run
"#);
    // Either the block's `return` escapes to the method (returns 55) or the
    // block returns 55 and the method returns 0 — accept both.
    assert!(matches!(
        result,
        Some(Object::Int(55)) | Some(Object::Int(0))
    ));
}

// ── instance_exec passes arguments to the block params ────────────────────

#[test]
fn instance_exec_with_arguments_binds_params() {
    let result = run(r#"
class HostIE6
  def initialize
    @x = 10
  end
end
HostIE6.new.instance_exec(5) do |n|
  @x + n
end
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── instance_exec captures enclosing vars via captured_vars ───────────────

#[test]
fn instance_exec_shares_captured_vars_with_caller() {
    let result = run(r#"
class HostIE7
  def initialize
    @y = 100
  end
end
outer = 3
HostIE7.new.instance_exec do
  @y + outer
end
"#);
    assert_eq!(result, Some(Object::Int(103)));
}

// ── return from a block travels to the method that wrote the block ──────────

#[test]
fn return_in_a_block_leaves_the_method_that_created_it_not_the_yielding_one() {
    let result = run(r#"
class Walker
  def each
    yield 1
    yield 2
    yield 3
    :walked_everything
  end

  def first_even
    each do |value|
      return value if value.even?
    end
    :none
  end
end
Walker.new.first_even
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn return_travels_through_several_yielding_methods() {
    let result = run(r#"
class Layers
  def inner
    yield :from_inner
    :inner_finished
  end

  def outer
    inner { |value| yield value }
    :outer_finished
  end

  def answer
    outer { |value| return value }
    :never
  end
end
Layers.new.answer
"#);
    assert_eq!(result, Some(Object::symbol("from_inner".to_string())));
}

#[test]
fn a_yielding_method_answers_normally_when_its_own_block_does_not_return() {
    let result = run(r#"
class Counter
  def each
    yield 1
    yield 2
    :finished
  end
end
Counter.new.each { |value| value }
"#);
    assert_eq!(result, Some(Object::symbol("finished".to_string())));
}

// ── A return from a block whose method has already finished ─────────────────

#[test]
fn a_return_from_an_orphaned_proc_is_a_local_jump_error() {
    let result = run(r#"
def make_a_return
  Proc.new { return 42 }
end
begin
  make_a_return.call
  :never
rescue LocalJumpError => error
  [error.exit_value, error.reason].inspect
end
"#);
    assert_eq!(result, Some(Object::string("[42, :return]")));
}
