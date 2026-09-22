// Visibility set at the top level rather than in a class body.

use super::*;

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
