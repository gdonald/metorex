// Targeted coverage for the object methods, grouped by what each one
// reaches.
//
// `run` evaluates a program and answers its last value. `run_err`
// evaluates one that is expected to fail and answers its message.

pub(crate) use metorex::lexer::Lexer;
pub(crate) use metorex::object::Object;
pub(crate) use metorex::parser::Parser;
pub(crate) use metorex::vm::VirtualMachine;

pub(crate) fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

pub(crate) fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

mod copying;
mod identity;
mod methods;
mod variables;
