// Coverage tests for vm/pattern_matching.rs uncovered paths and
// parser/statements/control_flow.rs uncovered paths

use metorex::ast::{Expression, MatchCase, MatchPattern, Statement};
use metorex::lexer::{Lexer, Position};
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn pos() -> Position {
    Position::new(1, 1, 0)
}

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

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().unwrap_err()[0].to_string()
}

// ── FloatLiteral pattern non-match (line 153 in pattern_matching.rs) ──────────

#[test]
fn float_pattern_does_not_match_int_value() {
    let mut vm = VirtualMachine::new();
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 1,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::FloatLiteral(1.5),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "matched".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "no match".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };
    let result = vm.execute_program(&[case_stmt]).expect("execution failed");
    assert_eq!(result, Some(Object::string("no match")));
}

// ── StringLiteral pattern non-match (line 157 in pattern_matching.rs) ─────────

#[test]
fn string_pattern_does_not_match_int_value() {
    let mut vm = VirtualMachine::new();
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::StringLiteral("hello".to_string()),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "matched".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "no match".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };
    let result = vm.execute_program(&[case_stmt]).expect("execution failed");
    assert_eq!(result, Some(Object::string("no match")));
}

// ── BoolLiteral pattern non-match (line 161 in pattern_matching.rs) ───────────

#[test]
fn bool_pattern_does_not_match_int_value() {
    let mut vm = VirtualMachine::new();
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 1,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::BoolLiteral(true),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "matched".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "no match".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };
    let result = vm.execute_program(&[case_stmt]).expect("execution failed");
    assert_eq!(result, Some(Object::string("no match")));
}

// ── Array pattern on non-array value (line 180) ───────────────────────────────

#[test]
fn array_pattern_does_not_match_int_value() {
    let mut vm = VirtualMachine::new();
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::Array(vec![MatchPattern::Wildcard]),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "array matched".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "no match".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };
    let result = vm.execute_program(&[case_stmt]).expect("execution failed");
    assert_eq!(result, Some(Object::string("no match")));
}

// ── Object pattern on non-dict value (line 195) ──────────────────────────────

#[test]
fn object_pattern_does_not_match_int_value() {
    let mut vm = VirtualMachine::new();
    let case_stmt = Statement::Match {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(),
        },
        cases: vec![
            MatchCase {
                pattern: MatchPattern::Object(vec![("x".to_string(), MatchPattern::Wildcard)]),
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "object matched".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![Statement::Expression {
                    expression: Expression::StringLiteral {
                        value: "no match".to_string(),
                        position: pos(),
                    },
                    position: pos(),
                }],
                position: pos(),
            },
        ],
        position: pos(),
    };
    let result = vm.execute_program(&[case_stmt]).expect("execution failed");
    assert_eq!(result, Some(Object::string("no match")));
}

// ── Range pattern with float value (lines 272-287) ───────────────────────────

#[test]
fn range_pattern_float_value_matches() {
    let result = run(r#"
case 2.5
when 1..5
  "in range"
else
  "out of range"
end
"#);
    assert_eq!(result, Some(Object::string("in range")));
}

#[test]
fn range_pattern_float_bounds_matches() {
    let result = run(r#"
case 3
when 1.0..5.0
  "in float range"
else
  "out"
end
"#);
    assert_eq!(result, Some(Object::string("in float range")));
}

// ── Multiple rest patterns error (lines 316-318) ──────────────────────────────

#[test]
fn multiple_rest_patterns_in_array_error() {
    let mut vm = VirtualMachine::new();
    let case_stmt = Statement::Match {
        expression: Expression::Array {
            elements: vec![
                Expression::IntLiteral {
                    value: 1,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 2,
                    position: pos(),
                },
                Expression::IntLiteral {
                    value: 3,
                    position: pos(),
                },
            ],
            position: pos(),
        },
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Rest("first".to_string()),
                MatchPattern::Rest("second".to_string()),
            ]),
            guard: None,
            body: vec![],
            position: pos(),
        }],
        position: pos(),
    };
    let result = vm.execute_program(&[case_stmt]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("rest") || err.contains("Only one") || err.contains("pattern"));
}

// ── Array too short for rest pattern (line 333) ───────────────────────────────

#[test]
fn array_too_short_for_rest_pattern() {
    let result = run(r#"
case [1]
when [first, second, ...rest]
  "matched"
else
  "too short"
end
"#);
    assert_eq!(result, Some(Object::string("too short")));
}

// ── case/in no matching pattern error ─────────────────────────────────────────

#[test]
fn case_in_no_matching_pattern_error() {
    let err = run_err(
        r#"
case 42
in "hello"
  "matched string"
in true
  "matched bool"
end
"#,
    );
    assert!(
        err.contains("NoMatchingPattern")
            || err.contains("pattern")
            || err.contains("matched")
            || err.contains("42")
    );
}

// ── case/when with guard clause ───────────────────────────────────────────────

#[test]
fn case_when_with_guard_clause_matches() {
    let result = run(r#"
x = 5
case x
when 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 if x > 3
  "big"
else
  "small"
end
"#);
    assert_eq!(result, Some(Object::string("big")));
}

#[test]
fn case_when_guard_fails_falls_through() {
    let result = run(r#"
x = 2
case x
when 1, 2, 3 if x > 5
  "big"
else
  "small"
end
"#);
    assert_eq!(result, Some(Object::string("small")));
}

// ── case/in with else clause ──────────────────────────────────────────────────

#[test]
fn case_in_with_else_clause() {
    let result = run(r#"
case 99
in 1
  "one"
in 2
  "two"
else
  "other"
end
"#);
    assert_eq!(result, Some(Object::string("other")));
}

// ── case/in with guard clause ─────────────────────────────────────────────────

#[test]
fn case_in_with_guard_clause_matches() {
    let result = run(r#"
x = 5
case x
in n if n > 3
  "big"
end
"#);
    assert_eq!(result, Some(Object::string("big")));
}

// ── case/in with non-matching guard falls to next ────────────────────────────

#[test]
fn case_in_guard_fails_next_arm_matches() {
    let result = run(r#"
x = 2
case x
in n if n > 5
  "huge"
in n
  "normal"
end
"#);
    assert_eq!(result, Some(Object::string("normal")));
}

// ── ... without name error in case/in array pattern ──────────────────────────

#[test]
fn case_in_array_rest_without_name_error() {
    let err = parse_err(
        r#"
case [1, 2, 3]
in [first, ...]
  first
end
"#,
    );
    assert!(err.contains("identifier") || err.contains("Expected") || err.contains("..."));
}

// ── ... without name error in case/when array pattern ────────────────────────

#[test]
fn case_when_array_rest_without_name_error() {
    let err = parse_err(
        r#"
case [1, 2, 3]
when [first, ...]
  first
end
"#,
    );
    assert!(err.contains("identifier") || err.contains("Expected") || err.contains("..."));
}

// ── case/in ControlFlow propagation (match body generates non-Next flow) ──────

#[test]
fn case_in_body_control_flow_propagates() {
    // A case/in statement where the body returns a value (ControlFlow::Return)
    // This propagates through execute_case_in → execute_program
    let result = run(r#"
x = 10
case x
in n if n > 5
  n * 2
end
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

// ── Match ControlFlow propagation (lines 54-56 in pattern_matching.rs) ────────

#[test]
fn match_body_with_break_generates_error() {
    let err = run_err(
        r#"
case 1
when 1
  break
end
"#,
    );
    assert!(err.contains("break") || err.contains("loop") || err.contains("control"));
}

// ── Object pattern with string key (control_flow.rs lines 643-646) ───────────

#[test]
fn case_in_object_pattern_string_key_matches() {
    let result = run(r#"
h = {"key" => 42}
case h
in {"key": v}
  v
else
  0
end
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn case_in_object_pattern_string_key_no_match() {
    let result = run(r#"
h = {"other" => 1}
case h
in {"key": v}
  v
else
  0
end
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

// ── Object pattern shorthand {name} (control_flow.rs line 666) ───────────────

#[test]
fn case_in_object_pattern_shorthand_matches() {
    let result = run(r#"
h = {"name" => "Alice"}
case h
in {name}
  name
else
  "no"
end
"#);
    assert_eq!(result, Some(Object::string("Alice")));
}

// ── Object pattern key error (control_flow.rs lines 648-655) ─────────────────

#[test]
fn case_in_object_pattern_numeric_key_parse_error() {
    let err = parse_err(
        r#"
case 1
in {42: v}
  v
end
"#,
    );
    assert!(
        err.contains("identifier")
            || err.contains("string")
            || err.contains("key")
            || err.contains("Expected")
    );
}
