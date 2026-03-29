mod binary_ops;
mod block_api;
mod call_and_lambda;
mod expressions;
mod method_api;
mod statements;
mod unary_ops;

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn eval(source: &str) -> Option<Object> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("Execution failed")
}
