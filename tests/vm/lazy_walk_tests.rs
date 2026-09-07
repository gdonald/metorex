// Coverage tests for the lazy walk and the Enumerable grouping methods

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

// ── A step reports the size it leaves behind ────────────────────────────────

#[test]
fn lazy_map_keeps_the_source_size() {
    assert_eq!(
        run("(1..10).lazy.map { |n| n }.size"),
        Some(Object::Int(10))
    );
}

#[test]
fn lazy_take_reports_the_smaller_of_the_count_and_the_size() {
    assert_eq!(run("(1..10).lazy.take(4).size"), Some(Object::Int(4)));
    assert_eq!(run("(1..10).lazy.take(40).size"), Some(Object::Int(10)));
}

#[test]
fn lazy_drop_reports_what_is_left_after_the_count() {
    assert_eq!(run("(1..10).lazy.drop(4).size"), Some(Object::Int(6)));
    assert_eq!(run("(1..10).lazy.drop(40).size"), Some(Object::Int(0)));
}

#[test]
fn lazy_select_reports_no_size() {
    assert_eq!(run("(1..10).lazy.select { |n| n }.size"), Some(Object::Nil));
}

#[test]
fn lazy_size_is_nil_for_a_source_that_reports_none() {
    assert_eq!(
        run("Enumerator.new { |y| y << 1 }.lazy.size"),
        Some(Object::Nil)
    );
}

// ── A walk with no end still answers first ─────────────────────────────────

#[test]
fn lazy_first_stops_a_walk_that_never_ends() {
    assert_eq!(
        run("(1..Float::INFINITY).lazy.map { |n| n * 2 }.first(3).inspect"),
        Some(Object::string("[2, 4, 6]"))
    );
}

#[test]
fn lazy_first_without_a_count_answers_one_element() {
    assert_eq!(run("(5..Float::INFINITY).lazy.first"), Some(Object::Int(5)));
}

#[test]
fn lazy_take_of_zero_walks_nothing() {
    assert_eq!(
        run("(1..Float::INFINITY).lazy.take(0).force.inspect"),
        Some(Object::string("[]"))
    );
}

#[test]
fn lazy_take_refuses_a_negative_count() {
    let err = run_err("[1, 2].lazy.take(-1)");
    assert!(err.contains("negative size"), "Error was: {}", err);
}

#[test]
fn lazy_drop_refuses_a_negative_count() {
    let err = run_err("[1, 2].lazy.drop(-1)");
    assert!(err.contains("negative size"), "Error was: {}", err);
}

#[test]
fn lazy_map_refuses_a_missing_block() {
    let err = run_err("[1, 2].lazy.map");
    assert!(err.contains("without a block"), "Error was: {}", err);
}

#[test]
fn lazy_new_refuses_a_missing_block() {
    let err = run_err("Enumerator::Lazy.new([1, 2])");
    assert!(err.contains("without a block"), "Error was: {}", err);
}

#[test]
fn lazy_with_index_refuses_an_offset_that_is_not_an_integer() {
    let err = run_err("[1, 2].lazy.with_index(false).force");
    assert!(err.contains("into Integer"), "Error was: {}", err);
}

#[test]
fn lazy_with_index_treats_a_nil_offset_as_zero() {
    assert_eq!(
        run("[:a, :b].lazy.with_index(nil).force.inspect"),
        Some(Object::string("[[:a, 0], [:b, 1]]"))
    );
}

#[test]
fn lazy_zip_refuses_an_argument_that_is_not_a_list() {
    let err = run_err("[1, 2].lazy.zip(Object.new)");
    assert!(err.contains("wrong argument type"), "Error was: {}", err);
}

#[test]
fn lazy_new_runs_its_block_against_a_yielder() {
    assert_eq!(
        run("Enumerator::Lazy.new([1, 2]) { |y, v| y << v * 10 }.force.inspect"),
        Some(Object::string("[10, 20]"))
    );
}

#[test]
fn lazy_new_reads_a_size_given_as_a_proc() {
    assert_eq!(
        run("Enumerator::Lazy.new([1], -> { 200 }) {}.size"),
        Some(Object::Int(200))
    );
}

// ── The runs the grouping methods cut ──────────────────────────────────────

#[test]
fn enumerable_chunk_groups_a_run_under_its_key() {
    assert_eq!(
        run("[1, 1, 2, 3, 3].chunk { |n| n }.to_a.inspect"),
        Some(Object::string("[[1, [1, 1]], [2, [2]], [3, [3, 3]]]"))
    );
}

#[test]
fn enumerable_chunk_without_a_block_answers_an_enumerator() {
    assert_eq!(
        run("[1, 2].chunk.class.name"),
        Some(Object::string("Enumerator"))
    );
}

#[test]
fn enumerable_chunk_while_keeps_a_run_going_while_the_block_agrees() {
    assert_eq!(
        run("[1, 2, 4, 5].chunk_while { |a, b| b == a + 1 }.to_a.inspect"),
        Some(Object::string("[[1, 2], [4, 5]]"))
    );
}

#[test]
fn enumerable_chunk_while_refuses_a_missing_block() {
    let err = run_err("[1, 2].chunk_while");
    assert!(err.contains("without a block"), "Error was: {}", err);
}

#[test]
fn enumerable_slice_when_cuts_where_the_block_agrees() {
    assert_eq!(
        run("[1, 2, 4, 5].slice_when { |a, b| b != a + 1 }.to_a.inspect"),
        Some(Object::string("[[1, 2], [4, 5]]"))
    );
}

#[test]
fn enumerable_slice_when_refuses_a_missing_block() {
    let err = run_err("[1, 2].slice_when");
    assert!(err.contains("without a block"), "Error was: {}", err);
}

#[test]
fn enumerable_slice_before_starts_a_run_at_every_match() {
    assert_eq!(
        run("[1, 2, 3, 4].slice_before { |n| n.odd? }.to_a.inspect"),
        Some(Object::string("[[1, 2], [3, 4]]"))
    );
}

#[test]
fn enumerable_slice_before_takes_a_pattern() {
    assert_eq!(
        run("[1, 2, 3, 4].slice_before(3).to_a.inspect"),
        Some(Object::string("[[1, 2], [3, 4]]"))
    );
}

#[test]
fn enumerable_slice_after_ends_a_run_at_every_match() {
    assert_eq!(
        run("[1, 2, 3, 4].slice_after { |n| n.even? }.to_a.inspect"),
        Some(Object::string("[[1, 2], [3, 4]]"))
    );
}

#[test]
fn enumerable_slice_after_takes_a_pattern() {
    assert_eq!(
        run("[1, 2, 3, 4].slice_after(2).to_a.inspect"),
        Some(Object::string("[[1, 2], [3, 4]]"))
    );
}

#[test]
fn enumerable_grouping_of_nothing_answers_nothing() {
    assert_eq!(
        run("[].chunk { |n| n }.to_a.inspect"),
        Some(Object::string("[]"))
    );
    assert_eq!(
        run("[].chunk_while { |a, b| true }.to_a.inspect"),
        Some(Object::string("[]"))
    );
    assert_eq!(
        run("[].slice_before { |n| true }.to_a.inspect"),
        Some(Object::string("[]"))
    );
    assert_eq!(
        run("[].slice_after { |n| true }.to_a.inspect"),
        Some(Object::string("[]"))
    );
}

// ── A chain walks its parts in order ───────────────────────────────────────

#[test]
fn chain_reports_the_sum_of_the_sizes_it_walks() {
    assert_eq!(run("[1, 2].chain([3]).size"), Some(Object::Int(3)));
}

#[test]
fn chain_reports_no_size_when_a_part_reports_none() {
    assert_eq!(
        run("[1, 2].chain(Enumerator.new { |y| y << 1 }).size"),
        Some(Object::Nil)
    );
}

#[test]
fn chain_that_was_never_built_says_so() {
    assert_eq!(
        run("Enumerator::Chain.allocate.inspect"),
        Some(Object::string("#<Enumerator::Chain: uninitialized>"))
    );
}

#[test]
fn chain_rewinds_to_itself() {
    assert_eq!(
        run("c = [1].chain([2])\nc.rewind.equal?(c)"),
        Some(Object::Bool(true))
    );
}
