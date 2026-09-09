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
fn module_function_outside_a_module_returns_nil() {
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
fn private_with_defined_method_returns_symbol() {
    // `private :foo` after defining `foo` returns :foo and marks it private on Object.
    assert_eq!(
        run("def foo; end\nprivate(:foo)"),
        Some(Object::symbol("foo".to_string()))
    );
}

#[test]
fn private_with_undefined_method_raises_name_error() {
    // `private :foo` without a definition raises NameError (Ruby semantics).
    let err = run_err("private(:foo)");
    assert!(err.contains("NameError") || err.contains("undefined method 'foo'"));
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
fn integer_conversion_from_true_raises() {
    let err = run_err("Integer(true)");
    assert!(err.contains("TrueClass"), "{}", err);
}

#[test]
fn integer_conversion_from_false_raises() {
    let err = run_err("Integer(false)");
    assert!(err.contains("FalseClass"), "{}", err);
}

#[test]
fn integer_conversion_from_nil_raises() {
    let err = run_err("Integer(nil)");
    assert!(err.contains("nil"), "{}", err);
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
    // String() goes through to_s, and nil.to_s is the empty string.
    assert_eq!(run("String(nil)"), Some(Object::string("")));
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
fn at_exit_returns_the_handler() {
    let result = run("at_exit { puts 'bye' }");
    assert!(matches!(result, Some(Object::Block(_))));
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
fn rand_with_zero_arg_gives_a_float() {
    // Ruby treats a bound of zero as no bound, so it draws a Float.
    let result = run("rand(0).is_a?(Float)");
    assert_eq!(result, Some(Object::Bool(true)));
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
    assert_eq!(result, Some(Object::symbol("greet".to_string())));
}

// ── rand with non-Int argument (line 89) ──────────────────────────────────────

#[test]
fn rand_with_a_float_bound_above_one_gives_an_integer() {
    let result = run("rand(3.14).is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_with_an_uncoercible_argument_raises_type_error() {
    let error = run_err(r#"rand("hello")"#);
    assert!(error.contains("no implicit conversion of String into Integer"));
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

// ── using error paths ───────────────────────────────────────────────────────

#[test]
fn using_no_args_errors() {
    let err = run_err("using");
    assert!(err.contains("argument") || err.contains("using") || err.contains("undefined"),);
}

#[test]
fn using_non_module_arg_errors() {
    let err = run_err("using(42)");
    assert!(err.contains("Module") || err.contains("type") || err.contains("TypeError"));
}

#[test]
fn using_inside_method_errors() {
    let err = run_err(
        r#"
module M
  refine(String) do
    def shout
      upcase + "!"
    end
  end
end
def foo
  using M
end
foo
"#,
    );
    assert!(err.contains("using") || err.contains("method") || err.contains("permitted"));
}

// ── top-level define_method error paths ─────────────────────────────────────

#[test]
fn top_level_define_method_no_args_errors() {
    let err = run_err("define_method()");
    assert!(err.contains("argument") || err.contains("define_method"));
}

#[test]
fn top_level_define_method_non_symbol_errors() {
    let err = run_err("define_method(42) { 1 }");
    assert!(err.contains("Symbol") || err.contains("String") || err.contains("type"));
}

#[test]
fn top_level_define_method_no_block_errors() {
    let err = run_err("define_method(:foo)");
    assert!(err.contains("block") || err.contains("define_method"));
}

// ── top-level private/public modifier ───────────────────────────────────────

#[test]
fn top_level_private_no_args() {
    let result = run("private()");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn top_level_public_no_args() {
    let result = run("public()");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn top_level_private_with_symbol() {
    let result = run(r#"
def foo
  42
end
private :foo
"#);
    assert!(matches!(result, Some(Object::Symbol(_))));
}

#[test]
fn top_level_private_multiple_symbols() {
    let result = run(r#"
def foo
  1
end
def bar
  2
end
private :foo, :bar
"#);
    assert!(matches!(result, Some(Object::Array(_))));
}

#[test]
fn top_level_private_undefined_method_errors() {
    let err = run_err("private :nonexistent_xyz");
    assert!(err.contains("undefined") || err.contains("nonexistent"));
}

#[test]
fn top_level_private_non_symbol_errors() {
    let err = run_err("private 42");
    assert!(err.contains("symbol") || err.contains("string") || err.contains("TypeError"));
}

// ── load function ───────────────────────────────────────────────────────────

#[test]
fn load_existing_file() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let code = format!(
        r#"load("{}/tests/_examples/basics/sum_literal.rb")"#,
        manifest_dir
    );
    let result = run(&code);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn load_nonexistent_file_errors() {
    let err = run_err(r#"load("nonexistent_file_xyz.rb")"#);
    assert!(err.contains("load") || err.contains("file"));
}

#[test]
fn load_non_string_arg_errors_cov() {
    let err = run_err("load(42)");
    assert!(err.contains("String") || err.contains("type"));
}

// ── require error paths ─────────────────────────────────────────────────────

#[test]
fn require_nonexistent_file_errors() {
    let err = run_err(r#"require "nonexistent_module_xyz_abc""#);
    assert!(err.contains("load") || err.contains("cannot") || err.contains("LoadError"));
}

// ── require_relative without file context errors ────────────────────────────

#[test]
fn require_relative_no_context_errors() {
    let err = run_err(r#"require_relative "foo""#);
    assert!(err.contains("require_relative") || err.contains("context") || err.contains("REPL"));
}

// ── Kernel#rand ──────────────────────────────────────────────────────────────

#[test]
fn rand_without_arguments_gives_a_float_below_one() {
    let result = run("value = rand\nvalue.is_a?(Float) && value >= 0.0 && value < 1.0");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_with_an_integer_bound_stays_below_it() {
    let result = run("1000.times.all? { |i| (0...100).include?(rand(100)) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_ignores_the_sign_of_its_bound() {
    let result = run("1000.times.all? { |i| (0...4).include?(rand(-4)) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_with_a_float_below_one_gives_a_float() {
    let result = run("rand(0.999).is_a?(Float)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_over_an_integer_range_gives_an_integer_inside_it() {
    let result =
        run("1000.times.all? { |i| x = rand(4...6); x.is_a?(Integer) && (4...6).include?(x) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_over_a_mixed_range_gives_a_float_inside_it() {
    let result =
        run("1000.times.all? { |i| x = rand(4...6.5); x.is_a?(Float) && (4...6.5).include?(x) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_over_a_backwards_range_is_nil() {
    let result = run("rand(1..0)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn rand_over_a_zero_width_integer_range_is_that_integer() {
    let result = run("rand(42..42)");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn rand_calls_to_int_on_its_argument() {
    let result = run(r#"
class Limit
  def to_int
    7
  end
end
1000.times.all? { |i| (0...7).include?(rand(Limit.new)) }
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:rand)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn srand_answers_the_previous_seed() {
    let result = run("srand(1)\nsrand(2).is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn the_same_seed_draws_the_same_sequence() {
    let result = run("srand(99)\nfirst = rand\nsrand(99)\nfirst == rand");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Numeric is a real ancestor ───────────────────────────────────────────────

#[test]
fn an_integer_is_a_numeric() {
    let result = run("5.is_a?(Numeric)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_float_is_a_numeric() {
    let result = run("0.5.is_a?(Numeric)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn float_reports_numeric_as_its_superclass() {
    let result = run("Float.superclass.name");
    assert_eq!(result.map(|o| o.to_string()), Some("Numeric".to_string()));
}

// ── Comparing an Integer against a Float ─────────────────────────────────────

#[test]
fn an_integer_range_includes_a_float_inside_it() {
    let result = run("(0...1).include?(0.38)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_range_with_one_float_side_includes_a_float() {
    let result = run("(3.5..6).include?(5.93)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn spaceship_compares_a_float_against_an_integer() {
    let result = run("[0.38 <=> 0, 5 <=> 5.5, 2 <=> 2.0].inspect");
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("[1, -1, 0]".to_string())
    );
}

// ── sprintf format coercion ──────────────────────────────────────────────────

#[test]
fn sprintf_converts_its_format_with_to_str() {
    let result = run(r#"
class Template
  def to_str
    "converted %s"
  end
end
sprintf(Template.new, "format")
"#);
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("converted format".to_string())
    );
}

#[test]
fn sprintf_raises_type_error_for_a_format_it_cannot_convert() {
    let error = run_err(r#"sprintf(42, "value")"#);
    assert!(error.contains("no implicit conversion of Integer into String"));
}

#[test]
fn a_numeric_modulo_by_a_string_raises() {
    let error = run_err(r#"42 % "not a format""#);
    assert!(error.contains("Cannot apply operator 'Modulo' to types 'Int' and 'String'"));
}

#[test]
fn percent_s_renders_a_symbol_with_to_s() {
    let result = run(r#"sprintf("%s", :symbol)"#);
    assert_eq!(result.map(|o| o.to_string()), Some("symbol".to_string()));
}

// ── Float constants ──────────────────────────────────────────────────────────

#[test]
fn float_infinity_is_larger_than_any_finite_value() {
    let result = run("Float::INFINITY > 1e308");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn float_nan_does_not_equal_itself() {
    let result = run("Float::NAN == Float::NAN");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn float_reports_its_precision_constants() {
    let result = run("[Float::DIG, Float::MANT_DIG].inspect");
    assert_eq!(result.map(|o| o.to_string()), Some("[15, 53]".to_string()));
}

#[test]
fn float_epsilon_and_bounds_are_present() {
    let result = run("[Float::EPSILON > 0, Float::MAX > Float::MIN].inspect");
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("[true, true]".to_string())
    );
}

// ── Kernel#srand ─────────────────────────────────────────────────────────────

#[test]
fn srand_answers_the_seed_it_replaced() {
    let result = run("srand(10)\nsrand(20)");
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn srand_accepts_a_seed_of_zero() {
    let result = run("srand(0)\nsrand");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn srand_accepts_a_negative_seed() {
    let result = run("srand(-17)\nsrand");
    assert_eq!(result, Some(Object::Int(-17)));
}

#[test]
fn srand_truncates_a_float_seed() {
    let result = run("srand(3.8)\nsrand");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn srand_calls_to_int_on_its_seed() {
    let result = run(r#"
class Seed
  def to_int
    7
  end
end
srand(Seed.new)
srand
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn srand_with_no_argument_picks_a_seed() {
    let result = run("srand.is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn the_same_seed_repeats_a_whole_sequence() {
    let result = run(r#"
srand(99)
first = 3.times.map { rand }
srand(99)
first == 3.times.map { rand }
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn srand_raises_type_error_for_nil() {
    let error = run_err("srand(nil)");
    assert!(error.contains("into Integer"));
}

#[test]
fn srand_raises_type_error_for_a_string() {
    let error = run_err(r#"srand("7")"#);
    assert!(error.contains("no implicit conversion of String into Integer"));
}

#[test]
fn srand_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:srand)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Behaviour the examples cannot reach ──────────────────────────────────────

#[test]
fn a_closed_handle_refuses_to_be_read() {
    let error = run_err(
        r#"
path = "/tmp/metorex_closed_handle_test.txt"
File.write(path, "abc\n")
handle = File.open(path)
handle.close
begin
  handle.getc
ensure
  File.delete(path)
end
"#,
    );
    assert!(
        error.contains("closed stream"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_closed_handle_still_says_what_it_was_opened_on() {
    let result = run(r#"
path = "/tmp/metorex_closed_handle_path.txt"
File.write(path, "abc\n")
handle = File.open(path)
handle.close
answer = [handle.closed?, handle.path, handle.to_io.equal?(handle)]
File.delete(path)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, /tmp/metorex_closed_handle_path.txt, true]".to_string())
    );
}

#[test]
fn reading_a_directory_is_refused_as_a_directory() {
    let error = run_err(r#"File.read("/tmp")"#);
    assert!(
        error.contains("Is a directory"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_missing_name_has_no_link_to_read() {
    let error = run_err(r#"File.readlink("/tmp/metorex_no_such_link_here")"#);
    assert!(
        error.contains("No such file or directory"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_thread_belongs_to_the_default_group_until_another_takes_it() {
    let result = run(r#"
group = ThreadGroup.new
before = Thread.main.group.equal?(ThreadGroup::Default)
group.add(Thread.main)
[before, Thread.main.group.equal?(group), group.list.include?(Thread.main)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true]".to_string())
    );
}

#[test]
fn an_enclosed_group_refuses_to_give_a_thread_up() {
    let error = run_err(
        r#"
held = ThreadGroup.new
held.add(Thread.main)
held.enclose
ThreadGroup.new.add(Thread.main)
"#,
    );
    assert!(
        error.contains("enclosed thread group"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_refinement_refuses_to_have_a_module_mixed_into_it() {
    let error = run_err(
        r#"
Module.new do
  refine String do
    include Module.new
  end
end
"#,
    );
    assert!(
        error.contains("Refinement#include has been removed"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn syscall_and_set_trace_func_are_private_kernel_methods() {
    let result = run(r#"
[Kernel.private_instance_methods(false).include?(:syscall),
 Kernel.private_instance_methods(false).include?(:set_trace_func)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_hash_subclass_keeps_its_class_through_merge() {
    let result = run(r#"
class MergeKeepsClass < Hash; end
held = MergeKeepsClass.new
held[1] = 2
merged = held.merge({ 3 => 4 })
[merged.class.name, merged[1], merged[3]]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[MergeKeepsClass, 2, 4]".to_string())
    );
}

#[test]
fn coerce_refuses_a_string_and_a_numeric_that_is_not_real() {
    let refused_string = run_err(r#"Complex(1, 0).coerce("20")"#);
    assert!(
        refused_string.contains("can't be coerced into Complex"),
        "unexpected error: {}",
        refused_string
    );
}

#[test]
fn mkfifo_reads_a_name_through_to_path() {
    let result = run(r#"
class FifoName
  def initialize(path)
    @path = path
  end
  def to_path
    @path
  end
end
path = "/tmp/metorex_fifo_to_path"
File.delete(path) if File.exist?(path)
File.mkfifo(FifoName.new(path))
answer = File.pipe?(path)
File.delete(path)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn mkfifo_refuses_a_name_it_cannot_read() {
    let error = run_err(r#"File.mkfifo(:"/tmp/metorex_fifo_symbol")"#);
    assert!(error.contains("String"), "unexpected error: {}", error);
}

#[test]
fn mkfifo_reports_a_directory_that_is_not_there() {
    let error = run_err(r#"File.mkfifo("/metorex_no_such_directory/fifo")"#);
    assert!(
        error.contains("No such file or directory"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn mkfifo_reports_a_directory_it_may_not_write() {
    // A directory with no write bit refuses the name, which is a different
    // refusal from a missing directory.
    let error = run_err(
        r#"
holder = "/tmp/metorex_fifo_unwritable"
Dir.mkdir(holder) unless Dir.exist?(holder)
File.chmod(0555, holder)
begin
  File.mkfifo(holder + "/fifo")
ensure
  File.chmod(0755, holder)
  Dir.rmdir(holder)
end
"#,
    );
    assert!(
        error.contains("Permission denied"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn coerce_refuses_a_numeric_that_says_it_is_not_real() {
    let error = run_err(
        r#"
class NotReal < Numeric
  def real?
    false
  end
end
Complex(1, 0).coerce(NotReal.new)
"#,
    );
    assert!(
        error.contains("can't be coerced into Complex"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn lgamma_grows_without_bound_at_infinity() {
    let result = run("Math.lgamma(Float::INFINITY)");
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[Infinity, 1]".to_string())
    );
}

#[test]
fn lgamma_approaches_the_pole_at_zero_from_either_side() {
    let result = run("[Math.lgamma(0.0)[1], Math.lgamma(-0.0)[1]]");
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, -1]".to_string())
    );
}

#[test]
fn waitall_takes_no_arguments() {
    let error = run_err("Process.waitall(0)");
    assert!(
        error.contains("wrong number of arguments"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn mkfifo_takes_the_mode_it_is_given() {
    let result = run(r#"
path = "/tmp/metorex_fifo_mode"
File.delete(path) if File.exist?(path)
File.mkfifo(path, 0644)
answer = File.pipe?(path)
File.delete(path)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn mkfifo_refuses_a_to_path_that_is_not_a_name() {
    let error = run_err(
        r#"
class NotAName
  def to_path
    42
  end
end
File.mkfifo(NotAName.new)
"#,
    );
    assert!(error.contains("String"), "unexpected error: {}", error);
}

#[test]
fn an_unknown_pack_directive_is_named_in_the_refusal() {
    let error = run_err(r#"[1].pack("K")"#);
    assert!(
        error.contains("unknown pack directive 'K'"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn an_unknown_unpack_directive_is_named_in_the_refusal() {
    let error = run_err(r#""abc".unpack("K")"#);
    assert!(
        error.contains("unknown unpack directive 'K'"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_width_modifier_is_refused_where_the_directive_has_no_platform_width() {
    let error = run_err(r#""abcdefgh".unpack("a!")"#);
    assert!(
        error.contains("unknown unpack directive '!'"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn packing_fewer_items_than_the_format_asks_for_is_refused() {
    let error = run_err(r#"[].pack("N")"#);
    assert!(
        error.contains("too few arguments"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn skipping_past_the_end_of_the_string_is_refused() {
    let error = run_err(r#""ab".unpack("x4C")"#);
    assert!(
        error.contains("outside of string"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn stepping_back_further_than_the_string_reaches_is_refused() {
    let error = run_err(r#""abcd".unpack("CX*C")"#);
    assert!(
        error.contains("outside of string"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_trace_reads_nothing_outside_a_handler() {
    let error = run_err("TracePoint.new(:line) {}.lineno");
    assert!(
        error.contains("access from outside"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_trace_refuses_an_event_it_does_not_know() {
    let error = run_err("TracePoint.new(:nowhere) {}");
    assert!(
        error.contains("unknown event"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_trace_needs_a_handler() {
    let error = run_err("TracePoint.new(:line)");
    assert!(
        error.contains("must be called with a block"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn allow_reentry_is_refused_outside_a_handler() {
    let error = run_err("TracePoint.allow_reentry { 1 }");
    assert!(
        error.contains("allow_reentry"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_trace_passes_over_the_core_library_it_runs_through() {
    // `upcase` is answered from the core library, whose statements a trace
    // never sees, so only the program's own lines are counted.
    let result = run(r#"
seen = 0
tracer = TracePoint.new(:line) { |point| seen += 1 }
tracer.enable
held = "quiet".upcase
tracer.disable
seen
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("1".to_string()));
}

// ── The encoding a string is tagged with ─────────────────────────────────────

#[test]
fn force_encoding_changes_what_a_string_says_it_is() {
    let result = run(r#"
held = "text"
before = held.encoding.name
held.force_encoding("EUC-JP")
[before, held.encoding.name, held]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[UTF-8, EUC-JP, text]".to_string())
    );
}

#[test]
fn every_reference_to_a_string_sees_the_encoding_it_was_given() {
    let result = run(r#"
held = "text"
alias_of_it = held
held.force_encoding("EUC-JP")
alias_of_it.encoding.name
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("EUC-JP".to_string())
    );
}

#[test]
fn packing_answers_a_run_of_bytes_and_an_empty_format_answers_ascii() {
    let result = run(r#"[[65].pack("C").encoding.name, [].pack("").encoding.name]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[ASCII-8BIT, US-ASCII]".to_string())
    );
}

#[test]
fn encode_tags_ascii_text_and_leaves_the_rest_alone() {
    // Text that is nothing but ASCII reads the same in every ASCII-compatible
    // encoding, so tagging it converts nothing. Text that is not needs a
    // conversion metorex does not carry out.
    let result = run(r#"
[ "plain".encode("US-ASCII").encoding.name,
  "é".encode("US-ASCII").encoding.name ]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[US-ASCII, UTF-8]".to_string())
    );
}

#[test]
fn two_strings_holding_the_same_text_are_two_strings() {
    let result = run(r#"
first = "same"
second = "same"
[first == second, first.equal?(second), first.equal?(first)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, false, true]".to_string())
    );
}

#[test]
fn a_string_answers_the_same_id_every_time_it_is_asked() {
    let result = run(r#"
held = "same"
other = "same"
[held.object_id == held.object_id, held.object_id == other.object_id]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, false]".to_string())
    );
}

#[test]
fn the_values_that_write_themselves_answer_one_unchanging_string() {
    let result = run(r#"
module NamedOnce; end
[nil.to_s.equal?(nil.to_s),
 true.to_s.equal?(true.to_s),
 false.to_s.equal?(false.to_s),
 :held.name.equal?(:held.name),
 NamedOnce.name.equal?(NamedOnce.name),
 :held.id2name.equal?(:held.id2name)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true, true, true, false]".to_string())
    );
}

#[test]
fn a_module_that_gains_a_name_answers_the_new_one() {
    let result = run(r#"
made = Module.new
before = made.name
Gained = made
[before, made.name]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[nil, Gained]".to_string())
    );
}

#[test]
fn a_tie_answers_the_one_that_came_first() {
    let result = run(r#"
first = "2"
second = "2"
held = [first, second]
[held.max_by { |value| value.to_i }.equal?(first),
 held.min_by { |value| value.to_i }.equal?(first)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_string_value_hashes_and_compares_by_the_text_it_holds() {
    // Two strings holding the same text are one hash key, even though they
    // are two objects.
    let result = run(r#"
held = {}
held["same"] = 1
held["same"] = 2
[held.size, held["same"], "same" == "same".dup]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, 2, true]".to_string())
    );
}

#[test]
fn a_wide_character_packs_as_the_bytes_its_encoding_needs() {
    let result = run(r#"[[960].pack("U").length, [960].pack("U").unpack("U")]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[2, [960]]".to_string())
    );
}
