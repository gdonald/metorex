// Lambda expression tests

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

#[test]
fn test_lambda_do_end_no_params() {
    let source = r#"
l = lambda do
  42
end
l.call
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 42);
    } else {
        panic!("Expected Int(42), got {:?}", result);
    }
}

#[test]
fn test_lambda_do_end_with_params() {
    let source = r#"
l = lambda do |x|
  x * 2
end
l.call(5)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 10);
    } else {
        panic!("Expected Int(10), got {:?}", result);
    }
}

#[test]
fn test_lambda_with_closure() {
    let source = r#"
def make_multiplier(factor)
  lambda do |value|
    factor * value
  end
end

double = make_multiplier(2)
double.call(5)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 10);
    } else {
        panic!("Expected Int(10), got {:?}", result);
    }
}

#[test]
fn test_lambda_multiple_params() {
    let source = r#"
l = lambda do |x, y|
  x + y
end
l.call(3, 4)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 7);
    } else {
        panic!("Expected Int(7), got {:?}", result);
    }
}

#[test]
fn test_arrow_lambda_zero_params() {
    let source = r#"
l = -> 42
l.call
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 42);
    } else {
        panic!("Expected Int(42), got {:?}", result);
    }
}

#[test]
fn test_arrow_lambda_single_param() {
    let source = r#"
double = x -> x * 2
double.call(5)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 10);
    } else {
        panic!("Expected Int(10), got {:?}", result);
    }
}

#[test]
fn test_arrow_lambda_with_expression() {
    let source = r#"
add_ten = x -> x + 10
add_ten.call(5)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 15);
    } else {
        panic!("Expected Int(15), got {:?}", result);
    }
}

#[test]
fn test_arrow_lambda_multi_params() {
    let source = r#"
add = (x, y) -> x + y
add.call(3, 4)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 7);
    } else {
        panic!("Expected Int(7), got {:?}", result);
    }
}

#[test]
fn test_arrow_lambda_three_params() {
    let source = r#"
sum = (x, y, z) -> x + y + z
sum.call(1, 2, 3)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 6);
    } else {
        panic!("Expected Int(6), got {:?}", result);
    }
}

#[test]
fn test_brace_lambda_no_params() {
    let source = r#"
l = lambda { || 42 }
l.call
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 42);
    } else {
        panic!("Expected Int(42), got {:?}", result);
    }
}

#[test]
fn test_brace_lambda_single_param() {
    let source = r#"
double = lambda { |x| x * 2 }
double.call(5)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 10);
    } else {
        panic!("Expected Int(10), got {:?}", result);
    }
}

#[test]
fn test_brace_lambda_multi_params() {
    let source = r#"
add = lambda { |a, b| a + b }
add.call(3, 7)
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 10);
    } else {
        panic!("Expected Int(10), got {:?}", result);
    }
}

#[test]
fn test_standalone_block_simple() {
    let source = r#"
result = do
  42
end
result
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    // Standalone blocks create lambda objects
    assert!(result.is_some());
    assert!(matches!(result, Some(Object::Block(_))));
}

#[test]
fn test_standalone_block_with_statements() {
    let source = r#"
result = do
  x = 10
  y = 20
  x + y
end
result
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    // Standalone blocks create lambda objects
    assert!(result.is_some());
    assert!(matches!(result, Some(Object::Block(_))));
}

#[test]
fn test_block_parameter_syntax() {
    let source = r#"
def times(n, &block)
  i = 0
  while i < n
    block.call()
    i = i + 1
  end
end

count = 0
times(3) { count = count + 1 }
count
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().expect("Parsing failed");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_program(&statements).expect("Execution failed");

    assert!(result.is_some());
    if let Some(Object::Int(val)) = result {
        assert_eq!(val, 3);
    } else {
        panic!("Expected Int(3), got {:?}", result);
    }
}

// Helper for concise tests
fn run_lambda(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// ── Stabby lambda -> ────────────────────────────────────────────────────────

#[test]
fn stabby_lambda_brace() {
    assert_eq!(run_lambda("f = -> { 42 }; f.call"), Some(Object::Int(42)));
}

#[test]
fn stabby_lambda_with_params() {
    assert_eq!(
        run_lambda("f = -> { |x| x * 2 }; f.call(5)"),
        Some(Object::Int(10))
    );
}

#[test]
fn stabby_lambda_as_argument() {
    assert_eq!(
        run_lambda("def apply(fn); fn.call; end; apply(-> { 99 })"),
        Some(Object::Int(99))
    );
}

#[test]
fn stabby_lambda_do_end() {
    assert_eq!(
        run_lambda("f = -> do\n  42\nend\nf.call"),
        Some(Object::Int(42))
    );
}

#[test]
fn stabby_lambda_parens_brace() {
    assert_eq!(
        run_lambda("f = -> (x) { x + 1 }; f.call(9)"),
        Some(Object::Int(10))
    );
}

#[test]
fn stabby_lambda_parens_multi() {
    assert_eq!(
        run_lambda("f = -> (a, b) { a * b }; f.call(3, 4)"),
        Some(Object::Int(12))
    );
}

#[test]
fn stabby_lambda_no_body_expr() {
    assert_eq!(run_lambda("f = -> 42; f.call"), Some(Object::Int(42)));
}

#[test]
fn stabby_lambda_in_parens_arg_brace() {
    assert_eq!(
        run_lambda("def r(fn); fn.call; end; r(-> { 77 })"),
        Some(Object::Int(77))
    );
}

#[test]
fn stabby_lambda_in_parens_arg_do() {
    assert_eq!(
        run_lambda("def r(fn)\n  fn.call\nend\nr(-> do\n  88\nend)"),
        Some(Object::Int(88))
    );
}

#[test]
fn stabby_lambda_in_parens_arg_with_params() {
    assert_eq!(
        run_lambda("def r(fn); fn.call(5); end; r(-> (x) { x * 3 })"),
        Some(Object::Int(15))
    );
}

#[test]
fn stabby_lambda_in_parens_arg_expr() {
    assert_eq!(
        run_lambda("def r(fn); fn.call; end; r(-> 42)"),
        Some(Object::Int(42))
    );
}

#[test]
fn arrow_lambda_parens_do_end() {
    assert_eq!(
        run_lambda("f = -> (x) do\n  x + 1\nend\nf.call(9)"),
        Some(Object::Int(10))
    );
}

#[test]
fn arrow_lambda_parens_expr() {
    assert_eq!(
        run_lambda("f = -> (x) x * 2; f.call(5)"),
        Some(Object::Int(10))
    );
}

// ── Lambda [] call ──────────────────────────────────────────────────────────

#[test]
fn lambda_bracket_call_single_arg() {
    assert_eq!(
        run_lambda("f = lambda { |x| x * 3 }; f[7]"),
        Some(Object::Int(21))
    );
}

#[test]
fn lambda_bracket_call_multi_arg() {
    assert_eq!(
        run_lambda("f = lambda { |a, b| a + b }; f[3, 4]"),
        Some(Object::Int(7))
    );
}

// ── Block captured vars ─────────────────────────────────────────────────────

#[test]
fn block_captured_vars() {
    assert_eq!(
        run_lambda("x = 10\nf = lambda { x + 5 }\nf.call"),
        Some(Object::Int(15))
    );
}

// ── Proc and lambda are distinct kinds ───────────────────────────────────────

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

#[test]
fn case_in_inside_a_block_yields_its_value() {
    let result = run_lambda(
        r#"
result = [1, 2].map do |n|
  case n
  in 1
    "one"
  in 2
    "two"
  end
end
result[1]
"#,
    );
    assert_eq!(result, Some(Object::string("two")));
}

#[test]
fn a_proc_ignores_extra_arguments() {
    let result = run_lambda("proc { |a| a }.call(1, 2, 3)");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn a_proc_fills_missing_arguments_with_nil() {
    let result = run_lambda("proc { |a, b| b }.call(1).inspect");
    assert_eq!(result, Some(Object::string("nil")));
}

#[test]
fn a_lambda_rejects_extra_arguments() {
    let error = run_err("lambda { |a| a }.call(1, 2)");
    assert!(error.contains("expected 1 argument(s) but received 2"));
}

#[test]
fn a_lambda_rejects_missing_arguments() {
    let error = run_err("lambda { |a, b| a }.call(1)");
    assert!(error.contains("expected 2 argument(s) but received 1"));
}

#[test]
fn a_do_end_lambda_is_a_lambda() {
    let result = run_lambda("lambda do 1 end.lambda?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn send_reaches_kernel_lambda() {
    let result = run_lambda(
        r#"
class Holder
  def make
    send(:lambda) { 1 }
  end
end
Holder.new.make.lambda?
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn lambda_without_a_block_raises_argument_error() {
    let error = run_err("lambda");
    assert!(error.contains("tried to create Proc object without a block"));
}

#[test]
fn lambda_is_a_private_instance_method_on_kernel() {
    let result = run_lambda("Kernel.private_instance_methods(false).include?(:lambda)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_symbol_to_proc_block_is_not_a_lambda() {
    let result = run_lambda("proc { |x| x }.lambda?");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Kernel#proc ──────────────────────────────────────────────────────────────

#[test]
fn proc_hands_back_an_existing_proc_unchanged() {
    let result = run_lambda(
        r#"
stabby = -> { 7 }
proc(&stabby).equal?(stabby)
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn proc_keeps_a_lambda_a_lambda() {
    let result = run_lambda(
        r#"
stabby = -> { 7 }
proc(&stabby).lambda?
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn send_reaches_kernel_proc() {
    let result = run_lambda(
        r#"
class Holder
  def build
    send(:proc) { :from_send }
  end
end
Holder.new.build.call.inspect
"#,
    );
    assert_eq!(result, Some(Object::string(":from_send")));
}

#[test]
fn a_bare_proc_without_a_block_raises_argument_error() {
    let error = run_err(
        r#"
class Holder
  def no_block
    proc
  end
end
Holder.new.no_block
"#,
    );
    assert!(error.contains("tried to create Proc object without a block"));
}

#[test]
fn proc_is_a_private_instance_method_on_kernel() {
    let result = run_lambda("Kernel.private_instance_methods(false).include?(:proc)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── equal? is identity for reference types ───────────────────────────────────

#[test]
fn equal_compares_two_blocks_by_identity() {
    let result = run_lambda(
        r#"
first = proc { 1 }
second = proc { 1 }
[first.equal?(first), first.equal?(second)].inspect
"#,
    );
    assert_eq!(result, Some(Object::string("[true, false]")));
}

#[test]
fn equal_terminates_on_a_block_that_closed_over_itself() {
    let result = run_lambda(
        r#"
looping = nil
looping = proc { looping }
looping.equal?(looping)
"#,
    );
    assert_eq!(result, Some(Object::Bool(true)));
}
