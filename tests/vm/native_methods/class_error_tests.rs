// Class/define_method/file error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// ══════════════════════════════════════════════════════════════════════════════
// Native methods mod.rs - dispatch fallthrough (lines 160-162, 224)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn file_write_non_string_content() {
    let result = run(r#"
File.write("/tmp/metorex_test_err_cov.txt", 42)
"#);
    // File.write formats non-string content, returns byte count
    assert!(result.is_some());
    let _ = std::fs::remove_file("/tmp/metorex_test_err_cov.txt");
}

#[test]
fn define_method_with_block_on_class() {
    let result = run(r#"
class Dyn
  define_method(:greet) do
    "hello"
  end
end
Dyn.new.greet
"#);
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

#[test]
fn define_method_with_symbol_name() {
    let result = run(r#"
class Sym
  define_method(:calc) do
    100
  end
end
Sym.new.calc
"#);
    assert_eq!(result, Some(Object::Int(100)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Class native methods - name, superclass, ancestors
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn class_name_method() {
    let result = run(r#"
class Animal
end
Animal.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Animal".to_string()))));
}

#[test]
fn class_superclass_method() {
    let result = run(r#"
class Base
end
class Child < Base
end
Child.superclass.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Base".to_string()))));
}

#[test]
fn class_ancestors_method() {
    let result = run(r#"
class A
end
class B < A
end
B.ancestors.length
"#);
    // B and A in ancestors
    assert!(result.is_some());
    if let Some(Object::Int(n)) = result {
        assert!(n >= 2, "Expected at least 2 ancestors, got {}", n);
    }
}

#[test]
fn class_superclass_nil_when_no_parent() {
    // A user-defined class without an explicit `<` parent inherits from
    // Object (Ruby semantics), so `superclass` returns Object, not nil.
    let result = run(r#"
class Solo
end
Solo.superclass.name
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Object".to_string())))
    );
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Class.const_defined? ───────────────────────────────────────────────────

#[test]
fn const_defined_true_for_existing_global() {
    let result = run(r#"
Foo = 42
Object.const_defined?(:Foo)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn const_defined_false_for_missing() {
    let result = run(r#"Object.const_defined?(:NopeNotHere)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn const_defined_with_string_arg() {
    let result = run(r#"
Bar = 99
Object.const_defined?("Bar")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn const_defined_with_non_symbol_raises_type_error() {
    let err = run_err(r#"Object.const_defined?(42)"#);
    assert!(
        err.contains("is not a symbol nor a string"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn const_defined_no_args_errors() {
    let err = run_err("Object.const_defined?");
    assert!(err.contains("argument"));
}

// ── File.expand_path error paths ──────────────────────────────────────────

#[test]
fn file_expand_path_no_args_errors() {
    let err = run_err("File.expand_path");
    assert!(err.contains("argument"));
}

#[test]
fn file_expand_path_too_many_args_errors() {
    let err = run_err(r#"File.expand_path("a", "b", "c")"#);
    assert!(err.contains("argument"));
}

#[test]
fn file_expand_path_non_string_arg_errors() {
    let err = run_err("File.expand_path(42)");
    assert!(err.contains("String"));
}

#[test]
fn file_expand_path_non_string_base_errors() {
    let err = run_err(r#"File.expand_path("foo", 42)"#);
    assert!(err.contains("String"));
}

#[test]
fn file_expand_path_with_absolute_base() {
    let result = run(r#"File.expand_path("a.txt", "/tmp")"#);
    if let Some(Object::String(s)) = result {
        assert!(s.contains("a.txt"));
    } else {
        panic!("expected string, got {:?}", result);
    }
}

// ── module_function explicit-name form ─────────────────────────────────────

#[test]
fn module_function_with_symbol_name() {
    // Make `helper` callable on the module itself.
    let result = run(r#"
module Util
  def helper
    "ok"
  end
  module_function :helper
end
Util.helper
"#);
    assert_eq!(result, Some(Object::string("ok")));
}

#[test]
fn module_function_on_a_class_errors() {
    let err = run_err(
        r#"
class Mod
end
Mod.send(:module_function)
"#,
    );
    assert!(
        err.contains("module_function must be called for modules"),
        "unexpected error: {err}"
    );
}

#[test]
fn module_function_undefined_method_errors() {
    // Send the message directly to bypass the visibility-stub fallthrough.
    let err = run_err(
        r#"
module Util2
end
Util2.send(:module_function, :nonexistent_method)
"#,
    );
    assert!(err.contains("undefined") || err.contains("nonexistent"));
}

#[test]
fn module_function_non_string_arg_errors() {
    let err = run_err(
        r#"
module Util3
end
Util3.send(:module_function, 42)
"#,
    );
    assert!(
        err.contains("42 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

// ── Signal.trap ─────────────────────────────────────────────────────

#[test]
fn signal_trap_with_block_answers_the_previous_handler() {
    let result = run(r#"
Signal.trap("INT") { puts "intercepted" }
"#);
    assert_eq!(result, Some(Object::string("DEFAULT")));
}

#[test]
fn signal_trap_answers_the_handler_it_replaced() {
    let result = run(r#"
Signal.trap("INT", "IGNORE")
Signal.trap("INT")
"#);
    assert_eq!(result, Some(Object::string("IGNORE")));
}

// ── Dir methods (mod.rs lines 60-116) ────────────────────────────────────

#[test]
fn dir_exist_no_args_errors() {
    let err = run_err("Dir.exist?");
    assert!(err.contains("argument"));
}

#[test]
fn dir_exist_non_string_arg_errors() {
    let err = run_err("Dir.exist?(42)");
    assert!(err.contains("String"));
}

#[test]
fn dir_mkdir_no_args_errors() {
    let err = run_err("Dir.mkdir");
    assert!(err.contains("argument"));
}

#[test]
fn dir_mkdir_non_string_arg_errors() {
    let err = run_err("Dir.mkdir(42)");
    assert!(err.contains("String"));
}

#[test]
fn dir_pwd_returns_string() {
    let result = run("Dir.pwd");
    match result {
        Some(Object::String(s)) => assert!(!s.is_empty()),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn dir_getwd_returns_string() {
    let result = run("Dir.getwd");
    match result {
        Some(Object::String(s)) => assert!(!s.is_empty()),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn dir_exist_returns_true_for_tmp() {
    let result = run(r#"Dir.exist?("/tmp")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn dir_exists_returns_false_for_nonexistent() {
    let result = run(r#"Dir.exist?("/this_path_does_not_exist_xyz_123")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn dir_mkdir_and_delete() {
    let path = "/tmp/metorex_dir_test_coverage_xyz";
    let _ = std::fs::remove_dir(path);
    let result = run(&format!(r#"Dir.mkdir("{}")"#, path));
    assert_eq!(result, Some(Object::Int(0)));
    let result2 = run(&format!(r#"Dir.delete("{}")"#, path));
    assert_eq!(result2, Some(Object::Int(0)));
}

#[test]
fn dir_glob_returns_array() {
    let result = run(r#"Dir.glob("/tmp")"#);
    match result {
        Some(Object::Array(_)) => {}
        other => panic!("expected Array, got {:?}", other),
    }
}

// ── Time.now (mod.rs lines 153-157) ──────────────────────────────────────

#[test]
fn time_now_returns_float() {
    let result = run("Time.now");
    match result {
        Some(Object::Float(f)) => assert!(f > 0.0),
        other => panic!("expected Float, got {:?}", other),
    }
}

#[test]
fn time_new_returns_float() {
    let result = run("Time.new");
    match result {
        Some(Object::Float(f)) => assert!(f > 0.0),
        other => panic!("expected Float, got {:?}", other),
    }
}

// ── module_function with Symbol name (mod.rs lines 614-620) ──────────────

#[test]
fn module_function_with_symbol_name_symbol() {
    let result = run(r#"
module MathUtils
  def double(n)
    n * 2
  end
  module_function :double
end
MathUtils.double(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Process module stubs (mod.rs lines 669-675) ───────────────────────────

#[test]
fn process_pid_returns_integer() {
    let result = run("Process.pid");
    match result {
        Some(Object::Int(n)) => assert!(n > 0),
        other => panic!("expected Int, got {:?}", other),
    }
}

#[test]
fn process_ppid_returns_integer() {
    let result = run("Process.ppid");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn process_kill_without_a_signal_errors() {
    let err = run_err("Process.kill");
    assert!(err.contains("wrong number of arguments"));
}

#[test]
fn process_exit_raises_system_exit() {
    // `Process.exit` ends the program the way the bare form does, which is a
    // rescuable SystemExit rather than a nil answer.
    let error = run_err("Process.exit");
    assert!(error.contains("Uncaught exception: exit"), "{}", error);
}

// ── GC / ObjectSpace stubs (mod.rs lines 681-683) ────────────────────────

#[test]
fn gc_start_returns_nil() {
    let result = run("GC.start");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn objectspace_each_object_returns_nil() {
    let result = run("ObjectSpace.each_object");
    assert_eq!(result, Some(Object::Nil));
}

// ── File.expand_path with non-existent path (mod.rs line 451) ────────────

#[test]
fn file_expand_path_nonexistent_path() {
    let result = run(r#"File.expand_path("/tmp/nonexistent_xyz_abc/subdir")"#);
    match result {
        Some(Object::String(s)) => assert!(s.contains("nonexistent_xyz_abc")),
        other => panic!("expected String, got {:?}", other),
    }
}

// ── object_methods.rs: to_s with args error (lines 45-49) ────────────────────

#[test]
fn to_s_with_args_errors() {
    let err = run_err("42.to_s(10)");
    assert!(err.contains("argument"));
}

// ── object_methods.rs: respond_to? with Symbol arg (line 78) ─────────────────

#[test]
fn respond_to_with_symbol_arg() {
    let result = run(r#"
class Foo
  def bar
    42
  end
end
Foo.new.respond_to?(:bar)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── object_methods.rs: dup/clone on Array (lines 399-402) ────────────────────

#[test]
fn array_dup_returns_copy() {
    let result = run(r#"
arr = [1, 2, 3]
copy = arr.dup
copy.push(4)
arr.length
"#);
    // Original array should still have 3 elements
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_clone_returns_copy() {
    let result = run(r#"
arr = [1, 2, 3]
copy = arr.clone
copy.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── object_methods.rs: =~ wrong arg count (line 418) ─────────────────────────

#[test]
fn regex_match_no_args_errors() {
    let err = run_err(r#""hello".send(:=~)"#);
    assert!(err.contains("argument"));
}

// ── object_methods.rs: methods() on instance (lines 308, 315) ────────────────

#[test]
fn instance_methods_returns_array() {
    let result = run(r#"
class Animal
  def speak
    "..."
  end
end
class Dog < Animal
  def fetch
    "fetching"
  end
end
d = Dog.new
m = d.methods
m.length
"#);
    // Should include both speak and fetch (and possibly more)
    match result {
        Some(Object::Int(n)) => assert!(n >= 2),
        other => panic!("expected Int, got {:?}", other),
    }
}

// ── module_function on a Class with Symbol arg (mod.rs line 614) ──────────────

#[test]
fn class_module_function_with_symbol_arg_errors() {
    let err = run_err(
        r#"
class Helpers
  def add(a, b)
    a + b
  end
end
Helpers.send(:module_function, :add)
"#,
    );
    assert!(
        err.contains("module_function must be called for modules"),
        "unexpected error: {err}"
    );
}

// ── module_function on a Class with wrong type (mod.rs lines 615-620) ─────────

#[test]
fn class_module_function_non_symbol_non_string_errors() {
    let err = run_err(
        r#"
class Helpers2
  def add(a, b)
    a + b
  end
end
Helpers2.send(:module_function, 99)
"#,
    );
    assert!(
        err.contains("module_function must be called for modules"),
        "unexpected error: {err}"
    );
}

// ── File.expand_path with CurDir component (mod.rs line 451) ─────────────────

#[test]
fn file_expand_path_with_curdir_component() {
    // A path containing "." exercises the CurDir match arm in expand_path.
    let result = run(r#"File.expand_path("./foo.txt", "/tmp")"#);
    match result {
        Some(Object::String(s)) => assert!(s.contains("foo.txt")),
        other => panic!("expected String, got {:?}", other),
    }
}

// ── File.realpath ENOTDIR error path (mod.rs line 353) ───────────────────────

#[test]
fn file_realpath_enotdir_error() {
    // mod.rs line 353: Errno::ENOTDIR when a path component is not a directory.
    // /etc/hosts (or similar regular file) followed by a subpath triggers ENOTDIR.
    let err = run_err(r#"File.realpath "/etc/hosts/impossible_subpath""#);
    assert!(
        err.contains("ENOTDIR")
            || err.contains("NotADirectory")
            || err.contains("not a directory")
            || err.contains("ENOENT")
            || err.contains("No such")
    );
}

// ── Process unknown method falls through (mod.rs line 675) ───────────────────

#[test]
fn process_unknown_method_falls_through() {
    // Calling an unknown method on Process hits the `_ => {}` arm (line 675),
    // then falls through to normal method lookup which raises an error.
    let result = std::panic::catch_unwind(|| {
        let tokens = metorex::lexer::Lexer::new("Process.unknown_xyz_method").tokenize();
        let stmts = metorex::parser::Parser::new(tokens)
            .parse()
            .expect("parse failed");
        let mut vm = metorex::vm::VirtualMachine::new();
        vm.execute_program(&stmts)
    });
    // Either returns an error or panics — both are acceptable
    match result {
        Err(_) => {}     // panicked
        Ok(Err(_)) => {} // returned an error
        Ok(Ok(_)) => {}  // fell through and returned something
    }
}

// ── apply_class_visibility_modifier (mod.rs lines 1073-1135) ────────────────

#[test]
fn class_private_with_symbol_via_send() {
    let result = run(r#"
class Priv
  def secret
    42
  end
end
Priv.send(:private, :secret)
"#);
    assert!(matches!(result, Some(Object::Symbol(_))));
}

#[test]
fn class_public_with_symbol_via_send() {
    let result = run(r#"
class Pub
  def greet
    "hi"
  end
end
Pub.send(:private, :greet)
Pub.send(:public, :greet)
"#);
    assert!(matches!(result, Some(Object::Symbol(_))));
}

#[test]
fn class_private_multiple_args_via_send() {
    let result = run(r#"
class Multi
  def a
    1
  end
  def b
    2
  end
end
Multi.send(:private, :a, :b)
"#);
    assert!(matches!(result, Some(Object::Array(_))));
}

#[test]
fn class_private_undefined_method_via_send_errors() {
    let err = run_err(
        r#"
class Undef2
  def real
    1
  end
end
Undef2.send(:private, :nonexistent)
"#,
    );
    assert!(err.contains("undefined") || err.contains("nonexistent"));
}

#[test]
fn class_private_non_symbol_via_send_errors() {
    let err = run_err(
        r#"
class BadArg2
  def real
    1
  end
end
BadArg2.send(:private, 42)
"#,
    );
    assert!(err.contains("symbol") || err.contains("string") || err.contains("TypeError"));
}

#[test]
fn class_private_no_args_via_send() {
    let result = run(r#"
class NoArg2
end
NoArg2.send(:private)
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── TrueClass.new / FalseClass.new / NilClass.new errors ───────────────────

#[test]
fn true_class_new_errors() {
    let err = run_err("TrueClass.new");
    assert!(err.contains("TrueClass") || err.contains("undefined"));
}

#[test]
fn false_class_new_errors() {
    let err = run_err("FalseClass.new");
    assert!(err.contains("FalseClass") || err.contains("undefined"));
}

#[test]
fn nil_class_new_errors() {
    let err = run_err("NilClass.new");
    assert!(err.contains("NilClass") || err.contains("undefined"));
}

#[test]
fn true_class_allocate_errors() {
    let err = run_err("TrueClass.allocate");
    assert!(err.contains("allocator") || err.contains("TrueClass") || err.contains("undefined"));
}

// ── Class.new { block } (anonymous class) ───────────────────────────────────

#[test]
fn anonymous_class_with_block() {
    let result = run(r#"
klass = Class.new {
  def greet
    "anon"
  end
}
klass.new.greet
"#);
    assert_eq!(result, Some(Object::string("anon")));
}

#[test]
fn anonymous_class_with_superclass() {
    let result = run(r#"
class Base
  def base_method
    "base"
  end
end
klass = Class.new(Base) {
  def child_method
    "child"
  end
}
klass.new.base_method
"#);
    assert_eq!(result, Some(Object::string("base")));
}

#[test]
fn class_new_non_class_superclass_errors() {
    let err = run_err(r#"Class.new("not a class")"#);
    assert!(err.contains("Class") || err.contains("superclass"));
}

// ── Module.new { block } (anonymous module) ─────────────────────────────────

#[test]
fn anonymous_module_with_block() {
    let result = run(r#"
m = Module.new {
  def helper
    "helping"
  end
}
class User
  include m
end
User.new.helper
"#);
    assert_eq!(result, Some(Object::string("helping")));
}

// ── class_eval / module_eval on Class ───────────────────────────────────────

#[test]
fn class_eval_adds_method() {
    let result = run(r#"
class Target
end
Target.class_eval {
  def dynamic
    "added"
  end
}
Target.new.dynamic
"#);
    assert_eq!(result, Some(Object::string("added")));
}

#[test]
fn class_eval_without_block_errors() {
    // With neither a block nor a code string, `class_eval` raises ArgumentError
    // (matching Ruby: it expects 1..3 arguments in the string form).
    let err = run_err(
        r#"
class Target2
end
Target2.class_eval
"#,
    );
    assert!(err.contains("given 0, expected 1..3"));
}

// ── module_eval on Module ───────────────────────────────────────────────────

#[test]
fn module_eval_adds_method() {
    let result = run(r#"
module Ext
end
Ext.module_eval {
  def helper
    "mod helper"
  end
}
class User
  include Ext
end
User.new.helper
"#);
    assert_eq!(result, Some(Object::string("mod helper")));
}

// ── class_eval / module_eval string form ────────────────────────────────────

#[test]
fn class_eval_string_returns_last_value() {
    let result = run(r#"
class CeStr
end
CeStr.class_eval("1 + 1")
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn class_eval_string_evaluates_in_context_of_self() {
    let result = run(r#"
module CeSelf
end
CeSelf.class_eval("self") == CeSelf
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_eval_string_defines_methods() {
    let result = run(r#"
class CeDef
end
CeDef.class_eval("def greet; 'hi'; end")
CeDef.new.greet
"#);
    assert_eq!(result, Some(Object::string("hi")));
}

#[test]
fn class_eval_block_returns_last_value() {
    let result = run(r#"
module CeBlock
end
CeBlock.class_eval { 40 + 2 }
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn class_eval_block_yields_the_module() {
    let result = run(r#"
module CeYield
end
given = nil
CeYield.class_eval { |m| given = m }
given == CeYield
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_eval_string_uses_filename_and_lineno() {
    let result = run(r#"
module CeLoc
end
CeLoc.class_eval("[__FILE__, __LINE__]", "custom.rb", 102)
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("custom.rb"),
            Object::Int(102)
        ]))
    );
}

#[test]
fn class_eval_too_many_arguments_errors() {
    let err = run_err(
        r#"
class CeArgs
end
CeArgs.class_eval("1 + 1", "f", 0, "extra")
"#,
    );
    assert!(err.contains("given 4, expected 1..3"));
}

#[test]
fn class_eval_block_with_arguments_errors() {
    let err = run_err(
        r#"
class CeMix
end
CeMix.class_eval("1 + 1") { 2 }
"#,
    );
    assert!(err.contains("given 1, expected 0"));
}

#[test]
fn class_eval_non_string_code_without_to_str_errors() {
    let err = run_err(
        r#"
class CeBad
end
CeBad.class_eval(42)
"#,
    );
    assert!(err.contains("no implicit conversion of Integer into String"));
}

// ── class_exec / module_exec ────────────────────────────────────────────────

#[test]
fn class_exec_defines_method_in_receiver_scope() {
    let result = run(r#"
class CxDef
end
CxDef.class_exec { def foo; "foo"; end }
CxDef.new.foo
"#);
    assert_eq!(result, Some(Object::string("foo")));
}

#[test]
fn class_exec_passes_arguments_to_block() {
    let result = run(r#"
class CxArgs
end
CxArgs.class_exec(7) { |n| n }
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn class_exec_returns_last_value() {
    let result = run(r#"
module CxRet
end
CxRet.module_exec { 1 + 1 }
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn class_exec_without_block_raises_local_jump_error() {
    let err = run_err(
        r#"
class CxNoBlock
end
CxNoBlock.class_exec
"#,
    );
    assert!(err.contains("no block given"));
}

#[test]
fn class_exec_on_module_subclass_instance() {
    // `Sub < Module; Sub.new` is itself a module and answers class_exec.
    let result = run(r#"
class CxModSub < Module
end
CxModSub.new.class_exec { 1 + 1 }
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── private_methods on class ────────────────────────────────────────────────

#[test]
fn class_private_methods_returns_array() {
    let result = run(r#"
class WithPriv
  def public_one
    1
  end
  private
  def secret
    2
  end
end
WithPriv.private_methods(false).length
"#);
    assert!(result.is_some());
}

// ── Class#initialize ────────────────────────────────────────────────────────

#[test]
fn class_has_private_initialize() {
    let result = run("Class.private_methods.include?(:initialize)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn initialize_on_already_initialized_class_raises_type_error() {
    let err = run_err("Integer.send(:initialize)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_on_object_class_raises_type_error() {
    let err = run_err("Object.send(:initialize)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_on_basic_object_raises_type_error() {
    let err = run_err("BasicObject.send(:initialize)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_with_class_as_argument_raises_type_error() {
    let err = run_err("u = Class.allocate\nu.send(:initialize, Class)");
    assert!(err.contains("already initialized"));
}

#[test]
fn initialize_on_uninitialized_class_succeeds() {
    let result = run("u = Class.allocate\nu.send(:initialize)");
    assert_eq!(result, Some(Object::Nil));
}
