// What a `break` in a block returns from, and the LocalJumpError it raises
// once the call its block was attached to has returned.

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
    Parser::new(Lexer::new(code).tokenize())
        .parse()
        .expect_err("the code parsed")
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

const CAPTURE: &str = "def capture(&block)\n  block\nend\ndef run_it\n  yield\nend\n";

fn symbol(name: &str) -> Option<Object> {
    Some(Object::symbol(name.to_string()))
}

#[test]
fn a_break_from_a_captured_block_raises_local_jump_error() {
    let code = format!(
        "{CAPTURE}held = capture {{ break :done }}\nbegin\n  held.call\nrescue LocalJumpError => error\n  error.message\nend"
    );
    assert_eq!(run(&code), Some(Object::string("break from proc-closure")));
}

#[test]
fn a_break_in_a_statement_begin_is_rescued_in_the_block() {
    let code = format!(
        "{CAPTURE}held = capture do\n  begin\n    break :inside\n  rescue LocalJumpError\n    :rescued\n  end\nend\nheld.call"
    );
    assert_eq!(run(&code), symbol("rescued"));
}

#[test]
fn a_break_in_a_statement_begin_returns_from_a_running_call() {
    let code = format!("{CAPTURE}run_it do\n  begin\n    break :left\n  end\n  :missed\nend");
    assert_eq!(run(&code), symbol("left"));
}

#[test]
fn a_break_in_a_while_loop_inside_a_finished_block_ends_the_loop() {
    let code = format!(
        "{CAPTURE}held = capture do\n  count = 0\n  while true\n    begin\n      count += 1\n      break if count == 3\n    end\n  end\n  count\nend\nheld.call"
    );
    assert_eq!(run(&code), Some(Object::Int(3)));
}

#[test]
fn a_break_in_an_until_and_a_for_loop_inside_a_finished_block_ends_the_loop() {
    let code = format!(
        "{CAPTURE}held = capture do\n  seen = []\n  begin\n    seen << 1\n    break\n  end until false\n  for value in [2, 3]\n    seen << value\n    break\n  end\n  seen\nend\nheld.call"
    );
    assert_eq!(
        run(&code),
        Some(Object::array(vec![Object::Int(1), Object::Int(2)]))
    );
}

#[test]
fn a_break_in_a_block_written_on_super_returns_from_super() {
    let code = "class Parent\n  def each_value\n    yield 1\n    :finished\n  end\nend\nclass Child < Parent\n  def each_value\n    super { break :from_super }\n  end\nend\nChild.new.each_value";
    assert_eq!(run(code), symbol("from_super"));
}

#[test]
fn a_super_without_a_block_answers_what_the_parent_answers() {
    let code = "class Parent\n  def value\n    :parent\n  end\nend\nclass Child < Parent\n  def value\n    super\n  end\nend\nChild.new.value";
    assert_eq!(run(code), symbol("parent"));
}

#[test]
fn a_break_in_a_lambda_made_by_kernel_lambda_ends_the_lambda() {
    let code = "held = lambda { break :from_lambda }\nheld.call";
    assert_eq!(run(code), symbol("from_lambda"));
}

#[test]
fn a_break_in_a_block_resumed_on_a_fiber_returns_from_its_call() {
    let code = "fiber = Fiber.new do\n  [1, 2].each do |value|\n    Fiber.yield value\n    break :stopped\n  end\nend\nfiber.resume\nfiber.resume";
    assert_eq!(run(code), symbol("stopped"));
}

#[test]
fn a_break_in_a_method_body_is_refused() {
    assert!(parse_error("def stops\n  break\nend").contains("Invalid break"));
}

#[test]
fn a_break_in_a_module_body_is_refused() {
    assert!(parse_error("module Stops\n  break\nend").contains("Invalid break"));
}

#[test]
fn a_break_in_a_class_body_inside_a_block_is_refused() {
    assert!(
        parse_error("[1].each do\n  class Stops\n    break\n  end\nend").contains("Invalid break")
    );
}

#[test]
fn a_break_in_a_loop_in_a_class_body_is_accepted() {
    let code = "class Counted\n  COUNT = [1, 2, 3].each { |value| break value if value == 2 }\nend\nCounted::COUNT";
    assert_eq!(run(code), Some(Object::Int(2)));
}
