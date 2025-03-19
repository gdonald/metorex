// Tests for the Metorex REPL

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::repl::Repl;
use metorex::vm::VirtualMachine;

/// Helper function to evaluate a single expression in a fresh VM
fn eval_expr(source: &str) -> Result<Option<Object>, String> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| format!("{:?}", e))?;
    let mut vm = VirtualMachine::new();
    vm.execute_program(&program).map_err(|e| e.to_string())
}

/// Helper function to evaluate multiple statements in sequence
fn eval_sequence(sources: &[&str]) -> Result<Vec<Option<Object>>, String> {
    let mut vm = VirtualMachine::new();
    let mut results = Vec::new();

    for source in sources {
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| format!("{:?}", e))?;
        let result = vm.execute_program(&program).map_err(|e| e.to_string())?;
        results.push(result);
    }

    Ok(results)
}

#[test]
fn test_repl_creation() {
    let result = Repl::new();
    assert!(result.is_ok(), "REPL should initialize successfully");
}

#[test]
fn test_repl_simple_arithmetic() {
    let result = eval_expr("1 + 2");
    assert!(result.is_ok());
    match result.unwrap() {
        Some(Object::Int(3)) => (),
        other => panic!("Expected Int(3), got {:?}", other),
    }
}

#[test]
fn test_repl_variable_assignment_and_retrieval() {
    let results = eval_sequence(&["x = 42", "x"]);
    assert!(results.is_ok());
    let results = results.unwrap();
    assert_eq!(results.len(), 2);

    // First statement (assignment) returns None or the assigned value
    // Second statement should return the value
    match &results[1] {
        Some(Object::Int(42)) => (),
        other => panic!("Expected Integer(42), got {:?}", other),
    }
}

#[test]
fn test_repl_string_interpolation() {
    let results = eval_sequence(&["name = \"Ada\"", "\"Hello, #{name}!\""]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[1] {
        Some(Object::String(s)) if s.as_str() == "Hello, Ada!" => (),
        other => panic!("Expected String(\"Hello, Ada!\"), got {:?}", other),
    }
}

#[test]
fn test_repl_array_operations() {
    let results = eval_sequence(&["arr = [1, 2, 3]", "arr.length"]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[1] {
        Some(Object::Int(3)) => (),
        other => panic!("Expected Integer(3), got {:?}", other),
    }
}

#[test]
fn test_repl_hash_operations() {
    let results = eval_sequence(&[
        "{\"name\" => \"Bob\", \"age\" => 30}",
        "h = {\"x\" => 10}",
        "h[\"x\"]",
    ]);
    assert!(results.is_ok());
    let results = results.unwrap();

    // Check the hash lookup result
    match &results[2] {
        Some(Object::Int(10)) => (),
        other => panic!("Expected Integer(10), got {:?}", other),
    }
}

#[test]
fn test_repl_function_definition_and_call() {
    let results = eval_sequence(&["def add(a, b)\n  a + b\nend", "add(5, 7)"]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[1] {
        Some(Object::Int(12)) => (),
        other => panic!("Expected Integer(12), got {:?}", other),
    }
}

#[test]
fn test_repl_class_definition_and_instantiation() {
    let results = eval_sequence(&[
        "class Person\n  def initialize(name)\n    @name = name\n  end\n  def name\n    @name\n  end\nend",
        "p = Person.new(\"Alice\")",
        "p.name",
    ]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[2] {
        Some(Object::String(s)) if s.as_str() == "Alice" => (),
        other => panic!("Expected String(\"Alice\"), got {:?}", other),
    }
}

#[test]
fn test_repl_lambda_creation_and_call() {
    let results = eval_sequence(&["square = lambda do |x| x * x end", "square.call(5)"]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[1] {
        Some(Object::Int(25)) => (),
        other => panic!("Expected Integer(25), got {:?}", other),
    }
}

#[test]
fn test_repl_block_with_each() {
    let results = eval_sequence(&["sum = 0", "[1, 2, 3].each do |x| sum = sum + x end", "sum"]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[2] {
        Some(Object::Int(6)) => (),
        other => panic!("Expected Integer(6), got {:?}", other),
    }
}

// TODO: Re-enable when if statement returns values properly
// #[test]
// fn test_repl_if_statement() {
//     let results = eval_sequence(&["x = 10", "if x > 5\n  \"big\"\nelse\n  \"small\"\nend"]);
//     assert!(results.is_ok());
//     let results = results.unwrap();
//
//     match &results[1] {
//         Some(Object::String(s)) if s.as_str() == "big" => (),
//         other => panic!("Expected String(\"big\"), got {:?}", other),
//     }
// }

#[test]
fn test_repl_range_operations() {
    let results = eval_sequence(&["r = 1..5", "r.to_a"]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[1] {
        Some(Object::Array(arr)) if arr.borrow().len() == 5 => {
            // Check that array contains [1, 2, 3, 4, 5]
            for (i, obj) in arr.borrow().iter().enumerate() {
                match obj {
                    Object::Int(n) if *n == (i as i64 + 1) => (),
                    other => panic!("Expected Int({}), got {:?}", i + 1, other),
                }
            }
        }
        other => panic!("Expected Array of length 5, got {:?}", other),
    }
}

// TODO: Re-enable when parser error detection is improved
// #[test]
// fn test_repl_error_recovery() {
//     // First command has a syntax error, but VM should still work for next command
//     let result1 = eval_expr("1 + + 2"); // Syntax error
//     assert!(result1.is_err());

//
//     // REPL should recover and work for the next command
//     let result2 = eval_expr("2 + 3");
//     assert!(result2.is_ok());
//     match result2.unwrap() {
//         Some(Object::Int(5)) => (),
//         other => panic!("Expected Int(5), got {:?}", other),
//     }
// }

#[test]
fn test_repl_persistent_state() {
    // Variables should persist across evaluations in the same VM
    let results = eval_sequence(&[
        "counter = 0",
        "counter = counter + 1",
        "counter = counter + 1",
        "counter",
    ]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[3] {
        Some(Object::Int(2)) => (),
        other => panic!("Expected Integer(2), got {:?}", other),
    }
}

#[test]
fn test_repl_method_chaining() {
    let result =
        eval_expr("[1, 2, 3, 4].map(lambda do |x| x * 2 end).filter(lambda do |x| x > 4 end)");
    assert!(result.is_ok());
    match result.unwrap() {
        Some(Object::Array(arr)) => {
            let arr_borrowed = arr.borrow();
            assert_eq!(arr_borrowed.len(), 2);
            // Should contain [6, 8]
            match (&arr_borrowed[0], &arr_borrowed[1]) {
                (Object::Int(6), Object::Int(8)) => (),
                other => panic!("Expected [6, 8], got {:?}", other),
            }
        }
        other => panic!("Expected Array, got {:?}", other),
    }
}

#[test]
fn test_repl_closure_capture() {
    let results = eval_sequence(&[
        "def make_counter()\n  count = 0\n  lambda do\n    count = count + 1\n    count\n  end\nend",
        "counter = make_counter()",
        "counter.call()",
        "counter.call()",
        "counter.call()",
    ]);
    assert!(results.is_ok());
    let results = results.unwrap();

    // Check that each call increments the counter
    match (&results[2], &results[3], &results[4]) {
        (Some(Object::Int(1)), Some(Object::Int(2)), Some(Object::Int(3))) => (),
        other => panic!("Expected (1, 2, 3), got {:?}", other),
    }
}

#[test]
fn test_repl_nil_handling() {
    let result = eval_expr("nil");
    assert!(result.is_ok());
    match result.unwrap() {
        Some(Object::Nil) => (),
        other => panic!("Expected Nil, got {:?}", other),
    }
}

// TODO: Re-enable when boolean operators (&&, ||, !) are implemented
// #[test]
// fn test_repl_boolean_operations() {
//     let results = eval_sequence(&["true && false", "true || false", "!true"]);
//     assert!(results.is_ok());
//     let results = results.unwrap();
//
//     match (&results[0], &results[1], &results[2]) {
//         (
//             Some(Object::Bool(false)),
//             Some(Object::Bool(true)),
//             Some(Object::Bool(false)),
//         ) => (),
//         other => panic!("Expected (false, true, false), got {:?}", other),
//     }
// }

#[test]
fn test_repl_string_methods() {
    let results = eval_sequence(&["\"hello\".upcase", "\"WORLD\".downcase", "\"test\".length"]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match (&results[0], &results[1], &results[2]) {
        (Some(Object::String(s1)), Some(Object::String(s2)), Some(Object::Int(4)))
            if s1.as_str() == "HELLO" && s2.as_str() == "world" =>
        {
            ()
        }
        other => panic!("Expected correct string method results, got {:?}", other),
    }
}

#[test]
fn test_repl_super_keyword() {
    let results = eval_sequence(&[
        "class Animal\n  def speak\n    \"Some sound\"\n  end\nend",
        "class Dog < Animal\n  def speak\n    super() + \" and woof\"\n  end\nend",
        "dog = Dog.new()",
        "dog.speak",
    ]);
    assert!(results.is_ok());
    let results = results.unwrap();

    match &results[3] {
        Some(Object::String(s)) if s.as_str() == "Some sound and woof" => (),
        other => panic!("Expected String(\"Some sound and woof\"), got {:?}", other),
    }
}
