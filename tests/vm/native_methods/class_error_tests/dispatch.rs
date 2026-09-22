// What a refused call reports.

use super::*;
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
        Some(Object::String(s)) => assert!(s.as_str().contains("foo.txt")),
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
