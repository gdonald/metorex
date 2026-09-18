// Coverage tests for vm/pattern_matching.rs uncovered paths

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

/// The name of the class of the exception `code` raises.
fn raised_class(body: &str) -> Option<Object> {
    run(&format!(
        "begin\n{body}\nrescue => raised\n  raised.class.name\nend\n"
    ))
}

// ── case/in arm body with non-expression statements (lines 105-106) ──────────
// Triggers ControlFlow::Next: assignment (non-expression stmt) inside case/in arm.

#[test]
fn case_in_arm_body_with_assignment_statement() {
    let result = run(r#"
x = 42
result = 0
case x
in Integer => n
  result = n + 1
  result
end
"#);
    assert_eq!(result, Some(Object::Int(43)));
}

// ── case/in arm body with return-producing control flow (lines 107-110) ──────
// Triggers `flow => { return Ok(flow) }` branch when return exits the method.

#[test]
fn case_in_arm_body_with_return_exits_method() {
    let result = run(r#"
def classify(n)
  case n
  in Integer => x
    return x * 2
    x
  end
end
classify 5
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── match arm body with non-expression statements (execute_match lines 52-58) ─

#[test]
fn match_arm_body_with_assignment_statement() {
    let result = run(r#"
x = 42
result = 0
matcher = ->(number) { number == 42 }
case x
when matcher
  result = x * 2
  result
end
"#);
    assert_eq!(result, Some(Object::Int(84)));
}

#[test]
fn match_arm_body_with_return_exits_method() {
    let result = run(r#"
def doubled(n)
  case n
  when Integer
    return n * 2
    n
  end
end
doubled 7
"#);
    assert_eq!(result, Some(Object::Int(14)));
}

// ── case/in no match raises error ─────────────────────────────────────────────

#[test]
fn case_in_no_match_raises_error() {
    let raised = raised_class(
        r#"
case 42
in String => s
  s
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternError")));
}

// ── Range pattern with non-numeric value (pattern_matching.rs line 273) ───────

#[test]
fn range_pattern_non_numeric_value_no_match() {
    let raised = raised_class(
        r#"
case "hello"
in 1..10 => n
  n
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternError")));
}

// ── Array pattern without rest - length mismatch (line 365) ───────────────────

#[test]
fn array_pattern_length_mismatch_no_match() {
    let raised = raised_class(
        r#"
case [1, 2]
in [1, 2, 3] => arr
  arr
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternError")));
}

// ── Array pattern without rest - element mismatch (line 371) ──────────────────

#[test]
fn array_pattern_element_mismatch_no_match() {
    let raised = raised_class(
        r#"
case [1, 2]
in [1, 3] => arr
  arr
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternError")));
}

// ── Array pattern with rest - element before rest mismatches (line 339) ────────

#[test]
fn array_pattern_rest_before_mismatch() {
    let raised = raised_class(
        r#"
case [1, 2, 3]
in [10, *rest]
  rest
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternError")));
}

// ── Array pattern with rest - element after rest mismatches (line 348) ─────────

#[test]
fn array_pattern_rest_after_mismatch() {
    let raised = raised_class(
        r#"
case [1, 2, 3]
in [*rest, 99]
  rest
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternError")));
}

// ── Object pattern - key missing from dict (line 395) ─────────────────────────

#[test]
fn object_pattern_missing_key_no_match() {
    // One `in` clause and no `else`, so the missing key is named by the more
    // particular error.
    let raised = raised_class(
        r#"
d = {"x" => 1}
case d
in {y: Integer}
  "matched"
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternKeyError")));
}

// ── Object pattern - value doesn't match pattern (line 392) ───────────────────

#[test]
fn object_pattern_value_mismatch_no_match() {
    let raised = raised_class(
        r#"
d = {x: 1}
case d
in {x: String}
  "matched"
end
"#,
    );
    assert_eq!(raised, Some(Object::string("NoMatchingPatternError")));
}

// ── A bare splat pattern takes the whole array ───────────────────────────────

#[test]
fn bare_splat_pattern_binds_every_element() {
    let result = run("case [1, 2, 3]\nin *rest\n  rest.length\nend\n");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Range pattern matching (lines 280, 287) ─────────────────────────────────

#[test]
fn range_pattern_float_end() {
    let result = run("case 2.5\nin 1.0..3.0\n  \"yes\"\nend");
    assert_eq!(result, Some(Object::string("yes".to_string())));
}

#[test]
fn range_pattern_exclusive() {
    let result = run("case 5\nin 1...5\n  \"match\"\nelse\n  \"no\"\nend");
    assert_eq!(result, Some(Object::string("no".to_string())));
}

#[test]
fn range_pattern_non_numeric() {
    let result = run("case \"abc\"\nin 1..10\n  \"match\"\nelse\n  \"no\"\nend");
    assert_eq!(result, Some(Object::string("no".to_string())));
}

#[test]
fn range_pattern_exclusive_float() {
    let result = run("case 3.5\nin 1.0...5.0\n  \"yes\"\nelse\n  \"no\"\nend");
    assert_eq!(result, Some(Object::string("yes".to_string())));
}

// ── control_flow.rs lines 768-769: :@ivar and :@@cvar symbol patterns ────────

#[test]
fn pattern_match_ivar_symbol() {
    let result = run(r#"
x = :@hello
case x
in :@hello
  "matched"
else
  "no"
end
"#);
    assert_eq!(result, Some(Object::string("matched".to_string())));
}

#[test]
fn pattern_match_cvar_symbol() {
    let result = run(r#"
x = :@@count
case x
in :@@count
  "matched"
else
  "no"
end
"#);
    assert_eq!(result, Some(Object::string("matched".to_string())));
}

// ── pattern_matching.rs line 153: SymbolLiteral pattern with non-symbol ──────

#[test]
fn symbol_pattern_against_non_symbol_no_match() {
    let result = run(r#"
case 42
when :foo
  "matched"
else
  "no"
end
"#);
    assert_eq!(result, Some(Object::string("no".to_string())));
}

// ── pattern_matching.rs lines 276, 283: range pattern with non-numeric bounds ─

#[test]
fn range_pattern_with_string_bounds_no_match_numeric_value() {
    // Integer value matched against string-bounded range → no match (returns else)
    let result = run(r#"
case 5
in "a".."z"
  "matched"
else
  "no"
end
"#);
    assert_eq!(result, Some(Object::string("no".to_string())));
}

// ── From vm/additional_tests ────────────────────────────────────────────────

// ── What a single `in` clause reports when it does not match ─────────────────
// A `case` holding one clause and no `else` has nowhere to go, so Ruby names
// what the pattern found wrong rather than the value alone.

fn raised_message(body: &str) -> Option<Object> {
    run(&format!(
        "begin\n{body}\nrescue => raised\n  raised.message\nend\n"
    ))
}

#[test]
fn single_clause_missing_key_raises_no_matching_pattern_key_error() {
    let raised = raised_class("case({x: 1})\nin {y: Integer}\nend\n");
    assert_eq!(raised, Some(Object::string("NoMatchingPatternKeyError")));
}

#[test]
fn missing_key_error_names_the_hash_and_the_key() {
    let raised = raised_message("case({x: 1})\nin {y: Integer}\nend\n");
    assert_eq!(raised, Some(Object::string("{x: 1}: key not found: :y")));
}

#[test]
fn missing_key_error_answers_the_key_it_looked_for() {
    let result = run("begin\n  {x: 1} => {y: Integer}\nrescue => raised\n  raised.key\nend\n");
    assert_eq!(result, Some(Object::symbol("y".to_string())));
}

#[test]
fn missing_key_error_answers_the_hash_it_looked_in() {
    // The matchee is the hash the key was missing from, which is the one
    // nested inside rather than the value the `case` was given.
    let result = run(
        "begin\n  case({a: {x: 1}})\n  in {a: {y: 1}}\n  end\nrescue => raised\n  raised.matchee.keys\nend\n",
    );
    assert_eq!(
        result,
        Some(Object::array(vec![Object::symbol("x".to_string())]))
    );
}

#[test]
fn missing_key_error_is_caught_as_a_no_matching_pattern_error() {
    let result = run(
        "begin\n  case({x: 1})\n  in {y: 1}\n  end\nrescue NoMatchingPatternError\n  \"caught\"\nend\n",
    );
    assert_eq!(result, Some(Object::string("caught")));
}

#[test]
fn more_than_one_clause_names_the_value_alone() {
    let raised = raised_message("case({x: 1})\nin {y: 1}\nin {z: 1}\nend\n");
    assert_eq!(raised, Some(Object::string("{x: 1}")));
}

#[test]
fn an_else_clause_leaves_the_value_unreported() {
    let result = run("case({x: 1})\nin {y: 1}\n  \"matched\"\nelse\n  \"else\"\nend\n");
    assert_eq!(result, Some(Object::string("else")));
}

#[test]
fn single_clause_names_a_value_with_nothing_to_deconstruct() {
    let raised = raised_message("case 5\nin [1, 2]\nend\n");
    assert_eq!(
        raised,
        Some(Object::string("5: 5 does not respond to #deconstruct"))
    );
}

#[test]
fn single_clause_names_a_value_with_no_keys_to_read() {
    let raised = raised_message("case 5\nin {y: 1}\nend\n");
    assert_eq!(
        raised,
        Some(Object::string("5: 5 does not respond to #deconstruct_keys"))
    );
}

#[test]
fn single_clause_names_a_length_that_does_not_fit() {
    let raised = raised_message("case [1, 2]\nin [1, 2, 3]\nend\n");
    assert_eq!(
        raised,
        Some(Object::string(
            "[1, 2]: [1, 2] length mismatch (given 2, expected 3)"
        ))
    );
}

#[test]
fn single_clause_names_the_fewest_elements_a_splat_can_take() {
    let raised = raised_message("case [1]\nin [1, 2, *rest]\nend\n");
    assert_eq!(
        raised,
        Some(Object::string(
            "[1]: [1] length mismatch (given 1, expected 2+)"
        ))
    );
}

#[test]
fn single_clause_names_the_element_that_did_not_match() {
    let raised = raised_message("case [1, 2]\nin [1, 3]\nend\n");
    assert_eq!(
        raised,
        Some(Object::string("[1, 2]: 3 === 2 does not return true"))
    );
}

#[test]
fn single_clause_names_the_last_choice_of_an_alternative() {
    let raised = raised_message("case 5\nin String | Symbol\nend\n");
    assert_eq!(
        raised,
        Some(Object::string("5: Symbol === 5 does not return true"))
    );
}

#[test]
fn single_clause_names_a_find_pattern_that_sits_nowhere() {
    let raised = raised_message("case [1, 2]\nin [*, 7, *]\nend\n");
    assert_eq!(
        raised,
        Some(Object::string(
            "[1, 2]: [1, 2] does not match to find pattern"
        ))
    );
}

#[test]
fn single_clause_names_a_guard_that_refused() {
    let raised = raised_message("case 5\nin Integer if false\nend\n");
    assert_eq!(
        raised,
        Some(Object::string("5: guard clause does not return true"))
    );
}

#[test]
fn single_clause_names_the_keys_a_refused_rest_left_over() {
    let raised = raised_message("case({x: 1, y: 2})\nin {x: 1, **nil}\nend\n");
    assert_eq!(
        raised,
        Some(Object::string("{x: 1, y: 2}: rest of {y: 2} is not empty"))
    );
}

#[test]
fn rightward_assignment_names_what_the_pattern_found_wrong() {
    let raised = raised_message("5 => 6\n");
    assert_eq!(
        raised,
        Some(Object::string("5: 6 === 5 does not return true"))
    );
}

#[test]
fn the_in_test_answers_false_rather_than_raising() {
    let result = run("answer = ({x: 1} in {y: 1})\nanswer\n");
    assert_eq!(result, Some(Object::Bool(false)));
}
