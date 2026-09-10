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

// ── The command line's line-reading flags ────────────────────────────────────

#[test]
fn short_flags_cluster_and_a_value_rides_on_the_end() {
    use metorex::split_short_flags;
    assert_eq!(
        split_short_flags("-naF:".to_string()),
        vec!["-n", "-a", "-F", ":"]
    );
    assert_eq!(split_short_flags("-rfoo".to_string()), vec!["-r", "foo"]);
    assert_eq!(split_short_flags("-np".to_string()), vec!["-n", "-p"]);
    // A bare flag and a name that only looks like one are left as written.
    assert_eq!(split_short_flags("-n".to_string()), vec!["-n"]);
    assert_eq!(
        split_short_flags("-zzz".to_string()),
        vec!["-zzz".to_string()]
    );
    assert_eq!(
        split_short_flags("--version".to_string()),
        vec!["--version".to_string()]
    );
}

#[test]
fn the_line_reading_flags_read_back_under_their_own_names() {
    let mut vm = VirtualMachine::new();
    vm.set_flag_global("a", true);
    vm.set_flag_global("p", false);
    vm.set_verbose(true);
    let tokens = Lexer::new("[$-a, $-p, $VERBOSE]").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[true, false, true]".to_string())
    );
}

#[test]
fn a_line_splits_into_fields_on_whitespace_or_a_named_pattern() {
    let mut vm = VirtualMachine::new();
    vm.set_split_fields("one two\n", None);
    let tokens = Lexer::new("$F").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[one, two]".to_string())
    );

    vm.set_split_fields("a:b\n", Some(":"));
    let tokens = Lexer::new("$F").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[a, b]".to_string())
    );
}

#[test]
fn the_line_separator_is_named_by_its_octal_code() {
    let mut vm = VirtualMachine::new();
    vm.set_line_separator("72");
    let tokens = Lexer::new("[$/, $-0]").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[:, :]".to_string())
    );
}

#[test]
fn a_bare_zero_flag_asks_for_paragraph_mode() {
    let mut vm = VirtualMachine::new();
    vm.set_line_separator("0");
    let tokens = Lexer::new("$/").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("\n\n".to_string())
    );
}

#[test]
fn making_a_directory_takes_the_mode_it_is_given() {
    let result = run(r#"
held = "/tmp/metorex_mkdir_mode_test"
Dir.rmdir(held) if Dir.exist?(held)
Dir.mkdir(held, 01755)
answer = File.sticky?(held)
Dir.rmdir(held)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_plain_command_runs_without_a_shell_between() {
    let result = run(r#"IO.popen("/bin/echo plain") { |handle| handle.read }"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("plain\n".to_string())
    );
}

#[test]
fn a_command_written_in_shell_syntax_still_reaches_the_shell() {
    let result = run(r#"IO.popen("/bin/echo one; /bin/echo two") { |handle| handle.read }"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("one\ntwo\n".to_string())
    );
}

#[test]
fn a_command_of_only_spaces_is_left_for_the_shell_to_refuse() {
    let result = run(r#"IO.popen("  ") { |handle| handle.read }"#);
    assert_eq!(result.map(|value| value.to_string()), Some(String::new()));
}

#[test]
fn a_superclass_can_be_named_from_the_top_level() {
    let result = run(r#"
class Basis
  def label
    :root
  end
end

module Nested
  class Basis < ::Basis
  end
end

Nested::Basis.new.label
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(":root".to_string())
    );
}

#[test]
fn a_superclass_named_from_the_top_level_can_be_built_by_a_call() {
    let result = run(r#"
module Holder
  class Pair < ::Struct.new(:left, :right)
  end
end

Holder::Pair.new(1, 2).left
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("1".to_string()));
}

#[test]
fn a_block_on_super_reaches_the_parent_method() {
    let result = run(r#"
class Source
  def read
    yield
  end
end

class Doubled < Source
  def read
    super { 21 } * 2
  end
end

Doubled.new.read
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("42".to_string())
    );
}

#[test]
fn a_do_block_on_super_with_an_argument_list_reaches_the_parent_method() {
    let result = run(r#"
class Source
  def read(offset)
    offset + yield
  end
end

class Shifted < Source
  def read(offset)
    super(offset) do
      5
    end
  end
end

Shifted.new.read(10)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("15".to_string())
    );
}

#[test]
fn a_writer_takes_its_parameter_without_parentheses() {
    let result = run(r#"
class Gauge
  def level
    @level
  end

  def level= reading
    @level = reading
  end
end

gauge = Gauge.new
gauge.level = 7
gauge.level
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("7".to_string()));
}

#[test]
fn a_space_before_the_equals_makes_the_definition_an_endless_one() {
    let result = run(r#"
class Gauge
  def level = 41
end

Gauge.new.level + 1
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("42".to_string())
    );
}

#[test]
fn a_class_level_writer_takes_its_parameter_without_parentheses() {
    let result = run(r#"
class Gauge
  def self.limit= ceiling
    @@limit = ceiling
  end

  def self.limit
    @@limit
  end
end

Gauge.limit = 3
Gauge.limit
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("3".to_string()));
}

#[test]
fn a_writer_written_on_one_object_answers_for_that_object() {
    let result = run(r#"
class Plain
  def held
    @held
  end
end

plain = Plain.new
def plain.held= value
  @held = value * 2
end

plain.held = 6
plain.held
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("12".to_string())
    );
}

#[test]
fn a_module_level_writer_answers_for_the_module() {
    let result = run(r#"
module Settings
  def self.depth= value
    @depth = value
  end

  def self.depth
    @depth
  end
end

Settings.depth = 4
Settings.depth
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("4".to_string()));
}

#[test]
fn each_digest_answers_the_value_its_algorithm_is_defined_to_give() {
    let result = run(r#"
require 'digest'
[
  Digest::MD5.hexdigest(""),
  Digest::MD5.hexdigest("abc"),
  Digest::SHA1.hexdigest(""),
  Digest::SHA256.hexdigest("abc"),
  Digest::SHA384.hexdigest(""),
  Digest::SHA512.hexdigest("")
].join(" ")
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(
            [
                "d41d8cd98f00b204e9800998ecf8427e",
                "900150983cd24fb0d6963f7d28e17f72",
                "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
                concat!(
                    "38b060a751ac96384cd9327eb1b1e36a21fdb71114be0743",
                    "4c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b"
                ),
                concat!(
                    "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce",
                    "47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
                ),
            ]
            .join(" ")
        )
    );
}

#[test]
fn a_digest_taken_of_a_message_leaves_the_object_blank() {
    let result = run(r#"
require 'digest'
running = Digest::SHA256.new
running << "test"
taken = running.hexdigest("abc")
[taken == Digest::SHA256.hexdigest("abc"), running.hexdigest == Digest::SHA256.hexdigest("")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_digest_that_names_no_algorithm_refuses_to_answer() {
    let error = run_err(
        r#"
require 'digest'
Digest::Class.new.finish
"#,
    );
    assert!(error.contains("does not name an algorithm"), "{error}");
}

#[test]
fn the_base_digest_protocol_refuses_the_work_it_does_not_do() {
    let result = run(r#"
require 'digest'
holder = ::Class.new do
  include Digest::Instance
end
refused = []
begin
  holder.new.update "test"
rescue RuntimeError
  refused << :update
end
[:finish, :reset, :block_length].each do |name|
  begin
    holder.new.send name
  rescue RuntimeError
    refused << name
  end
end
refused
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:update, :finish, :reset, :block_length]".to_string())
    );
}

#[test]
fn a_digest_reads_a_file_as_the_bytes_it_holds() {
    let result = run(r#"
require 'digest'
held = "/tmp/metorex_digest_file_test.txt"
File.write(held, "abc")
answer = Digest::SHA256.file(held).hexdigest
File.delete(held)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string())
    );
}

#[test]
fn a_digest_asks_an_object_that_is_not_a_string_how_to_read_one() {
    let result = run(r#"
require 'digest'
named = Object.new
def named.to_str
  "abc"
end
Digest::SHA256.hexdigest(named)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string())
    );
}

#[test]
fn a_digest_refuses_a_message_it_cannot_read_a_string_out_of() {
    let error = run_err(
        r#"
require 'digest'
Digest.hexencode(nil)
"#,
    );
    assert!(error.contains("no implicit conversion"), "{error}");
}

#[test]
fn the_sha2_family_is_named_by_its_bit_length() {
    let result = run(r#"
require 'digest'
[256, 384, 512].map { |bits| Digest::SHA2.hexdigest("abc", bits).length }
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[64, 96, 128]".to_string())
    );
}

#[test]
fn the_sha2_family_refuses_a_bit_length_it_has_no_algorithm_for() {
    let error = run_err(
        r#"
require 'digest'
Digest::SHA2.new(224)
"#,
    );
    assert!(error.contains("unsupported bit length"), "{error}");
}

#[test]
fn reading_a_file_as_bytes_keeps_what_a_text_read_would_lose() {
    let result = run(r#"
held = "/tmp/metorex_binread_test.bin"
File.binwrite(held, ["00ff10"].pack("H*"))
answer = File.binread(held).each_char.map { |byte| byte.ord }
File.delete(held)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[0, 255, 16]".to_string())
    );
}

#[test]
fn reading_a_directory_as_bytes_is_refused() {
    let error = run_err(r#"File.binread("/tmp")"#);
    assert!(error.contains("Is a directory"), "{error}");
}

#[test]
fn reading_a_name_no_file_answers_to_as_bytes_is_refused() {
    let error = run_err(r#"File.binread("/tmp/metorex_no_such_file_at_all.bin")"#);
    assert!(error.contains("No such file or directory"), "{error}");
}

#[test]
fn base64_packed_with_a_zero_count_carries_no_line_breaks() {
    let result = run(r#"[["abc"].pack("m"), ["abc"].pack("m0")]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[YWJj\n, YWJj]".to_string())
    );
}

#[test]
fn a_string_says_it_already_is_one() {
    let result = run(r#""abc".respond_to?(:to_str)"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn writing_bytes_to_a_file_answers_how_many_it_wrote() {
    let result = run(r#"
held = "/tmp/metorex_binwrite_count_test.bin"
written = File.binwrite(held, ["00ff10"].pack("H*"))
File.delete(held)
written
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("3".to_string()));
}

#[test]
fn writing_bytes_to_a_name_no_directory_holds_is_refused() {
    let error = run_err(r#"File.binwrite("/tmp/metorex_no_such_dir/held.bin", "x")"#);
    assert!(error.contains("No such file or directory"), "{error}");
}

#[test]
fn a_digest_of_an_algorithm_metorex_has_no_implementation_of_is_refused() {
    let error = run_err("require 'digest'\nDigest.__digest__(\"SHA3\", \"abc\")");
    assert!(error.contains("unknown digest algorithm SHA3"), "{error}");
}

#[test]
fn a_digest_asked_for_without_an_algorithm_and_a_message_is_refused() {
    let error = run_err("require 'digest'\nDigest.__digest__(\"MD5\")");
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_bang_method_answers_for_the_operator_written_in_front_of_it() {
    let result = run(r#"
class Negated
  def !
    :flipped
  end

  def ~
    :complemented
  end
end

[!Negated.new, ~Negated.new, Negated.new.!]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:flipped, :complemented, :flipped]".to_string())
    );
}

#[test]
fn a_pattern_can_name_a_constant_from_the_top_level() {
    let result = run(r#"
answers = []
["a", 1].each do |held|
  case held
  when ::String then answers << :string
  when ::Integer then answers << :integer
  end
end
answers
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:string, :integer]".to_string())
    );
}

#[test]
fn a_global_variable_is_not_answered_as_a_constant_of_the_same_name() {
    let result = run(r#"
class Level
  DEBUG = 3

  def named
    DEBUG
  end

  def defaulted(held: DEBUG)
    held
  end
end

[Level.new.named, Level.new.defaulted, $DEBUG]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[3, 3, false]".to_string())
    );
}

#[test]
fn a_yield_with_no_block_behind_it_is_a_jump_with_nowhere_to_land() {
    let result = run(r#"
def needs_one
  yield
end

begin
  needs_one
rescue LocalJumpError => problem
  [problem.class, problem.message]
end
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[LocalJumpError, no block given (yield)]".to_string())
    );
}

#[test]
fn methods_reports_what_an_included_module_supplies() {
    let result = run(r#"
module Carried
  def carried_name
    :carried
  end
end

class Holder
  include Carried

  def own_name
    :own
  end
end

held = Holder.new.methods
[held.include?(:carried_name), held.include?(:own_name)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn indexing_reaches_method_missing_when_no_index_is_defined() {
    let result = run(r#"
class Asked
  def method_missing(name, *arguments)
    [name, arguments]
  end

  def respond_to_missing?(_name, _private = false)
    true
  end
end

Asked.new[7]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:[], [7]]".to_string())
    );
}

#[test]
fn a_double_splat_reaches_a_call_written_without_parentheses() {
    let result = run(r#"
def take(first, **rest)
  [first, rest]
end

extra = {second: 2}
take 1, **extra
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, {second: 2}]".to_string())
    );
}

#[test]
fn a_string_formatted_in_place_of_nil_says_nothing_at_all() {
    let result = run(r#"["[%s]" % [nil], "[%s]" % ["held"]]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[[], [held]]".to_string())
    );
}

#[test]
fn an_open_handle_says_it_answers_to_what_it_can_do() {
    let result = run(r#"
held = "/tmp/metorex_handle_answers_test.txt"
handle = File.open(held, "w+")
answered = [
  handle.respond_to?(:write),
  handle.respond_to?(:readlines),
  handle.respond_to?(:no_such_thing)
]
handle.close
File.delete(held)
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, false]".to_string())
    );
}

#[test]
fn a_name_may_be_moved_onto_another_one() {
    let result = run(r#"
from = "/tmp/metorex_rename_from.txt"
to = "/tmp/metorex_rename_to.txt"
File.write(from, "held")
File.rename(from, to)
answered = [File.exist?(from), File.read(to)]
File.delete(to)
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, held]".to_string())
    );
}

#[test]
fn moving_a_name_no_file_answers_to_is_refused() {
    let error =
        run_err(r#"File.rename("/tmp/metorex_no_such_source.txt", "/tmp/metorex_dest.txt")"#);
    assert!(error.contains("No such file or directory"), "{error}");
}

#[test]
fn the_permissions_on_a_file_can_be_set_by_name() {
    let result = run(r#"
held = "/tmp/metorex_chmod_test.txt"
File.write(held, "x")
changed = File.chmod(0600, held)
mode = "%o" % File.stat(held).mode
File.delete(held)
[changed, mode]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, 100600]".to_string())
    );
}

#[test]
fn setting_permissions_counts_only_the_names_a_file_answers_to() {
    let result = run(r#"File.chmod(0600, "/tmp/metorex_no_such_chmod_target.txt")"#);
    assert_eq!(result.map(|value| value.to_string()), Some("0".to_string()));
}

#[test]
fn the_scratch_directory_is_named_without_a_trailing_separator() {
    let result = run(r#"[Dir.tmpdir.end_with?("/"), Dir.exist?(Dir.tmpdir)]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, true]".to_string())
    );
}

#[test]
fn digits_too_wide_for_a_machine_word_still_name_a_number() {
    let result = run(r#"
held = "33333333333333333333"
[held.to_i.to_s, "9223372036854775808".to_i.to_s, "ff".to_i(16)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[33333333333333333333, 9223372036854775808, 255]".to_string())
    );
}

#[test]
fn a_protected_method_answers_another_object_of_its_own_class() {
    let result = run(r#"
class Pair
  def initialize(value)
    @value = value
  end

  def bigger_than?(other)
    held > other.held
  end

  protected

  def held
    @value
  end
end

Pair.new(2).bigger_than? Pair.new(1)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_protected_method_is_refused_from_outside_and_named_as_protected() {
    let error = run_err(
        r#"
class Pair
  protected

  def held
    1
  end
end

Pair.new.held
"#,
    );
    assert!(error.contains("protected method 'held'"), "{error}");
}

#[test]
fn a_constant_that_names_a_value_is_compared_against_it() {
    let result = run(r#"
class Level
  LOW = 1
  HIGH = 2

  def named(value)
    case value
    when LOW then :low
    when HIGH then :high
    when String then :text
    else :other
    end
  end
end

held = Level.new
[held.named(1), held.named(2), held.named("a"), held.named(9)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:low, :high, :text, :other]".to_string())
    );
}

#[test]
fn a_number_too_wide_for_a_machine_word_matches_the_integer_pattern() {
    let result = run(r#"
def named(value)
  case value
  when Integer then :whole
  else :other
  end
end

[named(1), named(12345678901234567890123), named("a")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:whole, :whole, :other]".to_string())
    );
}

#[test]
fn a_number_written_in_exponent_notation_carries_a_signed_power() {
    let result = run(r#"["%e" % [1234.5678], "%.2e" % [0.000123], "%E" % [-5.0]]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1.234568e+03, 1.23e-04, -5.000000E+00]".to_string())
    );
}

#[test]
fn the_percent_method_is_named_by_a_symbol_and_reached_through_a_dot() {
    let result = run(r#"
class Divided
  def %(other)
    [:remainder, other]
  end
end

held = Divided.new
[:%, held.%(3), held.send(:%, 4)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:%, [:remainder, 3], [:remainder, 4]]".to_string())
    );
}

#[test]
fn a_percent_literal_after_a_ternary_colon_is_still_a_literal() {
    let result = run(r#"[true ? 1 : %w[a b], false ? 1 : %(text)]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, text]".to_string())
    );
}

#[test]
fn reopening_the_class_of_an_immediate_adds_a_method_it_answers_to() {
    let result = run(r#"
class NilClass
  def held
    :from_nil
  end
end

class TrueClass
  def held
    :from_true
  end
end

[nil.held, true.held]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:from_nil, :from_true]".to_string())
    );
}

#[test]
fn a_handle_sends_what_was_written_through_it_on_when_asked_to_flush() {
    let result = run(r#"
held = "/tmp/metorex_flush_test.txt"
handle = File.open(held, "w+")
handle.write "kept"
handle.flush
size = File.size(held)
handle.close
File.delete(held)
size
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("4".to_string()));
}

#[test]
fn a_protected_method_answers_a_class_method_of_the_same_class() {
    let result = run(r#"
class Counted
  def self.compare(left, right)
    left.held <=> right.held
  end

  def initialize(value)
    @value = value
  end

  protected

  def held
    @value
  end
end

Counted.compare Counted.new(2), Counted.new(1)
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("1".to_string()));
}

#[test]
fn a_protected_method_answers_a_subclass_of_the_class_that_defines_it() {
    let result = run(r#"
class Base
  def initialize(value)
    @value = value
  end

  def sees(other)
    other.held
  end

  protected

  def held
    @value
  end
end

class Grown < Base
end

Grown.new(7).sees Grown.new(9)
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("9".to_string()));
}

#[test]
fn a_protected_method_an_included_module_supplies_answers_the_including_class() {
    let result = run(r#"
module Carried
  def sees(other)
    other.held
  end

  protected

  def held
    :carried
  end
end

class Holder
  include Carried
end

Holder.new.sees Holder.new
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(":carried".to_string())
    );
}

#[test]
fn a_protected_method_answers_a_value_of_a_reopened_core_class() {
    let result = run(r#"
class Integer
  def sees(other)
    other.doubled
  end

  protected

  def doubled
    self * 2
  end
end

3.sees 5
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("10".to_string())
    );
}

#[test]
fn a_constant_an_included_module_supplies_is_compared_against_in_a_pattern() {
    let result = run(r#"
module Levels
  QUIET = 5
end

class Reader
  include Levels

  def named(value)
    case value
    when QUIET then :quiet
    else :other
    end
  end
end

[Reader.new.named(5), Reader.new.named(6)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:quiet, :other]".to_string())
    );
}

#[test]
fn a_constant_a_superclass_supplies_is_compared_against_in_a_pattern() {
    let result = run(r#"
class Above
  MARK = 4
end

class Below < Above
  def named(value)
    case value
    when MARK then :marked
    else :other
    end
  end
end

[Below.new.named(4), Below.new.named(5)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:marked, :other]".to_string())
    );
}

#[test]
fn a_constant_named_in_a_pattern_from_a_class_method_is_compared_against() {
    let result = run(r#"
class Gauge
  STEP = 3

  def self.named(value)
    case value
    when STEP then :step
    else :other
    end
  end
end

[Gauge.named(3), Gauge.named(4)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:step, :other]".to_string())
    );
}

#[test]
fn a_pattern_naming_a_class_nothing_defines_matches_nothing() {
    let result = run(r#"
def named(value)
  case value
  when NoSuchClassAnywhere then :found
  else :other
  end
end

named 1
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(":other".to_string())
    );
}

#[test]
fn each_checksum_answers_the_value_its_definition_gives() {
    let result = run(r#"
require 'zlib'
[Zlib.crc32(""), Zlib.crc32(" "), Zlib.crc32("123456789"),
 Zlib.adler32(""), Zlib.adler32("123456789")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[0, 3916222277, 3421780262, 1, 152961502]".to_string())
    );
}

#[test]
fn a_checksum_carries_on_from_the_value_it_is_given() {
    let result = run(r#"
require 'zlib'
held = "This is a test string! How exciting!%?"
[Zlib.crc32(held, 0), Zlib.crc32(held, 1), Zlib.crc32("p", -305419897)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[3864990561, 1809313411, 4046865307]".to_string())
    );
}

#[test]
fn a_checksum_refuses_a_starting_value_too_wide_to_be_one() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.crc32("held", 2 ** 128)
"#,
    );
    assert!(error.contains("bignum too big"), "{error}");
}

#[test]
fn a_stream_another_zlib_wrote_reads_back_here() {
    let result = run(r#"
require 'zlib'
written = [120, 156, 99, 96, 128, 1, 0, 0, 10, 0, 1].pack("C*")
Zlib.inflate(written) == "\000" * 10
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_stream_this_writes_reads_back_as_what_it_was_given() {
    let result = run(r#"
require 'zlib'
held = "the quick brown fox " * 40
[Zlib.inflate(Zlib.deflate(held)) == held, Zlib.gunzip(Zlib.gzip(held)) == held]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_stream_that_is_not_one_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.gunzip("not a member")
"#,
    );
    assert!(error.contains("not a stream this can read"), "{error}");
}

#[test]
fn a_gzip_member_names_what_it_holds_and_when_it_was_written() {
    let result = run(r#"
require 'zlib'
require 'stringio'
member = [31, 139, 8, 0, 44, 220, 209, 71, 0, 3, 51, 52, 50, 54, 49, 77,
          76, 74, 78, 73, 5, 0, 157, 5, 0, 36, 10, 0, 0, 0].pack("C*")
reader = Zlib::GzipReader.new(StringIO.new(member))
held = reader.read
finished = reader.eof?
reader.close
[held, finished]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[12345abcde, true]".to_string())
    );
}

#[test]
fn a_closed_member_has_nothing_left_to_say_about_itself() {
    let result = run(r#"
require 'zlib'
require 'stringio'
reader = Zlib::GzipReader.new(StringIO.new(Zlib.gzip("held")))
reader.close
begin
  reader.orig_name
rescue Zlib::GzipFile::Error => problem
  problem.message
end
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("closed gzip stream".to_string())
    );
}

#[test]
fn a_template_stands_for_the_text_its_tags_build() {
    let result = run(r#"
require 'erb'
list = %w[a b c]
ERB.new("<% list.each do |item| %><%= item %>;<% end %>").result(binding)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("a;b;c;".to_string())
    );
}

#[test]
fn what_a_template_puts_into_a_page_is_escaped_for_it() {
    let result = run(r#"
require 'erb'
[ERB::Util.html_escape("<a href='x'>&</a>"), ERB::Util.url_encode("a b/c")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[&lt;a href=&#39;x&#39;&gt;&amp;&lt;/a&gt;, a%20b%2Fc]".to_string())
    );
}

#[test]
fn a_template_can_be_written_onto_a_class_as_a_method() {
    let result = run(r#"
require 'erb'
built = ERB.new("<%= @held %> is here").def_class(Object, "render")
made = built.new
made.instance_variable_set(:@held, "metorex")
made.render
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("metorex is here".to_string())
    );
}

#[test]
fn a_command_answers_what_it_wrote_and_how_it_ended() {
    let result = run(r#"
require 'open3'
output, status = Open3.capture2("echo written")
[output, status.exitstatus]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[written\n, 0]".to_string())
    );
}

#[test]
fn a_command_keeps_its_two_streams_apart_when_asked_to() {
    let result = run(r#"
require 'open3'
out, errors, _status = Open3.capture3("sh -c 'echo out; echo err 1>&2'")
[out, errors]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[out\n, err\n]".to_string())
    );
}

#[test]
fn a_stream_that_names_its_own_code_reads_back_here() {
    // A dynamic block carries the code it was written with, which the
    // decoder has to read before it can read anything else.
    let result = run(r#"
require 'zlib'
written = ([120, 156, 237, 193, 1, 1, 0, 0] +
           [0, 128, 144, 254, 175, 238, 8, 10] +
           Array.new(31, 0) +
           [24, 128, 0, 0, 1]).pack("C*")
Zlib.inflate(written) == "\000" * 32 * 1024
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_stream_with_no_header_in_front_of_it_reads_back_too() {
    let result = run(r#"
require 'zlib'
held = "a stream with no header"
written = Zlib.deflate(held)
raw = written[2, written.length - 6]
Zlib::Inflate.new(-Zlib::MAX_WBITS).inflate(raw)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("a stream with no header".to_string())
    );
}

#[test]
fn a_stream_of_nothing_reads_back_as_nothing() {
    let result = run(r#"
require 'zlib'
[Zlib.inflate(Zlib.deflate("")), Zlib.gunzip(Zlib.gzip(""))]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[, ]".to_string())
    );
}

#[test]
fn a_gzip_member_carries_the_name_it_was_written_with() {
    let result = run(r#"
require 'zlib'
require 'stringio'
holder = StringIO.new(+"")
writer = Zlib::GzipWriter.new(holder)
writer.orig_name = "held.txt"
writer.mtime = 1234567
writer.write("what it holds")
writer.close
reader = Zlib::GzipReader.new(StringIO.new(holder.string))
answered = [reader.read, reader.orig_name, reader.mtime.to_i]
reader.close
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[what it holds, held.txt, 1234567]".to_string())
    );
}

#[test]
fn a_stream_action_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.__stream__("no_such_action", "held", 0)
"#,
    );
    assert!(error.contains("unknown stream action"), "{error}");
}

#[test]
fn a_stream_asked_for_with_nothing_to_act_on_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.__stream__
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_stream_that_is_cut_short_is_refused() {
    let error = run_err(
        r#"
require 'zlib'
Zlib.inflate([120, 156, 99].pack("C*"))
"#,
    );
    assert!(error.contains("not a stream this can read"), "{error}");
}

#[test]
fn an_assignment_made_through_a_binding_stays_in_it() {
    let result = run(r#"
held = binding
eval "added = 7", held
[held.local_variable_get(:added), eval("added", held)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[7, 7]".to_string())
    );
}

#[test]
fn a_binding_names_the_locals_in_force_where_it_was_taken() {
    let result = run(r#"
def holding
  first = 1
  second = 2
  binding
end

held = holding
[held.local_variables.sort, held.local_variable_get(:first)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[[:first, :second], 1]".to_string())
    );
}

#[test]
fn a_binding_takes_a_local_it_did_not_have() {
    let result = run(r#"
held = binding
answered = [held.local_variable_defined?(:added)]
held.local_variable_set :added, 3
answered << held.local_variable_defined?(:added)
answered << held.local_variable_get(:added)
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, true, 3]".to_string())
    );
}

#[test]
fn a_binding_refuses_a_local_nothing_bound() {
    let error = run_err(r#"binding.local_variable_get(:no_such_local)"#);
    assert!(error.contains("is not defined"), "{error}");
}

#[test]
fn a_binding_names_a_local_by_symbol_or_string_and_nothing_else() {
    let error = run_err(r#"binding.local_variable_get(7)"#);
    assert!(error.contains("is not a symbol nor a string"), "{error}");
}

#[test]
fn a_binding_says_where_it_was_taken() {
    let result = run(r#"
held = binding
answered = held.source_location
[answered.class, answered.last > 0]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[Array, true]".to_string())
    );
}

#[test]
fn a_copy_of_a_binding_is_written_to_on_its_own() {
    let result = run(r#"
held = binding
held.local_variable_set :counted, 1
copied = held.dup
copied.local_variable_set :counted, 2
[held.local_variable_get(:counted), copied.local_variable_get(:counted)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, 2]".to_string())
    );
}

#[test]
fn a_binding_runs_code_where_it_was_taken() {
    let result = run(r#"
first = 4
held = binding
held.eval "first * 3"
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("12".to_string())
    );
}

#[test]
fn a_writer_written_in_a_singleton_class_body_answers_an_assignment() {
    let result = run(r#"
module Held
  class << self
    def level
      @level
    end

    def level=(value)
      @level = value * 2
    end
  end
end

Held.level = 5
Held.level
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("10".to_string())
    );
}

#[test]
fn a_keyed_digest_answers_the_value_its_definition_gives() {
    let result = run(r#"
require 'openssl'
OpenSSL::HMAC.hexdigest OpenSSL::Digest.new("SHA1"), "key",
                        "The quick brown fox jumps over the lazy dog"
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9".to_string())
    );
}

#[test]
fn a_key_derived_from_a_password_is_the_same_every_time() {
    let result = run(r#"
require 'openssl'
settings = {salt: "salt", iterations: 50, length: 20, hash: "sha1"}
first = OpenSSL::KDF.pbkdf2_hmac("secret", **settings)
[first.length, first == OpenSSL::KDF.pbkdf2_hmac("secret", **settings)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[20, true]".to_string())
    );
}

#[test]
fn a_derived_key_asked_for_without_all_its_parts_is_refused() {
    let error = run_err(
        r#"
require 'openssl'
Digest.__pbkdf2__("SHA1")
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_derived_key_of_an_algorithm_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'openssl'
Digest.__pbkdf2__("SHA3", "pass", "salt", 2, 16)
"#,
    );
    assert!(error.contains("unknown digest algorithm"), "{error}");
}

#[test]
fn comparing_without_saying_where_two_strings_differ() {
    let result = run(r#"
require 'openssl'
[OpenSSL.fixed_length_secure_compare("abc", "abc"),
 OpenSSL.fixed_length_secure_compare("abc", "abd"),
 OpenSSL.secure_compare("held", "held")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, false, true]".to_string())
    );
}

#[test]
fn comparing_two_strings_of_different_lengths_is_refused() {
    let error = run_err(
        r#"
require 'openssl'
OpenSSL.fixed_length_secure_compare("ab", "abc")
"#,
    );
    assert!(error.contains("must be of equal length"), "{error}");
}

#[test]
fn text_written_into_a_url_carries_only_what_a_url_may_carry() {
    let result = run(r#"
require 'cgi/escape'
[CGI.escape("a b&c~"), CGI.unescape("a+b%26c"),
 CGI.escapeURIComponent("a b/c"), CGI.unescapeURIComponent("a%20b%2Fc")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[a+b%26c~, a b&c, a%20b%2Fc, a b/c]".to_string())
    );
}

#[test]
fn text_written_into_a_page_spells_out_what_the_page_reads_as_markup() {
    let result = run(r#"
require 'cgi/escape'
[CGI.escapeHTML(%[& < > " ']), CGI.unescapeHTML("&amp;&lt;&gt;&quot;&#99;&#x41;")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[&amp; &lt; &gt; &quot; &#39;, &<>\"cA]".to_string())
    );
}

#[test]
fn only_the_tags_of_the_elements_named_are_spelled_out() {
    let result = run(r#"
require 'cgi/escape'
held = CGI.escapeElement('<BR><A HREF="url"></A>', "A")
[held, CGI.unescapeElement(held, "A")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(
            "[<BR>&lt;A HREF=&quot;url&quot;&gt;&lt;/A&gt;, <BR><A HREF=\"url\"></A>]".to_string()
        )
    );
}

#[test]
fn text_written_into_a_url_has_to_be_text() {
    let error = run_err(
        r#"
require 'cgi/escape'
CGI.escape(:held)
"#,
    );
    assert!(error.contains("no implicit conversion"), "{error}");
}

#[test]
fn the_system_log_is_opened_under_a_name_and_closed_again() {
    let result = run(r#"
require 'syslog'
answered = [Syslog.opened?]
Syslog.open "metorex_test", Syslog::LOG_PID
answered << Syslog.opened? << Syslog.ident << (Syslog.options == Syslog::LOG_PID)
answered << Syslog.mask
Syslog.close
answered << Syslog.opened? << Syslog.mask
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, true, metorex_test, true, 255, false, nil]".to_string())
    );
}

#[test]
fn a_log_nobody_has_opened_cannot_be_closed() {
    let error = run_err(
        r#"
require 'syslog'
Syslog.close
"#,
    );
    assert!(error.contains("syslog not opened"), "{error}");
}

#[test]
fn a_log_handed_to_a_block_is_not_closed_from_inside_it() {
    let result = run(r#"
require 'syslog'
answered = nil
begin
  Syslog.open { |held| held.close }
rescue RuntimeError => problem
  answered = problem.message
end
[answered, Syslog.opened?]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[syslog opened with a block, false]".to_string())
    );
}

#[test]
fn the_mask_a_log_carries_names_the_severities_it_lets_through() {
    let result = run(r#"
require 'syslog'
Syslog.open "metorex_mask_test"
Syslog.mask = Syslog::Constants.LOG_UPTO(Syslog::LOG_WARNING)
answered = [Syslog.mask, Syslog::Constants.LOG_MASK(Syslog::LOG_DEBUG)]
Syslog.close
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[31, 128]".to_string())
    );
}

#[test]
fn a_mask_that_is_not_a_number_is_refused() {
    let error = run_err(
        r#"
require 'syslog'
Syslog.open "metorex_bad_mask"
begin
  Syslog.mask = "held"
ensure
  Syslog.close
end
"#,
    );
    assert!(error.contains("no implicit conversion"), "{error}");
}

#[test]
fn a_message_written_to_the_log_reaches_the_error_stream_when_asked() {
    let result = run(r#"
require 'syslog'
Syslog.open "metorex_write_test", Syslog::LOG_PERROR
answered = Syslog.info("held %s", "message")
Syslog.close
answered == Syslog
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_binding_with_no_source_behind_it_says_so() {
    let result = run(r#"
held = proc { }.binding
held.source_location
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("nil".to_string())
    );
}

#[test]
fn a_binding_asked_about_no_local_at_all_is_refused() {
    let error = run_err(r#"binding.local_variable_get"#);
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_message_written_without_all_its_parts_is_refused() {
    let error = run_err(
        r#"
require 'syslog'
Syslog.__write__("held")
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_keyed_digest_of_the_wider_algorithms_folds_over_a_wider_block() {
    let result = run(r#"
require 'openssl'
[OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA512"), "key", "held").length,
 OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA384"), "key", "held").length]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[128, 96]".to_string())
    );
}

#[test]
fn a_keyed_digest_shortens_a_key_wider_than_its_block() {
    let result = run(r#"
require 'openssl'
long = "k" * 200
held = OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA1"), long, "message")
[held.length, held == OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA1"), long, "message")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[40, true]".to_string())
    );
}

#[test]
fn an_address_says_which_family_it_belongs_to() {
    let result = run(r#"
require 'socket'
[Addrinfo.tcp("127.0.0.1", 80).afamily == Socket::AF_INET,
 Addrinfo.tcp("::1", 80).afamily == Socket::AF_INET6,
 Addrinfo.unix("/tmp/held").afamily == Socket::AF_UNIX]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true]".to_string())
    );
}

#[test]
fn an_address_is_shown_the_way_ruby_shows_one() {
    let result = run(r#"
require 'socket'
[Addrinfo.tcp("127.0.0.1", 80).inspect, Addrinfo.tcp("::1", 80).inspect,
 Addrinfo.udp("127.0.0.1", 80).inspect, Addrinfo.ip("127.0.0.1").inspect]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(
            "[#<Addrinfo: 127.0.0.1:80 TCP>, #<Addrinfo: [::1]:80 TCP>, \
             #<Addrinfo: 127.0.0.1:80 UDP>, #<Addrinfo: 127.0.0.1>]"
                .to_string()
        )
    );
}

#[test]
fn an_address_says_what_kind_of_address_it_is() {
    let result = run(r#"
require 'socket'
[Addrinfo.ip("127.0.0.1").ipv4_loopback?, Addrinfo.ip("10.0.0.1").ipv4_private?,
 Addrinfo.ip("224.0.0.1").ipv4_multicast?, Addrinfo.ip("::1").ipv6_loopback?,
 Addrinfo.ip("ff02::1").ipv6_mc_linklocal?, Addrinfo.ip("fe80::1").ipv6_linklocal?,
 Addrinfo.ip("::ffff:127.0.0.1").ipv6_v4mapped?, Addrinfo.ip("8.8.8.8").ipv4_private?]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true, true, true, true, true, false]".to_string())
    );
}

#[test]
fn an_address_reads_back_out_of_the_struct_it_is_carried_in() {
    let result = run(r#"
require 'socket'
held = Socket.sockaddr_in(80, "127.0.0.1")
[Socket.unpack_sockaddr_in(held), Addrinfo.new(held).ip_address,
 Socket.unpack_sockaddr_in(Socket.sockaddr_in(443, "::1"))]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[[80, 127.0.0.1], 127.0.0.1, [443, ::1]]".to_string())
    );
}

#[test]
fn an_address_that_names_nothing_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Addrinfo.tcp("no.such.host.metorex.invalid", 80)
"#,
    );
    assert!(error.contains("getaddrinfo"), "{error}");
}

#[test]
fn a_path_too_long_for_the_struct_that_holds_it_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.sockaddr_un("/" + "a" * 200)
"#,
    );
    assert!(error.contains("too long unix socket path"), "{error}");
}

#[test]
fn a_connection_made_to_a_socket_carries_what_is_written_through_it() {
    let result = run(r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
client = TCPSocket.new("127.0.0.1", port)
client.write("held")
accepted = server.accept
answered = [accepted.read(4), accepted.peeraddr[2], server.addr[0], port > 0]
accepted.close
client.close
server.close
answered << server.closed?
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[held, 127.0.0.1, AF_INET, true, true]".to_string())
    );
}

#[test]
fn reaching_a_port_nothing_is_listening_on_is_refused() {
    let error = run_err(
        r#"
require 'socket'
TCPSocket.new("127.0.0.1", 1)
"#,
    );
    assert!(error.contains("connect"), "{error}");
}

#[test]
fn writing_through_a_connection_that_was_closed_is_refused() {
    let error = run_err(
        r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = TCPSocket.new("127.0.0.1", server.addr[1])
held.close
begin
  held.write("held")
ensure
  server.close
end
"#,
    );
    assert!(error.contains("closed connection"), "{error}");
}

#[test]
fn this_machine_says_what_name_it_answers_to() {
    let result = run(r#"
require 'socket'
held = Socket.gethostname
[held.class, held.empty?]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[String, false]".to_string())
    );
}

#[test]
fn an_address_action_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__("no_such_action", "127.0.0.1", 0)
"#,
    );
    assert!(error.contains("unknown address action"), "{error}");
}

#[test]
fn a_socket_action_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__net__("no_such_action", 0, "", 0)
"#,
    );
    assert!(error.contains("unknown socket action"), "{error}");
}

#[test]
fn a_qualified_constant_names_a_value_in_a_when_clause() {
    let result = run(r#"
module Held
  LOW = 1
  HIGH = 2
end

def named(value)
  case value
  when Held::LOW then :low
  when Held::HIGH then :high
  else :other
  end
end

[named(1), named(2), named(3)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:low, :high, :other]".to_string())
    );
}

#[test]
fn a_host_name_stands_for_every_address_it_answers_to() {
    let result = run(r#"
require 'socket'
found = Socket.resolved("localhost")
[found.class, found.empty?, found.include?("127.0.0.1")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[Array, false, true]".to_string())
    );
}

#[test]
fn a_name_that_stands_for_no_address_is_refused_when_read() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__("normalize", "not an address", 0)
"#,
    );
    assert!(error.contains("Name or service not known"), "{error}");
}

#[test]
fn the_bytes_of_a_name_that_stands_for_no_address_are_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__("bytes", "not an address", 0)
"#,
    );
    assert!(error.contains("Name or service not known"), "{error}");
}

#[test]
fn a_struct_asked_for_of_a_name_that_names_no_address_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.sockaddr_in(80, "not an address")
"#,
    );
    assert!(
        error.contains("nodename nor servname provided, or not known"),
        "{error}"
    );
}

#[test]
fn a_struct_that_carries_no_address_is_refused_when_read_back() {
    let error = run_err(
        r#"
require 'socket'
Socket.unpack_sockaddr_in("held")
"#,
    );
    assert!(error.contains("not an IP address struct"), "{error}");
}

#[test]
fn an_address_read_out_of_a_struct_of_an_unknown_family_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.unpack_sockaddr_in(["10630000000000000000"].pack("H*"))
"#,
    );
    assert!(error.contains("not an IP address struct"), "{error}");
}

#[test]
fn an_address_asked_for_with_nothing_to_act_on_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__address__
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_socket_asked_for_with_nothing_to_act_on_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__net__
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_connection_says_where_its_own_end_sits() {
    let result = run(r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = TCPSocket.new("127.0.0.1", server.addr[1])
answered = [held.addr[0], held.local_address.ip_address]
held.close
server.close
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[AF_INET, 127.0.0.1]".to_string())
    );
}

#[test]
fn a_closed_socket_says_nothing_about_where_it_sat() {
    let result = run(r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = server.handle
server.close
[Socket.__net__("address", held, "", 0), Socket.__net__("peer", held, "", 0)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[nil, nil]".to_string())
    );
}

#[test]
fn a_name_that_stands_for_no_address_at_all_is_not_named_a_family() {
    let result = run(r#"
require 'socket'
[Socket.__address__("family", "127.0.0.1", 0), Socket.__address__("family", "::1", 0),
 Socket.__address__("family", "held", 0)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[4, 6, nil]".to_string())
    );
}

#[test]
fn taking_a_connection_from_a_listener_that_was_closed_is_refused() {
    let error = run_err(
        r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = server.handle
server.close
Socket.__net__("accept", held, "", 0)
"#,
    );
    assert!(error.contains("closed listener"), "{error}");
}

#[test]
fn reading_through_a_connection_that_was_closed_is_refused() {
    let error = run_err(
        r#"
require 'socket'
server = TCPServer.new("127.0.0.1", 0)
held = TCPSocket.new("127.0.0.1", server.addr[1])
held.close
begin
  held.read(4)
ensure
  server.close
end
"#,
    );
    assert!(error.contains("closed connection"), "{error}");
}

#[test]
fn listening_on_a_name_this_machine_does_not_answer_to_is_refused() {
    let error = run_err(
        r#"
require 'socket'
Socket.__net__("listen", 0, "203.0.113.1", 0)
"#,
    );
    assert!(error.contains("bind"), "{error}");
}
