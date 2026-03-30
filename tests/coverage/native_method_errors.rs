// Coverage tests for native method error paths
//
// These tests hit the wrong-argument-count and wrong-argument-type branches
// in native method implementations.

use metorex::lexer::Lexer;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Array method errors ─────────────────────────────────────────────────

#[test]
fn array_length_with_args() {
    let err = run_err("[1].length(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_each_with_args() {
    let err = run_err("[1].each(1) do |x| x end");
    assert!(err.contains("argument"));
}

#[test]
fn array_each_no_block() {
    let err = run_err("[1].each");
    assert!(err.contains("block") || err.contains("Block"));
}

#[test]
fn array_map_with_args() {
    let err = run_err("[1].map(1) { |x| x }");
    assert!(err.contains("argument"));
}

#[test]
fn array_map_no_block() {
    let err = run_err("[1].map");
    assert!(err.contains("block") || err.contains("Block"));
}

#[test]
fn array_filter_with_args() {
    let err = run_err("[1].filter(1) { |x| x }");
    assert!(err.contains("argument"));
}

#[test]
fn array_filter_no_block() {
    let err = run_err("[1].filter");
    assert!(err.contains("block") || err.contains("Block"));
}

#[test]
fn array_pop_with_args() {
    let err = run_err("[1].pop(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_join_wrong_arg_type() {
    let err = run_err("[1, 2].join(42)");
    assert!(err.contains("String") || err.contains("type"));
}

// ── String method errors ────────────────────────────────────────────────

#[test]
fn string_length_with_args() {
    let err = run_err("\"hi\".length(1)");
    assert!(err.contains("argument"));
}

#[test]
fn string_upcase_with_args() {
    let err = run_err("\"hi\".upcase(1)");
    assert!(err.contains("argument"));
}

#[test]
fn string_downcase_with_args() {
    let err = run_err("\"hi\".downcase(1)");
    assert!(err.contains("argument"));
}

#[test]
fn string_plus_wrong_type() {
    let err = run_err("\"a\" + 42");
    assert!(err.contains("type") || err.contains("Type") || err.contains("String"));
}

#[test]
fn string_trim_with_args() {
    let err = run_err("\"hi\".trim(1)");
    assert!(err.contains("argument"));
}

#[test]
fn string_include_wrong_type() {
    let err = run_err("\"hello\".include?(42)");
    assert!(err.contains("String") || err.contains("type"));
}

#[test]
fn string_split_wrong_type() {
    let err = run_err("\"hello\".split(42)");
    assert!(err.contains("String") || err.contains("type"));
}

// ── Hash method errors ──────────────────────────────────────────────────

#[test]
fn hash_keys_with_args() {
    let err = run_err("{\"a\" => 1}.keys(1)");
    assert!(err.contains("argument"));
}

#[test]
fn hash_values_with_args() {
    let err = run_err("{\"a\" => 1}.values(1)");
    assert!(err.contains("argument"));
}

#[test]
fn hash_size_with_args() {
    let err = run_err("{\"a\" => 1}.size(1)");
    assert!(err.contains("argument"));
}

#[test]
fn hash_has_key_wrong_arg_count() {
    let err = run_err("{\"a\" => 1}.has_key?()");
    assert!(err.contains("argument"));
}

#[test]
fn hash_each_with_args() {
    let err = run_err("{\"a\" => 1}.each(1) do |k, v| k end");
    assert!(err.contains("argument"));
}

#[test]
fn hash_each_no_block() {
    let err = run_err("{\"a\" => 1}.each");
    assert!(err.contains("block") || err.contains("Block"));
}

// ── Set method errors ───────────────────────────────────────────────────

#[test]
fn set_add_wrong_arg_count() {
    let err = run_err("s = Set.new\ns.add()");
    assert!(err.contains("argument"));
}

#[test]
fn set_delete_wrong_arg_count() {
    let err = run_err("s = Set.new\ns.delete()");
    assert!(err.contains("argument"));
}

#[test]
fn set_include_wrong_arg_count() {
    let err = run_err("s = Set.new\ns.include?()");
    assert!(err.contains("argument"));
}

#[test]
fn set_union_wrong_arg_count() {
    let err = run_err("s = Set.new\ns.union()");
    assert!(err.contains("argument"));
}

#[test]
fn set_intersection_wrong_arg_count() {
    let err = run_err("s = Set.new\ns.intersection()");
    assert!(err.contains("argument"));
}

#[test]
fn set_difference_wrong_arg_count() {
    let err = run_err("s = Set.new\ns.difference()");
    assert!(err.contains("argument"));
}

#[test]
fn set_size_with_args() {
    let err = run_err("s = Set.new\ns.size(1)");
    assert!(err.contains("argument"));
}

#[test]
fn set_each_no_block() {
    let err = run_err("s = Set.new\ns.each");
    assert!(err.contains("block") || err.contains("Block"));
}

// ── Range method errors ─────────────────────────────────────────────────

#[test]
fn range_each_with_args() {
    let err = run_err("(1..5).each(1) do |x| x end");
    assert!(err.contains("argument"));
}

#[test]
fn range_to_a_with_args() {
    let err = run_err("(1..5).to_a(1)");
    assert!(err.contains("argument"));
}

// ── Int method errors ───────────────────────────────────────────────────

#[test]
fn int_times_with_args() {
    let err = run_err("3.times(1) do |i| i end");
    assert!(err.contains("argument"));
}

#[test]
fn int_times_no_block() {
    let err = run_err("3.times");
    assert!(err.contains("block") || err.contains("Block"));
}

#[test]
fn int_to_s_with_args() {
    let err = run_err("42.to_s(1)");
    assert!(err.contains("argument"));
}

// ── Native function errors ──────────────────────────────────────────────

#[test]
fn gets_with_args() {
    let err = run_err("gets(1)");
    assert!(err.contains("argument"));
}

#[test]
fn assert_no_args() {
    let err = run_err("assert()");
    assert!(err.contains("argument"));
}

#[test]
fn assert_too_many_args() {
    let err = run_err("assert(true, \"msg\", \"extra\")");
    assert!(err.contains("argument"));
}
