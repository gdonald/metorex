// Where `redo` restarts a body, and where the parser refuses it.

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

fn ints(values: &[i64]) -> Option<Object> {
    Some(Object::array(
        values.iter().map(|value| Object::Int(*value)).collect(),
    ))
}

fn refuses(code: &str) -> bool {
    let tokens = Lexer::new(code).tokenize();
    match Parser::new(tokens).parse() {
        Ok(_) => false,
        Err(errors) => errors
            .iter()
            .any(|error| error.to_string().contains("Invalid redo")),
    }
}

/// A block body whose `redo` sits inside a `begin` with an ensure clause.
const REDO_THROUGH_ENSURE: &str =
    "  begin\n    list << 1\n    redo if list.size == 1\n  ensure\n    list << 0\n  end";

#[test]
fn redo_through_an_ensure_clause_restarts_a_block_given_to_each() {
    let code = format!(
        "list = []\n[7].each do |x|\n{}\nend\nlist",
        REDO_THROUGH_ENSURE
    );
    assert_eq!(run(&code), ints(&[1, 0, 1, 0]));
}

#[test]
fn redo_through_an_ensure_clause_restarts_a_block_given_to_a_yielding_method() {
    let code = format!(
        "def yields_once\n  yield\nend\nlist = []\nyields_once do\n{}\nend\nlist",
        REDO_THROUGH_ENSURE
    );
    assert_eq!(run(&code), ints(&[1, 0, 1, 0]));
}

#[test]
fn redo_through_an_ensure_clause_restarts_a_lambda() {
    let code = format!("list = []\n-> {{\n{}\n}}.call\nlist", REDO_THROUGH_ENSURE);
    assert_eq!(run(&code), ints(&[1, 0, 1, 0]));
}

#[test]
fn redo_as_the_value_of_a_begin_restarts_the_block() {
    let code = "list = []\n[7].each do |x|\n  list << x\n  value = begin\n    redo if list.size == 1\n  ensure\n    list << 0\n  end\nend\nlist";
    assert_eq!(run(code), ints(&[7, 0, 7, 0]));
}

#[test]
fn redo_in_a_method_body_is_refused() {
    assert!(refuses("def restarts\n  redo\nend"));
}

#[test]
fn redo_in_a_class_body_is_refused() {
    assert!(refuses("class Restarts\n  redo\nend"));
}

#[test]
fn redo_at_the_top_level_is_refused() {
    assert!(refuses("redo"));
}

#[test]
fn redo_inside_a_loop_or_a_block_is_accepted() {
    assert!(!refuses("while false\n  redo\nend"));
    assert!(!refuses("def restarts\n  [1].each { redo }\nend"));
}

#[test]
fn redo_named_by_defined_is_an_expression() {
    assert_eq!(run("defined?(redo)"), Some(Object::string("expression")));
}

#[test]
fn redo_in_a_post_test_loop_inside_a_method_restarts_the_body() {
    let code = "def counts\n  turns = 0\n  begin\n    turns += 1\n    redo if turns < 3\n  end while false\n  turns\nend\ncounts";
    assert_eq!(run(code), Some(Object::Int(3)));
}

#[test]
fn redo_with_a_while_modifier_is_accepted() {
    assert!(!refuses("redo while false"));
    assert!(!refuses("begin\n  redo\nend until true"));
}

#[test]
fn redo_in_a_method_defined_inside_a_post_test_loop_is_refused() {
    assert!(refuses(
        "begin\n  def restarts\n    redo\n  end\nend while false"
    ));
}

#[test]
fn redo_in_a_module_body_or_an_endless_method_is_refused() {
    assert!(refuses("module Restarts\n  redo\nend"));
    assert!(refuses("def restarts = redo"));
}

#[test]
fn a_refused_redo_leaves_later_statements_to_parse() {
    let tokens = Lexer::new("redo\nvalue = 1").tokenize();
    let errors = Parser::new(tokens)
        .parse()
        .expect_err("the redo is refused");
    assert_eq!(errors.len(), 1);
}
