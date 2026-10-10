// VM tests for operators: regex =~ / !~, unless, unary +, format %, Regexp type name.

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

// ── =~ and !~ operators ─────────────────────────────────────────────────────

#[test]
fn regex_match_operator() {
    let result = run("'hello' =~ /ell/");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn regex_match_no_match() {
    let result = run("'hello' =~ /xyz/");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn regex_not_match_operator() {
    let result = run("'hello' !~ /xyz/");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn regex_not_match_does_match() {
    let result = run("'hello' !~ /ell/");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── unless expression ────────────────────────────────────────────────────────

#[test]
fn unless_expression_falsy_branch() {
    let result = run(r#"
x = unless true
  1
else
  2
end
x
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn unless_expression_truthy_branch() {
    let result = run(r#"
x = unless false
  1
else
  2
end
x
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── unary + on non-numeric value ──────────────────────────────────────────────

#[test]
fn unary_plus_on_non_numeric_errors() {
    let err = run_err("+true");
    assert!(err.contains("undefined method '+@' for true"));
}

// ── operators.rs: %d format ───────────────────────────────────────────────────

#[test]
fn format_d_with_float_uses_display() {
    let result = run(r#""%d" % 3.14"#);
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn format_d_refuses_a_string_that_names_no_number() {
    let message = run_err(r#""%d" % "hello""#);
    assert!(message.contains("invalid value for Integer()"));
}

// ── operators.rs: %c format ───────────────────────────────────────────────────

#[test]
fn format_c_refuses_a_code_past_the_last_character() {
    let message = run_err(r#""%c" % 1114112"#);
    assert!(message.contains("out of char range"));
}

#[test]
fn format_c_with_float_uses_display() {
    let result = run(r#""%c" % 3.14"#);
    assert!(matches!(result, Some(Object::String(_))));
}

// ── Regexp type name in error message ─────────────────────────────────────────

#[test]
fn regex_type_name_in_error_message() {
    let err = run_err(r#"/hello/ + 1"#);
    assert!(
        err.contains("Regexp") || err.contains("type") || err.contains("Cannot"),
        "Error was: {}",
        err
    );
}
