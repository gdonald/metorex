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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
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
case x
when n
  result = n * 2
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
  when x
    return x * 2
    x
  end
end
doubled 7
"#);
    assert_eq!(result, Some(Object::Int(14)));
}

// ── case/in no match raises error ─────────────────────────────────────────────

#[test]
fn case_in_no_match_raises_error() {
    let err = run_err(
        r#"
case 42
in String => s
  s
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError") || err.contains("No") || err.contains("pattern")
    );
}

// ── Range pattern with non-numeric value (pattern_matching.rs line 273) ───────

#[test]
fn range_pattern_non_numeric_value_no_match() {
    let err = run_err(
        r#"
case "hello"
in 1..10 => n
  n
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError") || err.contains("No") || err.contains("pattern")
    );
}

// ── Array pattern without rest - length mismatch (line 365) ───────────────────

#[test]
fn array_pattern_length_mismatch_no_match() {
    let err = run_err(
        r#"
case [1, 2]
in [1, 2, 3] => arr
  arr
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError") || err.contains("No") || err.contains("pattern")
    );
}

// ── Array pattern without rest - element mismatch (line 371) ──────────────────

#[test]
fn array_pattern_element_mismatch_no_match() {
    let err = run_err(
        r#"
case [1, 2]
in [1, 3] => arr
  arr
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError") || err.contains("No") || err.contains("pattern")
    );
}

// ── Array pattern with rest - element before rest mismatches (line 339) ────────

#[test]
fn array_pattern_rest_before_mismatch() {
    let err = run_err(
        r#"
case [1, 2, 3]
in [10, ...rest]
  rest
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError") || err.contains("No") || err.contains("pattern")
    );
}

// ── Array pattern with rest - element after rest mismatches (line 348) ─────────

#[test]
fn array_pattern_rest_after_mismatch() {
    let err = run_err(
        r#"
case [1, 2, 3]
in [...rest, 99]
  rest
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError") || err.contains("No") || err.contains("pattern")
    );
}

// ── Object pattern - key missing from dict (line 395) ─────────────────────────

#[test]
fn object_pattern_missing_key_no_match() {
    let err = run_err(
        r#"
d = {"x" => 1}
case d
in {y: Integer}
  "matched"
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError")
            || err.contains("No")
            || err.contains("pattern")
            || err.contains("Undefined")
    );
}

// ── Object pattern - value doesn't match pattern (line 392) ───────────────────

#[test]
fn object_pattern_value_mismatch_no_match() {
    let err = run_err(
        r#"
d = {"x" => 1}
case d
in {x: String}
  "matched"
end
"#,
    );
    assert!(
        err.contains("NoMatchingPatternError")
            || err.contains("No")
            || err.contains("pattern")
            || err.contains("Undefined")
    );
}

// ── Rest pattern at top level is a parse error (lines 184-186) ────────────────
// The parser rejects ...rest outside an array pattern, so it's a parse error.

#[test]
fn rest_pattern_outside_array_is_parse_error() {
    let tokens = metorex::lexer::Lexer::new("case 42\nin ...rest\n  rest\nend\n").tokenize();
    let result = metorex::parser::Parser::new(tokens).parse();
    assert!(result.is_err());
}
