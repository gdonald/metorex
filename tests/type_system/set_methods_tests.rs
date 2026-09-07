// Tests for Set native methods

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

// ============================================================================
// Set.new
// ============================================================================

#[test]
fn set_new_empty() {
    let result = run("Set.new.size");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn set_new_from_array() {
    let result = run("Set.new([1, 2, 3]).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_new_deduplicates() {
    let result = run("Set.new([1, 1, 2, 2, 3]).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_new_error_non_array_arg() {
    let err = run_err("Set.new(42)");
    assert!(err.contains("Array") || err.contains("type"));
}

#[test]
fn set_new_error_too_many_args() {
    let err = run_err("Set.new([1], [2])");
    assert!(err.contains("argument"));
}

// ============================================================================
// add / insert
// ============================================================================

#[test]
fn set_add_element() {
    let result = run(r#"
s = Set.new
s.add(1)
s.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn set_add_duplicate_no_change() {
    let result = run(r#"
s = Set.new([1, 2])
s.add(1)
s.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_insert_alias() {
    let result = run(r#"
s = Set.new
s.insert(42)
s.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn set_add_returns_set() {
    let result = run(r#"
s = Set.new
s.add(1).add(2).size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_add_error_no_args() {
    let err = run_err("Set.new.add");
    assert!(err.contains("argument"));
}

// ============================================================================
// remove / delete
// ============================================================================

#[test]
fn set_remove_existing() {
    let result = run(r#"
s = Set.new([1, 2, 3])
s.remove(2)
s.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_remove_returns_bool() {
    let result = run("Set.new([1, 2]).remove(1)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_remove_missing_returns_false() {
    let result = run("Set.new([1, 2]).remove(99)");
    assert_eq!(result, Some(Object::Bool(false)));
}

// `delete` answers the set itself, so removals chain.
#[test]
fn set_delete_answers_the_set() {
    let result = run("Set.new([1, 2]).delete(1).to_a.inspect");
    assert_eq!(result, Some(Object::string("[2]")));
}

// `remove` reports whether it removed anything.
#[test]
fn set_remove_reports_whether_it_removed_anything() {
    let result = run("[Set.new([1]).remove(1), Set.new([1]).remove(9)].inspect");
    assert_eq!(result, Some(Object::string("[true, false]")));
}

// ============================================================================
// contains? / include? / has?
// ============================================================================

#[test]
fn set_contains_true() {
    let result = run(r#"Set.new([1, 2, 3]).contains?(2)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_contains_false() {
    let result = run(r#"Set.new([1, 2, 3]).contains?(99)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn set_include_alias() {
    let result = run(r#"Set.new([1, 2]).include?(1)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_has_alias() {
    let result = run(r#"Set.new([1, 2]).has?(2)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_contains_string() {
    let result = run(r#"Set.new(["a", "b"]).contains?("a")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ============================================================================
// size / length
// ============================================================================

#[test]
fn set_size() {
    let result = run("Set.new([1, 2, 3]).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_length_alias() {
    let result = run("Set.new([1, 2]).length");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_size_error_with_args() {
    let err = run_err("Set.new.size(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// empty?
// ============================================================================

#[test]
fn set_empty_true() {
    let result = run("Set.new.empty?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_empty_false() {
    let result = run("Set.new([1]).empty?");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ============================================================================
// to_a
// ============================================================================

#[test]
fn set_to_a_returns_array() {
    let result = run("Set.new([1]).to_a.length");
    assert_eq!(result, Some(Object::Int(1)));
}

// ============================================================================
// union
// ============================================================================

#[test]
fn set_union() {
    let result = run("Set.new([1, 2]).union(Set.new([2, 3])).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_union_with_a_non_enumerable_errors() {
    let err = run_err("Set.new([1]).union(42)");
    assert!(err.contains("must be enumerable"), "{}", err);
}

// ============================================================================
// intersection
// ============================================================================

#[test]
fn set_intersection() {
    let result = run("Set.new([1, 2, 3]).intersection(Set.new([2, 3, 4])).size");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_intersection_empty() {
    let result = run("Set.new([1, 2]).intersection(Set.new([3, 4])).size");
    assert_eq!(result, Some(Object::Int(0)));
}

// ============================================================================
// difference
// ============================================================================

#[test]
fn set_difference() {
    let result = run("Set.new([1, 2, 3]).difference(Set.new([2, 3])).size");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn set_difference_contains_correct_element() {
    let result = run("Set.new([1, 2, 3]).difference(Set.new([2, 3])).contains?(1)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ============================================================================
// each
// ============================================================================

#[test]
fn set_each_iterates() {
    let result = run(r#"
count = 0
Set.new([1, 2, 3]).each { |x| count += 1 }
count
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_each_returns_set() {
    let result = run("Set.new([1]).each { |x| x }.size");
    assert_eq!(result, Some(Object::Int(1)));
}

// Without a block the walk hands back an Enumerator over the elements.
#[test]
fn set_each_without_a_block_answers_an_enumerator() {
    let result = run("Set.new([1, 2]).each.to_a.inspect");
    assert_eq!(result, Some(Object::string("[1, 2]")));
}

#[test]
fn set_each_with_break() {
    let result = run(r#"
count = 0
Set.new([1, 2, 3, 4, 5]).each do |x|
  count += 1
  if count == 3
    break
  end
end
count
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ============================================================================
// Error paths for coverage
// ============================================================================

#[test]
fn set_each_with_exception_in_block() {
    let err = run_err(
        r#"
Set.new([1, 2]).each do |x|
  raise "error in set each"
end
"#,
    );
    assert!(err.contains("error in set each") || err.contains("exception"));
}

// An Array is an element like any other, told apart by what it holds.
#[test]
fn set_holds_arrays_as_elements() {
    let result = run("Set.new([[1, 2]]).include?([1, 2])");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_add_element_without_a_stable_rendering_errors() {
    let err = run_err("s = Set.new\ns.add(proc { 1 })");
    assert!(
        err.contains("not hashable") || err.contains("Cannot add"),
        "{}",
        err
    );
}

#[test]
fn set_remove_element_without_a_stable_rendering_errors() {
    let err = run_err("s = Set.new([1])\ns.remove(proc { 1 })");
    assert!(
        err.contains("not hashable") || err.contains("Cannot remove"),
        "{}",
        err
    );
}

#[test]
fn set_new_with_an_element_without_a_stable_rendering_errors() {
    let err = run_err("Set.new([proc { 1 }])");
    assert!(
        err.contains("not hashable") || err.contains("Cannot add"),
        "{}",
        err
    );
}

#[test]
fn set_union_error_no_args() {
    let err = run_err("Set.new([1]).union");
    assert!(err.contains("argument"));
}

#[test]
fn set_intersection_with_a_non_enumerable_errors() {
    let err = run_err("Set.new([1]).intersection(42)");
    assert!(err.contains("must be enumerable"), "{}", err);
}

#[test]
fn set_difference_with_a_non_enumerable_errors() {
    let err = run_err("Set.new([1]).difference(42)");
    assert!(err.contains("must be enumerable"), "{}", err);
}

#[test]
fn set_remove_error_no_args() {
    let err = run_err("Set.new([1]).remove");
    assert!(err.contains("argument"));
}

#[test]
fn set_contains_error_no_args() {
    let err = run_err("Set.new([1]).contains?");
    assert!(err.contains("argument"));
}

#[test]
fn set_empty_error_with_args() {
    let err = run_err("Set.new.empty?(1)");
    assert!(err.contains("argument"));
}

#[test]
fn set_to_a_error_with_args() {
    let err = run_err("Set.new.to_a(1)");
    assert!(err.contains("argument"));
}

#[test]
fn set_size_coverage() {
    let result = run("s = Set.new([1, 2, 3]); s.size");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Ordering, arbitrary elements, and the wider algebra ──────────────────────

#[test]
fn a_set_keeps_its_elements_in_the_order_they_were_added() {
    let result = run("Set.new([3, 1, 2, 1]).to_a.inspect");
    assert_eq!(result, Some(Object::string("[3, 1, 2]")));
}

#[test]
fn a_set_holds_symbols_arrays_and_nil_alongside_numbers() {
    let result = run(r#"Set.new([:a, "b", 3, [4, 5], nil]).to_a.inspect"#);
    assert_eq!(result, Some(Object::string(r#"[:a, "b", 3, [4, 5], nil]"#)));
}

#[test]
fn a_set_renders_the_way_ruby_inspects_one() {
    let result = run(r#"Set.new(["1", "2"]).inspect"#);
    assert_eq!(result, Some(Object::string(r#"Set["1", "2"]"#)));
}

#[test]
fn a_set_that_reaches_itself_renders_without_recursing() {
    let result = run(r#"
inner = Set.new
outer = Set.new([inner])
inner << outer
inner.inspect.include?("Set[...]")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn the_algebra_operators_read_the_same_as_the_named_methods() {
    let result = run(r#"
left = Set.new([1, 2, 3])
right = Set.new([3, 4])
[
  (left | right).to_a,
  (left - right).to_a,
  (left & right).to_a,
  (left ^ right).to_a
].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[[1, 2, 3, 4], [1, 2], [3], [4, 1, 2]]"))
    );
}

#[test]
fn a_set_operand_may_be_any_enumerable() {
    let result = run(r#"
[
  Set.new([1, 2, 3]).difference([3]).to_a,
  Set.new([1, 2]).union([2, 5]).to_a,
  Set.new([1, 2, 3]).intersection([2, 3, 9]).to_a
].inspect
"#);
    assert_eq!(result, Some(Object::string("[[1, 2], [1, 2, 5], [2, 3]]")));
}

#[test]
fn the_comparisons_report_containment_both_ways() {
    let result = run(r#"
left = Set.new([1, 2, 3])
[
  left.subset?(Set.new([1, 2, 3, 4])),
  left.superset?(Set.new([1, 2])),
  left.proper_subset?(Set.new([1, 2, 3])),
  left <= Set.new([1, 2, 3]),
  left < Set.new([1, 2, 3, 4]),
  left.disjoint?(Set.new([9])),
  left.intersect?(Set.new([3]))
].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string(
            "[true, true, false, true, true, true, true]"
        ))
    );
}

#[test]
fn the_in_place_filters_answer_the_set_itself() {
    let result = run(r#"
kept = Set.new([1, 2, 3, 4])
kept.keep_if { |value| value.even? }
dropped = Set.new([1, 2, 3, 4])
dropped.delete_if { |value| value.even? }
mapped = Set.new([1, 2, 3])
mapped.map! { |value| value + 10 }
[kept.to_a, dropped.to_a, mapped.to_a].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[[2, 4], [1, 3], [11, 12, 13]]"))
    );
}

#[test]
fn merge_subtract_replace_and_clear_change_the_set_in_place() {
    let result = run(r#"
set = Set.new([1, 2])
set.merge([3, 4])
first = set.to_a
set.subtract([1, 2])
second = set.to_a
set.replace([7, 8])
third = set.to_a
set.clear
[first, second, third, set.to_a, set.empty?].inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[[1, 2, 3, 4], [3, 4], [7, 8], [], true]"))
    );
}

#[test]
fn add_question_answers_nil_when_the_element_was_already_there() {
    let result = run(r#"
set = Set.new([1])
[set.add?(2).class.to_s, set.add?(2).inspect].inspect
"#);
    assert_eq!(result, Some(Object::string(r#"["Set", "nil"]"#)));
}

#[test]
fn membership_asks_the_element_for_its_hash_and_eql() {
    let result = run(r#"
class Tagged
  def hash
    42
  end
  def eql?(other)
    hash == other.hash
  end
end
holder = Set.new(["a", Tagged.new])
other = Tagged.new
[holder.include?(other), holder.member?(other), holder === other].inspect
"#);
    assert_eq!(result, Some(Object::string("[true, true, true]")));
}

#[test]
fn an_integer_and_a_float_of_the_same_value_are_different_elements() {
    let result = run("Set.new([1, 1.0]).size");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn a_set_refuses_to_change_while_a_walk_over_it_is_open() {
    let result = run(r#"
walked = Set.new([:a, :b])
refused = 0
walked.each do |_|
  begin
    walked << :c
  rescue RuntimeError
    refused += 1
  end
end
[refused, walked.to_a].inspect
"#);
    assert_eq!(result, Some(Object::string("[2, [:a, :b]]")));
}

#[test]
fn join_renders_each_element_as_its_to_s() {
    let result = run(r#"[Set.new([:a, :b, :c]).join, Set.new([:a, :b]).join("-")].inspect"#);
    assert_eq!(result, Some(Object::string(r#"["abc", "a-b"]"#)));
}

#[test]
fn a_set_compares_by_containment_with_spaceship() {
    let result = run(r#"
[
  (Set.new([1, 2]) <=> Set.new([1, 2])),
  (Set.new([1, 2]) <=> Set.new([1, 2, 3])),
  (Set.new([1, 2, 3]) <=> Set.new([1, 2])),
  (Set.new([1]) <=> Set.new([9])),
  (Set.new([1]) <=> false)
].inspect
"#);
    assert_eq!(result, Some(Object::string("[0, -1, 1, nil, nil]")));
}

#[test]
fn two_names_for_one_native_method_answer_the_same_method_object() {
    let result = run(r#"
set = Set.new
[
  set.method(:to_s) == set.method(:inspect),
  set.method(:===) == set.method(:include?),
  set.method(:size) == set.method(:length)
].inspect
"#);
    assert_eq!(result, Some(Object::string("[true, true, true]")));
}

#[test]
fn a_set_subclass_holds_its_elements_and_reports_its_own_class() {
    let result = run(r#"
class Bag < Set
end
holder = Bag.new([1, 2, 3])
[holder.class.to_s, holder.to_a.sort, holder.include?(2)].inspect
"#);
    assert_eq!(result, Some(Object::string(r#"["Bag", [1, 2, 3], true]"#)));
}
