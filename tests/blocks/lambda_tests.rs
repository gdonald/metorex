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
fn test_compact_lambda_no_params() {
    let source = r#"
l = lambda || 42 end
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
fn test_compact_lambda_single_param() {
    let source = r#"
double = lambda |x| x * 2 end
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
fn test_compact_lambda_multi_params() {
    let source = r#"
add = lambda |a, b| a + b end
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
times(3, lambda || count = count + 1 end)
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
