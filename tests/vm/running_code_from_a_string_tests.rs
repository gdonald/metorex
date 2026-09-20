// What `eval` does with the scope around it: where a `return` in the code
// lands, and where a class it opens belongs.

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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements)
        .expect_err("the code should have been refused")
        .to_string()
}

#[test]
fn a_return_in_evald_code_returns_from_the_method_around_it() {
    let result = run(r#"
def doubled(n)
  eval "return n * 2"
  :never_reached
end
doubled 21
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn a_return_in_a_begin_block_returns_from_the_method_around_it() {
    let result = run(r#"
def tripled(n)
  eval "BEGIN { return n * 3 }"
  :never_reached
end
tripled 4
"#);
    assert_eq!(result, Some(Object::Int(12)));
}

#[test]
fn a_return_in_evald_code_returns_from_the_lambda_around_it() {
    let result = run(r#"
-> do
  proc do
    eval "return :from_the_eval"
  end.call
  :never_reached
end.call
"#);
    assert_eq!(result, Some(Object::symbol("from_the_eval".to_string())));
}

#[test]
fn a_return_with_no_lambda_or_method_around_it_is_refused() {
    let message = run_err(r#"proc { eval "return :nowhere" }.call"#);
    assert!(message.contains("unexpected return"), "Got: {}", message);
}

#[test]
fn a_return_written_at_the_top_level_of_the_code_ends_the_program() {
    let result = run(r#"eval "return :done""#);
    assert_eq!(result, Some(Object::symbol("done".to_string())));
}

#[test]
fn a_class_opened_in_evald_code_belongs_to_the_scope_the_eval_was_written_in() {
    let result = run(r#"
class Cabinet
  def stock
    eval "class Shelf; end"
    Shelf.name
  end
end
Cabinet.new.stock
"#);
    assert_eq!(result, Some(Object::string("Cabinet::Shelf")));
}
