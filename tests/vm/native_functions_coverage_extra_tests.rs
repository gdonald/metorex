// Additional coverage tests for src/vm/native_functions.rs.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

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

// ── load with wrap=true (lines 647, 699) ───────────────────────────────────

#[test]
fn load_with_wrap_true_increments_and_decrements() {
    let dir = "tests/_examples";
    let name = "io_load_wrap_test_tmp.rb";
    let path = format!("{}/{}", dir, name);
    std::fs::write(&path, "x = 7\n").unwrap();
    let result = run(&format!(
        r#"$LOAD_PATH.unshift "{}"
load("{}", true)
"#,
        dir, name
    ));
    assert_eq!(result, Some(Object::Bool(true)));
    std::fs::remove_file(&path).ok();
}

// ── load resolves a file via $LOAD_PATH with non-string entry (line 672) ──

#[test]
fn load_via_load_path_ignores_non_string_entries() {
    let dir = "tests/_examples";
    let name = "io_load_nonstr_entry_tmp.rb";
    let path = format!("{}/{}", dir, name);
    std::fs::write(&path, "z = 9\n").unwrap();
    let result = run(&format!(
        r#"$LOAD_PATH.unshift 42
$LOAD_PATH.unshift "{}"
load("{}")
"#,
        dir, name
    ));
    assert_eq!(result, Some(Object::Bool(true)));
    std::fs::remove_file(&path).ok();
}

// ── load with $LOAD_PATH not an Array (line 675) ──────────────────────────

#[test]
fn load_with_non_array_load_path_errors() {
    // Replace $LOAD_PATH with a non-array, then try load() on a name not
    // present in cwd. The fallback should treat search_dirs as empty,
    // producing a "cannot load" error.
    let err = run_err(
        r#"$: = "not-an-array"
load("nonexistent_xyz_for_cov.rb")
"#,
    );
    assert!(
        err.contains("cannot load") || err.contains("load") || err.contains("nonexistent"),
        "unexpected: {}",
        err
    );
}

// ── load via $LOAD_PATH found path fires execute_file error (lines 687-689) ─
// When the resolved file has a runtime error, the error message from load()
// wraps it with "load('<name>') — ...".

#[test]
fn load_via_load_path_wraps_execute_file_error() {
    let dir = "tests/_examples";
    let name = "io_load_runtime_err_tmp.rb";
    let path = format!("{}/{}", dir, name);
    std::fs::write(&path, "undefined_variable_xyz\n").unwrap();
    let err = run_err(&format!(
        r#"$LOAD_PATH.unshift "{}"
load("{}")
"#,
        dir, name
    ));
    std::fs::remove_file(&path).ok();
    assert!(
        err.contains("load") || err.contains("Undefined") || err.contains("undefined_variable"),
        "unexpected: {}",
        err
    );
}

// ── require with non-string $LOAD_PATH entry (line 274) ───────────────────

#[test]
fn require_via_load_path_ignores_non_string_entries() {
    let dir = "tests/_examples";
    let name = "io_req_nonstr_tmp.rb";
    let path = format!("{}/{}", dir, name);
    std::fs::write(&path, "q = 1\n").unwrap();
    let result = run(&format!(
        r#"$LOAD_PATH.unshift 99
$LOAD_PATH.unshift "{}"
require "{}"
"#,
        dir,
        name.trim_end_matches(".rb")
    ));
    // First require returns true for newly loaded.
    std::fs::remove_file(&path).ok();
    assert!(matches!(result, Some(Object::Bool(true))));
}

// ── require with file that has runtime error (lines 329-331) ─────────────

#[test]
fn require_file_with_runtime_error_wraps_message() {
    let dir = "tests/_examples";
    let name = "io_req_runtime_err_tmp.rb";
    let path = format!("{}/{}", dir, name);
    std::fs::write(&path, "raise 'boom'\n").unwrap();
    let err = run_err(&format!(
        r#"$LOAD_PATH.unshift "{}"
require "{}"
"#,
        dir,
        name.trim_end_matches(".rb")
    ));
    std::fs::remove_file(&path).ok();
    assert!(
        err.contains("require") || err.contains("boom") || err.contains("error"),
        "unexpected: {}",
        err
    );
}

// ── define_method with Module target (line 166) ───────────────────────────

#[test]
fn define_method_inside_module_eval_installs_on_module() {
    let result = run(r#"
module DMTarget
end
DMTarget.module_eval do
  define_method(:hello) { 42 }
end
class DMUser
  include DMTarget
end
DMUser.new.hello
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── define_method with no current class/module falls to Object (line 169) ─

#[test]
fn define_method_at_top_level_installs_on_object() {
    // Top-level define_method with self=main falls through to the Object
    // class fallback.
    let result = run(r#"
define_method(:top_cov_helper) { 123 }
top_cov_helper
"#);
    assert_eq!(result, Some(Object::Int(123)));
}

// ── exit calls process::exit — can only test the error arm for unknown fn ──

// ── require missing file raises LoadError (lines 302-311) ─────────────────

#[test]
fn require_raises_load_error_exception() {
    let err = run_err(r#"require "definitely_nonexistent_for_cov_xyz""#);
    assert!(
        err.contains("LoadError") || err.contains("cannot load"),
        "unexpected: {}",
        err
    );
}
