// Tests for the Metorex REPL

use metorex::object::Object;
use metorex::repl::{CommandResult, LineResult, Repl, ReplCore};
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// ReplCore — command handling
// ============================================================================

#[test]
fn command_exit() {
    let mut core = ReplCore::new();
    assert_eq!(core.handle_command(".exit"), CommandResult::Exit);
}

#[test]
fn command_quit() {
    let mut core = ReplCore::new();
    assert_eq!(core.handle_command(".quit"), CommandResult::Exit);
}

#[test]
fn command_help() {
    let mut core = ReplCore::new();
    match core.handle_command(".help") {
        CommandResult::Help(text) => {
            assert!(text.contains(".help"));
            assert!(text.contains(".exit"));
            assert!(text.contains(".reset"));
            assert!(text.contains("Ctrl-C"));
        }
        other => panic!("Expected Help, got {:?}", other),
    }
}

#[test]
fn command_clear() {
    let mut core = ReplCore::new();
    match core.handle_command(".clear") {
        CommandResult::Clear(text) => {
            assert!(text.contains("Metorex REPL"));
        }
        other => panic!("Expected Clear, got {:?}", other),
    }
}

#[test]
fn command_reset() {
    let mut core = ReplCore::new();
    // Set some state first
    core.process_line("x = 42");
    assert_eq!(core.vm().environment().get("x"), Some(Object::Int(42)));

    // Reset should clear VM state
    assert_eq!(core.handle_command(".reset"), CommandResult::Reset);
    assert_eq!(core.vm().environment().get("x"), None);
}

#[test]
fn command_unknown() {
    let mut core = ReplCore::new();
    match core.handle_command(".foo") {
        CommandResult::Unknown(msg) => {
            assert!(msg.contains("Unknown command"));
            assert!(msg.contains(".foo"));
        }
        other => panic!("Expected Unknown, got {:?}", other),
    }
}

// ============================================================================
// ReplCore — prompt
// ============================================================================

#[test]
fn prompt_normal_when_buffer_empty() {
    let core = ReplCore::new();
    assert_eq!(core.prompt(), ">> ");
    assert!(!core.is_continuation());
}

#[test]
fn prompt_continuation_when_buffer_has_content() {
    let mut core = ReplCore::new();
    core.process_line("def foo");
    // Buffer should be non-empty (incomplete input)
    assert!(core.is_continuation());
    assert_eq!(core.prompt(), ".. ");
}

// ============================================================================
// ReplCore — should_evaluate
// ============================================================================

#[test]
fn should_evaluate_empty_input() {
    let core = ReplCore::new();
    assert!(core.should_evaluate());
}

#[test]
fn should_evaluate_complete_expression() {
    let mut core = ReplCore::new();
    core.process_line("1 + 2");
    // After a complete expression, buffer is cleared
    assert!(core.buffer().is_empty());
}

// ============================================================================
// ReplCore — process_line
// ============================================================================

#[test]
fn process_line_simple_expression() {
    let mut core = ReplCore::new();
    match core.process_line("1 + 2") {
        LineResult::Value(text) => assert!(text.contains("3")),
        other => panic!("Expected Value, got {:?}", other),
    }
}

#[test]
fn process_line_assignment_returns_continue() {
    let mut core = ReplCore::new();
    // Assignments produce nil, which is suppressed
    match core.process_line("x = 42") {
        LineResult::Continue => {}
        other => panic!("Expected Continue for assignment, got {:?}", other),
    }
    assert_eq!(core.vm().environment().get("x"), Some(Object::Int(42)));
}

#[test]
fn process_line_exit_command() {
    let mut core = ReplCore::new();
    assert_eq!(core.process_line(".exit"), LineResult::Exit);
}

#[test]
fn process_line_help_command() {
    let mut core = ReplCore::new();
    match core.process_line(".help") {
        LineResult::Value(text) => assert!(text.contains(".help")),
        other => panic!("Expected Value with help text, got {:?}", other),
    }
}

#[test]
fn process_line_unknown_command() {
    let mut core = ReplCore::new();
    match core.process_line(".bogus") {
        LineResult::Error(msg) => assert!(msg.contains("Unknown command")),
        other => panic!("Expected Error, got {:?}", other),
    }
}

#[test]
fn process_line_incomplete_input() {
    let mut core = ReplCore::new();
    assert_eq!(core.process_line("def foo"), LineResult::Incomplete);
    assert!(core.is_continuation());
    assert!(!core.buffer().is_empty());
}

#[test]
fn process_line_multiline_completion() {
    let mut core = ReplCore::new();
    assert_eq!(core.process_line("def add(a, b)"), LineResult::Incomplete);
    assert_eq!(core.process_line("  a + b"), LineResult::Incomplete);
    // Completing the function definition
    let result = core.process_line("end");
    assert!(
        matches!(result, LineResult::Continue | LineResult::Value(_)),
        "Expected Continue or Value, got {:?}",
        result
    );
    assert!(core.buffer().is_empty());
}

#[test]
fn process_line_parse_error() {
    let mut core = ReplCore::new();
    match core.process_line("1 +* 2") {
        LineResult::Error(msg) => assert!(msg.contains("Parse error") || msg.contains("error")),
        other => panic!("Expected Error for syntax error, got {:?}", other),
    }
}

#[test]
fn process_line_runtime_error() {
    let mut core = ReplCore::new();
    match core.process_line("undefined_var_xyz") {
        LineResult::Error(msg) => assert!(msg.contains("Undefined variable")),
        other => panic!("Expected Error for undefined var, got {:?}", other),
    }
}

// ============================================================================
// ReplCore — clear_buffer
// ============================================================================

#[test]
fn clear_buffer_resets_continuation() {
    let mut core = ReplCore::new();
    core.process_line("def foo");
    assert!(core.is_continuation());
    core.clear_buffer();
    assert!(!core.is_continuation());
    assert!(core.buffer().is_empty());
}

// ============================================================================
// ReplCore — stateful evaluation (persistent VM)
// ============================================================================

#[test]
fn state_persists_across_lines() {
    let mut core = ReplCore::new();
    core.process_line("x = 10");
    core.process_line("y = 20");
    match core.process_line("x + y") {
        LineResult::Value(text) => assert!(text.contains("30")),
        other => panic!("Expected Value(30), got {:?}", other),
    }
}

#[test]
fn function_persists_across_lines() {
    let mut core = ReplCore::new();
    core.process_line("def add(a, b)");
    core.process_line("  a + b");
    core.process_line("end");
    match core.process_line("add(3, 4)") {
        LineResult::Value(text) => assert!(text.contains("7")),
        other => panic!("Expected Value(7), got {:?}", other),
    }
}

#[test]
fn class_persists_across_lines() {
    let mut core = ReplCore::new();
    core.process_line("class Dog");
    core.process_line("  def initialize(name)");
    core.process_line("    @name = name");
    core.process_line("  end");
    core.process_line("  def name");
    core.process_line("    @name");
    core.process_line("  end");
    core.process_line("end");
    match core.process_line("Dog.new(\"Rex\").name") {
        LineResult::Value(text) => assert!(text.contains("Rex")),
        other => panic!("Expected Value with Rex, got {:?}", other),
    }
}

#[test]
fn reset_clears_state() {
    let mut core = ReplCore::new();
    core.process_line("x = 42");
    assert_eq!(core.vm().environment().get("x"), Some(Object::Int(42)));
    core.handle_command(".reset");
    assert_eq!(core.vm().environment().get("x"), None);
}

// ============================================================================
// ReplCore — banner and help
// ============================================================================

#[test]
fn banner_contains_version() {
    let banner = ReplCore::banner();
    assert!(banner.contains("Metorex REPL"));
    assert!(banner.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_text_contains_commands() {
    let help = ReplCore::help_text();
    assert!(help.contains(".help"));
    assert!(help.contains(".exit"));
    assert!(help.contains(".quit"));
    assert!(help.contains(".clear"));
    assert!(help.contains(".reset"));
}

// ============================================================================
// ReplCore::format_object
// ============================================================================

#[test]
fn format_object_primitives() {
    assert_eq!(ReplCore::format_object(&Object::Nil), "nil");
    assert_eq!(ReplCore::format_object(&Object::Bool(true)), "true");
    assert_eq!(ReplCore::format_object(&Object::Bool(false)), "false");
    assert_eq!(ReplCore::format_object(&Object::Int(42)), "42");
    assert_eq!(ReplCore::format_object(&Object::Float(3.14)), "3.14");
    assert_eq!(
        ReplCore::format_object(&Object::String(Rc::new("hello".to_string()))),
        "\"hello\""
    );
}

#[test]
fn format_object_float_whole_number() {
    assert_eq!(ReplCore::format_object(&Object::Float(5.0)), "5.0");
}

#[test]
fn format_object_symbol() {
    assert_eq!(
        ReplCore::format_object(&Object::Symbol(Rc::new("foo".to_string()))),
        ":foo"
    );
}

#[test]
fn format_object_array() {
    let array = Object::Array(Rc::new(RefCell::new(vec![
        Object::Int(1),
        Object::Int(2),
        Object::Int(3),
    ])));
    assert_eq!(ReplCore::format_object(&array), "[1, 2, 3]");
}

#[test]
fn format_object_empty_array() {
    let array = Object::Array(Rc::new(RefCell::new(vec![])));
    assert_eq!(ReplCore::format_object(&array), "[]");
}

#[test]
fn format_object_nested_array() {
    let inner = Object::Array(Rc::new(RefCell::new(vec![Object::Int(1), Object::Int(2)])));
    let outer = Object::Array(Rc::new(RefCell::new(vec![inner, Object::Int(3)])));
    assert_eq!(ReplCore::format_object(&outer), "[[1, 2], 3]");
}

#[test]
fn format_object_dict() {
    let mut map = std::collections::HashMap::new();
    map.insert("a".to_string(), Object::Int(1));
    let dict = Object::Dict(Rc::new(RefCell::new(map)));
    assert_eq!(ReplCore::format_object(&dict), "{\"a\" => 1}");
}

#[test]
fn format_object_range_inclusive() {
    let range = Object::Range {
        start: Box::new(Object::Int(1)),
        end: Box::new(Object::Int(10)),
        exclusive: false,
    };
    assert_eq!(ReplCore::format_object(&range), "1..10");
}

#[test]
fn format_object_range_exclusive() {
    let range = Object::Range {
        start: Box::new(Object::Int(1)),
        end: Box::new(Object::Int(10)),
        exclusive: true,
    };
    assert_eq!(ReplCore::format_object(&range), "1...10");
}

#[test]
fn format_object_native_function() {
    let nf = Object::NativeFunction("puts".to_string());
    assert_eq!(ReplCore::format_object(&nf), "<NativeFunction: puts>");
}

#[test]
fn format_object_block() {
    // Block variant — just check it doesn't panic
    use metorex::object::BlockStatement;
    let block = Object::Block(Rc::new(BlockStatement {
        parameters: vec![],
        body: vec![],
        captured_vars: std::collections::HashMap::new(),
    }));
    assert_eq!(ReplCore::format_object(&block), "<Block>");
}

#[test]
fn format_object_result_ok() {
    let r = Object::Result(Ok(Box::new(Object::Int(42))));
    assert_eq!(ReplCore::format_object(&r), "<Ok: 42>");
}

#[test]
fn format_object_result_err() {
    let r = Object::Result(Err(Box::new(Object::String(Rc::new("oops".to_string())))));
    assert_eq!(ReplCore::format_object(&r), "<Err: \"oops\">");
}

// ============================================================================
// ReplCore::format_error
// ============================================================================

#[test]
fn format_error_returns_display() {
    use metorex::error::{MetorexError, SourceLocation};
    let err = MetorexError::runtime_error("test error", SourceLocation::new(1, 1, 0));
    let formatted = ReplCore::format_error(&err);
    assert!(formatted.contains("test error"));
}

// ============================================================================
// Repl (interactive wrapper) — minimal smoke test
// ============================================================================

#[test]
fn repl_creation() {
    let result = Repl::new();
    assert!(result.is_ok(), "REPL should initialize successfully");
}

#[test]
fn repl_default() {
    // Exercises the Default impl on Repl (covers the .expect path)
    let _repl = Repl::default();
}

#[test]
fn repl_core_default() {
    let core = ReplCore::default();
    assert!(core.buffer().is_empty());
    assert!(!core.is_continuation());
}

// ============================================================================
// Evaluation sequences (simulating multi-line REPL sessions)
// ============================================================================

#[test]
fn session_variable_and_retrieval() {
    let mut core = ReplCore::new();
    core.process_line("x = 42");
    match core.process_line("x") {
        LineResult::Value(text) => assert!(text.contains("42")),
        other => panic!("Expected Value(42), got {:?}", other),
    }
}

#[test]
fn session_string_interpolation() {
    let mut core = ReplCore::new();
    core.process_line("name = \"Ada\"");
    // Note: interpolated strings need to be tested carefully in Rust string literals
    let code = r#""Hello, #{name}!""#;
    let result = core.process_line(code);
    match result {
        LineResult::Value(text) => assert!(text.contains("Ada")),
        other => panic!("Expected Value with Ada, got {:?}", other),
    }
}

#[test]
fn session_array_operations() {
    let mut core = ReplCore::new();
    core.process_line("arr = [1, 2, 3]");
    match core.process_line("arr.length") {
        LineResult::Value(text) => assert!(text.contains("3")),
        other => panic!("Expected Value(3), got {:?}", other),
    }
}

#[test]
fn session_lambda_and_call() {
    let mut core = ReplCore::new();
    core.process_line("square = lambda do |x| x * x end");
    match core.process_line("square.call(5)") {
        LineResult::Value(text) => assert!(text.contains("25")),
        other => panic!("Expected Value(25), got {:?}", other),
    }
}

#[test]
fn session_block_with_each() {
    let mut core = ReplCore::new();
    core.process_line("sum = 0");
    core.process_line("[1, 2, 3].each do |x| sum = sum + x end");
    match core.process_line("sum") {
        LineResult::Value(text) => assert!(text.contains("6")),
        other => panic!("Expected Value(6), got {:?}", other),
    }
}

#[test]
fn session_closure_counter() {
    let mut core = ReplCore::new();
    core.process_line("def make_counter()");
    core.process_line("  count = 0");
    core.process_line("  lambda do");
    core.process_line("    count = count + 1");
    core.process_line("    count");
    core.process_line("  end");
    core.process_line("end");
    core.process_line("counter = make_counter()");
    match core.process_line("counter.call()") {
        LineResult::Value(text) => assert!(text.contains("1")),
        other => panic!("Expected Value(1), got {:?}", other),
    }
    match core.process_line("counter.call()") {
        LineResult::Value(text) => assert!(text.contains("2")),
        other => panic!("Expected Value(2), got {:?}", other),
    }
}

#[test]
fn session_range_to_array() {
    let mut core = ReplCore::new();
    core.process_line("r = 1..5");
    match core.process_line("r.to_a") {
        LineResult::Value(text) => {
            assert!(text.contains("[1, 2, 3, 4, 5]"));
        }
        other => panic!("Expected Value with array, got {:?}", other),
    }
}

#[test]
fn session_method_chaining() {
    let mut core = ReplCore::new();
    match core.process_line("[1, 2, 3, 4].map { |x| x * 2 }.filter { |x| x > 4 }") {
        LineResult::Value(text) => {
            assert!(text.contains("6"));
            assert!(text.contains("8"));
        }
        other => panic!("Expected Value with [6, 8], got {:?}", other),
    }
}

#[test]
fn session_string_methods() {
    let mut core = ReplCore::new();
    match core.process_line("\"hello\".upcase") {
        LineResult::Value(text) => assert!(text.contains("HELLO")),
        other => panic!("Expected Value(HELLO), got {:?}", other),
    }
}

#[test]
fn session_super_keyword() {
    let mut core = ReplCore::new();
    core.process_line("class Animal");
    core.process_line("  def speak");
    core.process_line("    \"Some sound\"");
    core.process_line("  end");
    core.process_line("end");
    core.process_line("class Dog < Animal");
    core.process_line("  def speak");
    core.process_line("    super() + \" and woof\"");
    core.process_line("  end");
    core.process_line("end");
    core.process_line("dog = Dog.new()");
    match core.process_line("dog.speak") {
        LineResult::Value(text) => assert!(text.contains("Some sound and woof")),
        other => panic!("Expected Value with 'Some sound and woof', got {:?}", other),
    }
}

#[test]
fn session_persistent_counter() {
    let mut core = ReplCore::new();
    core.process_line("counter = 0");
    core.process_line("counter = counter + 1");
    core.process_line("counter = counter + 1");
    match core.process_line("counter") {
        LineResult::Value(text) => assert!(text.contains("2")),
        other => panic!("Expected Value(2), got {:?}", other),
    }
}

#[test]
fn session_nil_handling() {
    let mut core = ReplCore::new();
    // nil should produce Continue (suppressed display)
    match core.process_line("nil") {
        LineResult::Continue => {}
        other => panic!("Expected Continue for nil, got {:?}", other),
    }
}
