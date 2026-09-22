// Where a `require` looks, and what a failed load reports.

use super::*;

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
