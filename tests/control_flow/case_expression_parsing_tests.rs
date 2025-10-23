use metorex::ast::node::MatchPattern;
use metorex::ast::{Expression, Statement};
use metorex::lexer::Lexer;
use metorex::parser::Parser;

/// Helper to parse a case expression from source
/// Wraps the case in an assignment to force expression context
fn parse_case_expr(source: &str) -> Expression {
    // Wrap in assignment to force expression parsing
    let wrapped = format!("x = {}", source.trim());
    let lexer = Lexer::new(&wrapped);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);

    let program = parser.parse().expect("Failed to parse");
    assert_eq!(program.len(), 1, "Expected single statement");

    match &program[0] {
        Statement::Assignment { value, .. } => value.clone(),
        other => panic!("Expected assignment statement, got {:?}", other),
    }
}

#[test]
fn test_parse_case_expression_basic() {
    let source = r#"
case x
when 1
  "one"
when 2
  "two"
else
  "other"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse basic case expression");
}

#[test]
fn test_parse_case_expression_structure() {
    let source = r#"
case val
when 1
  "one"
when 2
  "two"
else
  "other"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case {
            expression,
            cases,
            else_case,
            ..
        } => {
            // Check match expression
            assert!(
                matches!(expression.as_ref(), Expression::Identifier { name, .. } if name == "val")
            );

            // Check cases
            assert_eq!(cases.len(), 2);
            assert!(matches!(cases[0].pattern, MatchPattern::IntLiteral(1)));
            assert!(matches!(cases[1].pattern, MatchPattern::IntLiteral(2)));

            // Check bodies are string literals
            assert!(matches!(cases[0].body, Expression::StringLiteral { .. }));
            assert!(matches!(cases[1].body, Expression::StringLiteral { .. }));

            // Check else
            assert!(else_case.is_some());
            assert!(matches!(
                else_case.unwrap().as_ref(),
                Expression::StringLiteral { .. }
            ));
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_without_else() {
    let source = r#"
case x
when 1
  "one"
when 2
  "two"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case {
            else_case, cases, ..
        } => {
            assert_eq!(cases.len(), 2);
            assert!(else_case.is_none());
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_guard() {
    let source = r#"
case val
when x if x > 0
  "positive"
when x if x < 0
  "negative"
else
  "zero"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 2);

            // Both cases should have guards
            assert!(cases[0].guard.is_some());
            assert!(cases[1].guard.is_some());

            // Guards should be binary operations
            assert!(matches!(
                cases[0].guard.as_ref().unwrap(),
                Expression::BinaryOp { .. }
            ));
            assert!(matches!(
                cases[1].guard.as_ref().unwrap(),
                Expression::BinaryOp { .. }
            ));
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_array_pattern() {
    let source = r#"
case arr
when [a, b]
  a + b
when [x]
  x
else
  0
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 2);

            // First case should have array pattern with 2 elements
            match &cases[0].pattern {
                MatchPattern::Array(patterns) => {
                    assert_eq!(patterns.len(), 2);
                    assert!(matches!(patterns[0], MatchPattern::Identifier(_)));
                    assert!(matches!(patterns[1], MatchPattern::Identifier(_)));
                }
                _ => panic!("Expected Array pattern"),
            }

            // Second case should have array pattern with 1 element
            match &cases[1].pattern {
                MatchPattern::Array(patterns) => {
                    assert_eq!(patterns.len(), 1);
                }
                _ => panic!("Expected Array pattern"),
            }
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_object_pattern() {
    let source = r#"
case point
when {x: px, y: py}
  px + py
else
  0
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 1);

            // Should have object pattern
            match &cases[0].pattern {
                MatchPattern::Object(pairs) => {
                    assert_eq!(pairs.len(), 2);
                    assert_eq!(pairs[0].0, "x");
                    assert_eq!(pairs[1].0, "y");
                }
                _ => panic!("Expected Object pattern"),
            }
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_rest_pattern() {
    let source = r#"
case arr
when [first, ...rest]
  first
else
  nil
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 1);

            // Should have array pattern with rest
            match &cases[0].pattern {
                MatchPattern::Array(patterns) => {
                    assert_eq!(patterns.len(), 2);
                    assert!(matches!(patterns[0], MatchPattern::Identifier(_)));
                    assert!(matches!(patterns[1], MatchPattern::Rest(_)));
                }
                _ => panic!("Expected Array pattern"),
            }
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_wildcard() {
    let source = r#"
case val
when 1
  "one"
when _
  "other"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(matches!(cases[1].pattern, MatchPattern::Wildcard));
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_type_pattern() {
    let source = r#"
case obj
when Integer
  "int"
when String
  "str"
else
  "other"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 2);

            match &cases[0].pattern {
                MatchPattern::Type(name) => assert_eq!(name, "Integer"),
                _ => panic!("Expected Type pattern"),
            }

            match &cases[1].pattern {
                MatchPattern::Type(name) => assert_eq!(name, "String"),
                _ => panic!("Expected Type pattern"),
            }
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_in_assignment() {
    let source = r#"
result = case x
when 1
  "one"
when 2
  "two"
else
  "other"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Failed to parse");

    assert_eq!(program.len(), 1);
    match &program[0] {
        Statement::Assignment { value, .. } => {
            assert!(matches!(value, Expression::Case { .. }));
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_parse_nested_case_expressions() {
    let source = r#"
case x
when 1
  case y
  when 2
    "one-two"
  else
    "one-other"
  end
else
  "other"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 1);

            // The body of the first case should be another case expression
            assert!(matches!(cases[0].body, Expression::Case { .. }));
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_single_when() {
    let source = r#"
case x
when 1
  "one"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case {
            cases, else_case, ..
        } => {
            assert_eq!(cases.len(), 1);
            assert!(else_case.is_none());
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_multiple_when_clauses() {
    let source = r#"
case val
when 1
  "one"
when 2
  "two"
when 3
  "three"
when 4
  "four"
else
  "other"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case {
            cases, else_case, ..
        } => {
            assert_eq!(cases.len(), 4);
            assert!(else_case.is_some());
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_boolean_patterns() {
    let source = r#"
case flag
when true
  "yes"
when false
  "no"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(matches!(cases[0].pattern, MatchPattern::BoolLiteral(true)));
            assert!(matches!(cases[1].pattern, MatchPattern::BoolLiteral(false)));
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_with_nil_pattern() {
    let source = r#"
case val
when nil
  "nothing"
else
  "something"
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 1);
            assert!(matches!(cases[0].pattern, MatchPattern::NilLiteral));
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_parse_case_expression_complex_body() {
    let source = r#"
case x
when 1
  a + b * c
when 2
  foo.bar(baz)
else
  [1, 2, 3]
end
"#;

    let expr = parse_case_expr(source);

    match expr {
        Expression::Case {
            cases, else_case, ..
        } => {
            assert_eq!(cases.len(), 2);

            // First body should be binary op
            assert!(matches!(cases[0].body, Expression::BinaryOp { .. }));

            // Second body should be method call
            assert!(matches!(cases[1].body, Expression::MethodCall { .. }));

            // Else should be array
            assert!(matches!(
                else_case.unwrap().as_ref(),
                Expression::Array { .. }
            ));
        }
        _ => panic!("Expected Expression::Case"),
    }
}
