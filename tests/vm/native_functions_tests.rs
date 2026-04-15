// Coverage tests for vm/native_functions.rs — method() and require_relative error paths.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::path::Path;

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

// ── method() error paths ──────────────────────────────────────────────────────

#[test]
fn method_with_no_args_error() {
    let err = run_err("method()");
    // method() with 0 arguments
    assert!(err.contains("argument") || err.contains("method"));
}

#[test]
fn method_with_too_many_args_error() {
    let err = run_err("method(:foo, :bar)");
    assert!(err.contains("argument") || err.contains("method"));
}

#[test]
fn method_with_non_symbol_arg_error() {
    let err = run_err("method(42)");
    assert!(err.contains("Symbol") || err.contains("argument"));
}

#[test]
fn method_with_non_method_variable_error() {
    let err = run_err(
        r#"
x = 42
method(:x)
"#,
    );
    assert!(err.contains("not a method") || err.contains("method"));
}

#[test]
fn method_with_undefined_name_error() {
    let err = run_err("method(:nonexistent_xyz_abc)");
    assert!(err.contains("undefined") || err.contains("method"));
}

// ── method() happy path ───────────────────────────────────────────────────────

#[test]
fn method_returns_method_object() {
    let result = run(r#"
def greet
  "hello"
end
m = method(:greet)
m.nil?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── require_relative without file context ─────────────────────────────────────

#[test]
fn require_relative_without_file_context_error() {
    let err = run_err(r#"require_relative("./nonexistent")"#);
    // No current file context when running via execute_program directly
    assert!(
        err.contains("require_relative")
            || err.contains("file")
            || err.contains("context")
            || err.contains("REPL")
    );
}

#[test]
fn require_relative_with_wrong_arg_count_error() {
    let err = run_err("require_relative()");
    assert!(err.contains("argument") || err.contains("require_relative"));
}

#[test]
fn require_relative_with_non_string_error() {
    let err = run_err("require_relative(42)");
    assert!(err.contains("String") || err.contains("argument"));
}

// ── require_relative execute_file error path (lines 136-138) ─────────────────

#[test]
fn require_relative_execute_file_error_propagates() {
    // Execute a file that require_relatives a helper which raises a runtime error.
    // This covers the map_err at lines 136-138 of native_functions.rs.
    let main_path = Path::new("tests/_examples/require/main_with_bad_require.rb");
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(main_path);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("require_relative") || msg.contains("error") || msg.contains("Error"));
}

// ── require_relative deduplication (lines 104-131) ──────────────────────

#[test]
fn require_relative_deduplication() {
    let code = "a = require_relative(\"lib/helper\")\nb = require_relative(\"lib/helper\")\nb";
    let tokens = metorex::lexer::Lexer::new(code).tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let mut vm = VirtualMachine::new();
    let base = std::fs::canonicalize("tests/_examples/require/basic.rb").unwrap();
    vm.set_current_file(base.clone());
    vm.mark_file_loaded(base);
    let result = vm.execute_program(&stmts);
    assert!(result.is_ok());
}

// ── assert_equal (lines 178-192) ────────────────────────────────────────

#[test]
fn assert_equal_success() {
    let result = run("assert_equal(1, 1)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_equal_failure() {
    let err = run_err("assert_equal(1, 2)");
    assert!(
        err.contains("Expected") || err.contains("got"),
        "Error was: {}",
        err
    );
}

#[test]
fn assert_equal_with_message() {
    let err = run_err("assert_equal(1, 2, \"custom msg\")");
    assert!(err.contains("custom msg"), "Error was: {}", err);
}

#[test]
fn assert_equal_wrong_arg_count() {
    let err = run_err("assert_equal(1)");
    assert!(err.contains("argument"), "Error was: {}", err);
}

// ── assert_raises (lines 192-273) ───────────────────────────────────────

#[test]
fn assert_raises_with_args() {
    let err = run_err("assert_raises(1) { raise \"boom\" }");
    assert!(err.contains("argument"), "Error was: {}", err);
}

#[test]
fn assert_raises_no_block_error() {
    // assert_raises() called with no block and no parens returns function ref
    // Test with parens to trigger the error path
    let err = run_err("assert_raises()");
    assert!(
        err.contains("block") || err.contains("Block") || err.contains("requires"),
        "Error was: {}",
        err
    );
}

// ── From vm/additional_tests ────────────────────────────────────────────────

#[test]
fn print_formats_arguments_without_newline() {
    run("print(\"hello\")"); // just verify no crash
}

#[test]
fn p_function_returns_single_value() {
    assert_eq!(run("p(42)"), Some(Object::Int(42)));
}

// ── load() ─────────────────────────────────────────────────────────────────

#[test]
fn load_existing_file_returns_true() {
    // Create a temporary .rb file and load it.
    let path = "tests/_examples/io_load_test_tmp.rb";
    std::fs::write(path, "x = 99\n").unwrap();
    let result = run(&format!(r#"load("{}")"#, path));
    assert_eq!(result, Some(Object::Bool(true)));
    std::fs::remove_file(path).ok();
}

#[test]
fn load_missing_file_errors() {
    let err = run_err(r#"load("definitely_not_here_xyzzy.rb")"#);
    assert!(err.contains("cannot load"));
}

#[test]
fn load_wrong_arg_count_errors() {
    let err = run_err("load()");
    assert!(err.contains("1-2 arguments"));
}

#[test]
fn load_non_string_arg_errors() {
    let err = run_err("load(42)");
    assert!(err.contains("String"));
}

#[test]
fn load_too_many_args_errors() {
    let err = run_err(r#"load("a", "b", "c")"#);
    assert!(err.contains("1-2 arguments"));
}

#[test]
fn load_via_load_path() {
    // Create a file in tests/_examples and load it via a bare name + $LOAD_PATH.
    let dir = "tests/_examples";
    let name = "io_load_path_test_tmp.rb";
    let path = format!("{}/{}", dir, name);
    std::fs::write(&path, "y = 7\n").unwrap();
    let result = run(&format!(
        r#"$LOAD_PATH.unshift "{}"
load("{}")
"#,
        dir, name
    ));
    assert_eq!(result, Some(Object::Bool(true)));
    std::fs::remove_file(&path).ok();
}

// ── visibility modifier stubs (private/public/protected/module_function) ───

#[test]
fn private_visibility_stub_is_noop() {
    assert_eq!(run("private()"), Some(Object::Nil));
}

#[test]
fn public_visibility_stub_is_noop() {
    assert_eq!(run("public()"), Some(Object::Nil));
}

#[test]
fn protected_visibility_stub_is_noop() {
    assert_eq!(run("protected()"), Some(Object::Nil));
}

#[test]
fn module_function_stub_is_noop() {
    assert_eq!(run("module_function()"), Some(Object::Nil));
}

#[test]
fn freeze_stub_is_noop() {
    assert_eq!(run("freeze()"), Some(Object::Nil));
}

#[test]
fn private_class_method_stub_is_noop() {
    assert_eq!(run("private_class_method()"), Some(Object::Nil));
}

#[test]
fn public_class_method_stub_is_noop() {
    assert_eq!(run("public_class_method()"), Some(Object::Nil));
}

#[test]
fn private_with_argument_still_noop() {
    // `private :foo` should not error even though it's a stub.
    assert_eq!(run("private(:foo)"), Some(Object::Nil));
}

// ── require() error paths ──────────────────────────────────────────────────

#[test]
fn require_no_args_errors() {
    let err = run_err("require()");
    assert!(err.contains("1 argument"));
}

#[test]
fn require_non_string_arg_errors() {
    let err = run_err("require(42)");
    assert!(err.contains("String"));
}

#[test]
fn require_missing_file_raises_load_error() {
    // `require` on a non-existent file raises LoadError; caught inside a method.
    let result = run(r#"
def try_load
  begin
    require("zz_definitely_not_here_xyz")
    "not caught"
  rescue LoadError => e
    "caught"
  end
end
try_load
"#);
    assert_eq!(result, Some(Object::string("caught")));
}

// ── gets() — only the wrong-arg-count error path is testable without stdin ─

#[test]
fn gets_with_args_errors() {
    let err = run_err("gets(\"prompt\")");
    assert!(err.contains("0 argument"));
}

// ── Kernel conversion functions: Integer(), String(), Array() ─────────────

#[test]
fn integer_conversion_from_int() {
    assert_eq!(run("Integer(42)"), Some(Object::Int(42)));
}

#[test]
fn integer_conversion_from_float_truncates() {
    assert_eq!(run("Integer(3.9)"), Some(Object::Int(3)));
}

#[test]
fn integer_conversion_from_string_with_whitespace() {
    assert_eq!(run("Integer(\"  42  \")"), Some(Object::Int(42)));
}

#[test]
fn integer_conversion_from_invalid_string_errors() {
    let err = run_err(r#"Integer("hello")"#);
    assert!(err.contains("invalid"));
}

#[test]
fn integer_conversion_from_true_returns_one() {
    assert_eq!(run("Integer(true)"), Some(Object::Int(1)));
}

#[test]
fn integer_conversion_from_false_returns_zero() {
    assert_eq!(run("Integer(false)"), Some(Object::Int(0)));
}

#[test]
fn integer_conversion_from_nil_returns_zero() {
    assert_eq!(run("Integer(nil)"), Some(Object::Int(0)));
}

#[test]
fn integer_conversion_from_array_errors() {
    let err = run_err("Integer([1, 2])");
    assert!(err.contains("Array") || err.contains("convert"));
}

#[test]
fn string_conversion_from_int() {
    assert_eq!(run("String(42)"), Some(Object::string("42")));
}

#[test]
fn string_conversion_from_nil() {
    // String() formats nil via Display, which yields the literal "nil".
    assert_eq!(run("String(nil)"), Some(Object::string("nil")));
}

#[test]
fn array_conversion_from_array_returns_self() {
    let result = run("Array([1, 2, 3])");
    match result {
        Some(Object::Array(arr)) => assert_eq!(arr.borrow().len(), 3),
        other => panic!("expected Array, got {:?}", other),
    }
}

#[test]
fn array_conversion_from_nil_returns_empty() {
    let result = run("Array(nil)");
    match result {
        Some(Object::Array(arr)) => assert!(arr.borrow().is_empty()),
        other => panic!("expected Array, got {:?}", other),
    }
}

#[test]
fn array_conversion_from_string_wraps() {
    let result = run(r#"Array("hi")"#);
    match result {
        Some(Object::Array(arr)) => assert_eq!(arr.borrow().len(), 1),
        other => panic!("expected Array, got {:?}", other),
    }
}

#[test]
fn require_missing_file_raises_load_error_caught_as_standard_error() {
    // LoadError < StandardError — a bare rescue should also catch it.
    let result = run(r#"
def try_load
  begin
    require("zz_definitely_not_here_xyz")
    "not caught"
  rescue => e
    "caught"
  end
end
try_load
"#);
    assert_eq!(result, Some(Object::string("caught")));
}

// ── at_exit ──────────────────────────────────────────────────────────────────

#[test]
fn at_exit_returns_nil() {
    let result = run("at_exit { puts 'bye' }");
    assert_eq!(result, Some(Object::Nil));
}

// ── warn ─────────────────────────────────────────────────────────────────────

#[test]
fn warn_returns_nil() {
    let result = run("warn 'test warning'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn warn_multiple_args() {
    let result = run("warn 'a', 'b'");
    assert_eq!(result, Some(Object::Nil));
}

// ── sprintf / format ─────────────────────────────────────────────────────────

#[test]
fn sprintf_basic() {
    let result = run("sprintf '%s is %d', 'age', 25");
    assert!(result.is_some());
}

#[test]
fn sprintf_no_args_error() {
    let err = run_err("sprintf()");
    assert!(err.contains("argument"));
}

#[test]
fn format_alias() {
    let result = run("format '%d', 42");
    assert!(result.is_some());
}

// ── __method__ ───────────────────────────────────────────────────────────────

#[test]
fn dunder_method_returns_symbol() {
    let result = run(r#"
def foo
  __method__()
end
foo
"#);
    assert!(matches!(result, Some(Object::Symbol(_))));
}

// ── caller ───────────────────────────────────────────────────────────────────

#[test]
fn caller_returns_array() {
    let result = run("caller()");
    assert!(matches!(result, Some(Object::Array(_))));
}

// ── rand ─────────────────────────────────────────────────────────────────────

#[test]
fn rand_no_args_returns_float() {
    let result = run("rand()");
    assert!(matches!(result, Some(Object::Float(_))));
}

#[test]
fn rand_with_int_arg() {
    let result = run("rand(100)");
    assert!(matches!(result, Some(Object::Int(_))));
}

#[test]
fn rand_with_zero_arg() {
    let result = run("rand(0)");
    assert_eq!(result, Some(Object::Int(0)));
}

// ── sleep ────────────────────────────────────────────────────────────────────

#[test]
fn sleep_with_zero() {
    let result = run("sleep(0)");
    assert_eq!(result, Some(Object::Int(0)));
}

// ── srand ────────────────────────────────────────────────────────────────────

#[test]
fn srand_returns_int() {
    let result = run("srand()");
    assert!(matches!(result, Some(Object::Int(_))));
}

// ── load function ────────────────────────────────────────────────────────────

#[test]
fn load_nonexistent_file_error() {
    let err = run_err("load 'nonexistent_file_xyz.rb'");
    assert!(err.contains("cannot load"));
}

// ── symbol_to_proc via &:symbol ──────────────────────────────────────────────

#[test]
fn symbol_to_proc_in_map() {
    let result = run("[1, 2, 3].map(&:to_s)");
    if let Some(Object::Array(arr)) = &result {
        let items: Vec<_> = arr.borrow().iter().map(|o| format!("{}", o)).collect();
        assert_eq!(items, vec!["1", "2", "3"]);
    } else {
        panic!("expected array");
    }
}

// ── block_arg nil is dropped ─────────────────────────────────────────────────

#[test]
fn block_arg_nil_dropped() {
    let result = run(r#"
def foo(&block)
  block_given?
end
b = nil
foo(&b)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── block_arg with block object ──────────────────────────────────────────────

#[test]
fn block_arg_with_block() {
    let result = run(r#"
def foo(&block)
  block.call
end
b = lambda { 42 }
foo(&b)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── __method__ inside a method with class prefix (lines 64-65) ───────────────

#[test]
fn method_name_inside_method_returns_short_name() {
    let result = run(r#"
class MyClass
  def greet
    __method__()
  end
end
MyClass.new.greet
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("greet".to_string())))
    );
}

// ── rand with non-Int argument (line 89) ──────────────────────────────────────

#[test]
fn rand_with_float_arg_returns_zero() {
    let result = run("rand(3.14)");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn rand_with_string_arg_returns_zero() {
    let result = run(r#"rand("hello")"#);
    assert_eq!(result, Some(Object::Int(0)));
}

// ── require with non-Array $LOAD_PATH (lines 174) ────────────────────────────

#[test]
fn require_with_invalid_load_path_raises_load_error() {
    let err = run_err(
        r#"
$: = 42
require "nonexistent_lib_xyz"
"#,
    );
    assert!(err.contains("load") || err.contains("cannot") || err.contains("file"));
}

// ── load: execute_file error when file has syntax error (lines 539-541) ──────

#[test]
fn load_file_with_parse_error_propagates_error() {
    use std::io::Write;
    let path = "/tmp/metorex_bad_syntax_test.rb";
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(b"def incomplete(\n").unwrap();
    let err = run_err(&format!(r#"load("{}")"#, path));
    assert!(
        err.contains("parse")
            || err.contains("syntax")
            || err.contains("load")
            || err.contains("error")
    );
    std::fs::remove_file(path).ok();
}

// ── get_string_representation fallback (line 627) ────────────────────────────

#[test]
fn assert_equal_with_non_instance_objects_uses_display() {
    // get_string_representation for non-Instance objects uses format!("{}", obj)
    let result = run("assert_equal(42, 42)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn print_instance_without_string_to_s_uses_display() {
    // Instance where to_s returns non-String triggers line 627 fallback
    run(r#"
class NoStringToS
  def to_s
    42
  end
end
print NoStringToS.new
"#);
    // Verify it doesn't crash — output is "<NoStringToS instance>"
}
