// Coverage tests for arithmetic sequences, the Cartesian product of walks,
// a Yielder made with a block, and how a missing method names its receiver.

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

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

fn number(value: i64) -> Option<Object> {
    Some(Object::Int(value))
}

// ── The sequence a step answers ────────────────────────────────────────────

#[test]
fn a_numeric_step_answers_a_sequence_when_it_is_handed_no_block() {
    assert_eq!(
        run("1.step(10, 3).class.name"),
        text("Enumerator::ArithmeticSequence")
    );
    assert_eq!(run("1.step(10, 3).to_a.inspect"), text("[1, 4, 7, 10]"));
    assert_eq!(run("1.step(10, 3).step"), number(3));
    assert_eq!(run("1.step(10).begin"), number(1));
    assert_eq!(run("1.step(10).end"), number(10));
    assert_eq!(run("1.step(10).exclude_end?"), Some(Object::Bool(false)));
    assert_eq!(
        run("walked = []\n1.step(10, 4) { |value| walked.push(value) }\nwalked.inspect"),
        text("[1, 5, 9]")
    );
}

#[test]
fn a_range_step_carries_the_ranges_own_end() {
    assert_eq!(run("(1...10).step.exclude_end?"), Some(Object::Bool(true)));
    assert_eq!(run("(1..10).step.exclude_end?"), Some(Object::Bool(false)));
    assert_eq!(run("(1...10).step(4).last"), number(9));
    assert_eq!(run("(1..10).step(3).to_a.inspect"), text("[1, 4, 7, 10]"));
    assert_eq!(run("(1...10).step.size"), number(9));
    assert_eq!(run("(1..10).step.size"), number(10));
    assert_eq!(run("((1..10) % 2).to_a.inspect"), text("[1, 3, 5, 7, 9]"));
}

#[test]
fn a_sequence_is_written_the_way_it_was_asked_for() {
    assert_eq!(run("1.step(10).inspect"), text("(1.step(10))"));
    assert_eq!(run("1.step(10, 3).inspect"), text("(1.step(10, 3))"));
    assert_eq!(run("(1..10).step.inspect"), text("((1..10).step)"));
    assert_eq!(run("(1...10).step(3).inspect"), text("((1...10).step(3))"));
    assert_eq!(run("((1..10) % 2).inspect"), text("((1..10).%(2))"));
}

#[test]
fn two_sequences_over_the_same_numbers_are_equal() {
    assert_eq!(
        run("(1..10).step(100) == 1.step(10, 100)"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("(1..10).step == (1...10).step"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("(1..10).step.hash == (1..10).step.hash"),
        Some(Object::Bool(true))
    );
}

#[test]
fn a_sequence_with_no_end_is_endless() {
    assert_eq!(run("1.step(Float::INFINITY).size.infinite?"), number(1));
    assert_eq!(run("(1..).step(1).end"), Some(Object::Nil));
    assert_eq!(run("(..10).step(1).begin"), Some(Object::Nil));
}

#[test]
fn a_sequence_is_made_through_step_rather_than_new() {
    assert_eq!(
        run(
            "begin\n  Enumerator::ArithmeticSequence.new\nrescue NoMethodError => error\n  error.class.name\nend"
        ),
        text("NoMethodError")
    );
    assert_eq!(
        run(
            "begin\n  Enumerator::ArithmeticSequence.allocate\nrescue TypeError => error\n  error.message\nend"
        ),
        text("allocator undefined for Enumerator::ArithmeticSequence")
    );
}

// ── The Cartesian product of walks ─────────────────────────────────────────

#[test]
fn a_product_yields_one_array_per_combination() {
    assert_eq!(
        run("Enumerator::Product.new([1, 2], [:a, :b]).to_a.inspect"),
        text("[[1, :a], [1, :b], [2, :a], [2, :b]]")
    );
    assert_eq!(
        run("Enumerator::Product.new(1..2, 1..3, 1..4).size"),
        number(24)
    );
    assert_eq!(
        run("Enumerator::Product.new([1, 2], [:a, :b]).each.size"),
        number(4)
    );
    assert_eq!(
        run("Enumerator::Product.new([1, 2], [:a, :b]).inspect"),
        text("#<Enumerator::Product: [[1, 2], [:a, :b]]>")
    );
    assert_eq!(
        run("Enumerator::Product.allocate.inspect"),
        text("#<Enumerator::Product: uninitialized>")
    );
}

#[test]
fn a_product_cannot_say_how_long_it_is_when_one_walk_cannot() {
    assert_eq!(
        run(
            "held = Object.new\ndef held.size\n  nil\nend\nEnumerator::Product.new(1..2, held).size"
        ),
        Some(Object::Nil)
    );
    assert_eq!(
        run("held = Object.new\nEnumerator::Product.new(1..2, held).size"),
        Some(Object::Nil)
    );
    assert_eq!(
        run(
            "held = Object.new\ndef held.size\n  Float::INFINITY\nend\nEnumerator::Product.new(1..2, held).size.infinite?"
        ),
        number(1)
    );
}

// ── A Yielder made with a block ────────────────────────────────────────────

#[test]
fn a_yielder_hands_everything_to_the_block_it_was_made_with() {
    assert_eq!(
        run(
            "seen = []\nheld = Enumerator::Yielder.new { |*values| seen.push(values) }\nheld.yield(1, 2)\nheld << 3\nseen.inspect"
        ),
        text("[[1, 2], [3]]")
    );
    assert_eq!(
        run("held = Enumerator::Yielder.new { |value| value + 1 }\nheld.yield(1)"),
        number(2)
    );
    assert_eq!(
        run("held = Enumerator::Yielder.new { |value| value }\n(held << 1).equal?(held)"),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run("Enumerator::Yielder.new { |value| value }.to_proc.class.name"),
        text("Proc")
    );
    assert_eq!(
        run("Enumerator.new { |y| y << 1; y.yield 2, 3; y << [4] }.to_a.inspect"),
        text("[1, [2, 3], [4]]")
    );
}

// ── How a missing method names what it was called on ───────────────────────

#[test]
fn a_missing_method_names_the_receiver_the_way_ruby_does() {
    assert_eq!(
        run("begin\n  Object.new.nowhere\nrescue NoMethodError => error\n  error.message\nend"),
        text("undefined method 'nowhere' for an instance of Object")
    );
    assert_eq!(
        run("begin\n  nil.nowhere\nrescue NoMethodError => error\n  error.message\nend"),
        text("undefined method 'nowhere' for nil")
    );
    assert_eq!(
        run("begin\n  1.nowhere\nrescue NoMethodError => error\n  error.message\nend"),
        text("undefined method 'nowhere' for an instance of Integer")
    );
    assert_eq!(
        run("begin\n  true.nowhere\nrescue NoMethodError => error\n  error.message\nend"),
        text("undefined method 'nowhere' for true")
    );
}

// ── A class that answers allocate itself ───────────────────────────────────

#[test]
fn a_class_that_defines_allocate_answers_with_it() {
    assert_eq!(
        run("class Held\n  def self.allocate\n    \"mine\"\n  end\nend\nHeld.allocate"),
        text("mine")
    );
    assert_eq!(
        run(
            "class Held\n  def self.allocate\n    \"mine\"\n  end\nend\nclass Kept < Held\nend\nKept.allocate"
        ),
        text("mine")
    );
}

// ── Rewinding a walk ───────────────────────────────────────────────────────

#[test]
fn rewinding_a_walk_rewinds_the_object_it_was_cut_from() {
    assert_eq!(
        run(
            "class Counted\n  def rewind\n    @went = true\n    self\n  end\n  def went?\n    @went ? true : false\n  end\n  def each\n    yield 1\n  end\nend\nheld = Counted.new\nheld.to_enum(:each).rewind\nheld.went?"
        ),
        Some(Object::Bool(true))
    );
}
