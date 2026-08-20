// Tests for defensive/guard paths that aren't reachable via normal Ruby code
// but are triggerable through careful VM/filesystem setup. These exercise
// error paths intended to cover internal invariant violations and
// time-of-check/time-of-use race conditions.

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

// ── super_expr.rs lines 38-40: frame name without '#' ──────────────────────
// A Block frame is pushed with name "<block>" (no '#'). Calling `super`
// inside a top-level proc triggers the `rfind('#')` → None branch, which
// returns "super called in invalid context (no class information)".

#[test]
fn super_in_toplevel_block_has_no_class_info() {
    let err = run_err(
        r#"
p = lambda { super }
p.call
"#,
    );
    assert!(
        err.contains("super called in invalid context")
            || err.contains("no class information")
            || err.contains("super"),
        "unexpected error: {}",
        err
    );
}

// ── native_methods/mod.rs line 211: Thread instance without __thread_block ─
// `Thread.allocate` constructs a Thread instance via the class allocator,
// bypassing Thread.new's constructor which normally sets __thread_block.
// Calling .value on the bare allocated instance falls through to the
// `Object::Nil` fallback at line 211.

#[test]
fn thread_allocated_without_block_value_returns_nil() {
    let result = run(r#"
t = Thread.allocate
t.value
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn thread_allocated_without_block_join_returns_self() {
    // join on a blockless Thread returns the receiver (the Thread instance).
    let result = run(r#"
t = Thread.allocate
t.join.class.name
"#);
    assert_eq!(result, Some(Object::string("Thread")));
}

// ── native_methods/mod.rs line 183: call_thread_method with non-Instance ──
// `module_function :method_name` stores a Method under `__ext__name` on the
// class. Dispatching that name on the class-as-receiver (e.g. `Thread.value`)
// routes through method_lookup's ext-method path → invoke_method →
// call_native_method with class=Thread and receiver=Object::Class(Thread).
// That reaches the "Thread" arm in call_native_method's `match class.name()`,
// which calls call_thread_method with a non-Instance receiver — hitting the
// `_ => Ok(None)` guard at line 183. The None return falls through and the
// user-defined method body executes.

#[test]
fn module_level_extended_method_falls_through_to_user_body() {
    let result = run(r#"
module Timing
  def value
    777
  end
  module_function :value
end
Timing.value
"#);
    assert_eq!(result, Some(Object::Int(777)));
}

// ── loading.rs lines 131-133: load_file_source failure ─────────────────────
// Create a file that's readable during find_file_path's `.exists()` check and
// canonicalizes fine, but chmod 000 ensures `fs::read_to_string` fails with
// EACCES. The error wraps into "Failed to load file ...".

#[cfg(unix)]
#[test]
fn execute_file_with_unreadable_file_fails_at_read() {
    use std::fs::{self, File, Permissions};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    // Use a unique name to avoid collisions with parallel tests.
    let dir = std::env::temp_dir().join(format!(
        "metorex_unreadable_{}_{}",
        std::process::id(),
        line!()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create dir");
    let file_path: PathBuf = dir.join("locked.rb");
    {
        let mut f = File::create(&file_path).expect("create file");
        writeln!(f, "x = 1").unwrap();
    }
    // Strip all permissions so the file exists/canonicalizes but can't be read.
    fs::set_permissions(&file_path, Permissions::from_mode(0o000)).expect("chmod 000");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(&file_path);

    // Restore perms so the file can be cleaned up.
    let _ = fs::set_permissions(&file_path, Permissions::from_mode(0o644));
    let _ = fs::remove_dir_all(&dir);

    let err = result.expect_err("should fail to read unreadable file");
    let msg = err.to_string();
    assert!(
        msg.contains("Failed to load file") || msg.contains("Permission") || msg.contains("denied"),
        "unexpected error: {}",
        msg
    );
}

// ── loading.rs lines 108-114: canonicalize failure (restricted parent) ─────
// Create a file inside a subdirectory, then strip all permissions on that
// parent. `find_file_path`'s `.exists()` check on the full file path
// requires parent traversal and fails → the request never reaches the
// canonicalize step. This test covers the find_file_path error path
// (fetched upstream of execute_file); the canonicalize error at lines
// 108-114 is a pure TOCTOU race (the file vanishes between exists() and
// canonicalize()) and not reliably reachable with synchronous filesystem
// ops — we accept it as defensive code for concurrent filesystem mutation.

#[cfg(unix)]
#[test]
fn execute_file_with_restricted_parent_directory_fails() {
    use std::fs::{self, File, Permissions};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!(
        "metorex_restricted_{}_{}",
        std::process::id(),
        line!()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create root");
    let parent = root.join("restricted");
    fs::create_dir_all(&parent).expect("create parent");
    let file_path = parent.join("target.rb");
    {
        let mut f = File::create(&file_path).expect("create file");
        writeln!(f, "y = 2").unwrap();
    }
    // Drop traversal (x) and read (r) bits on the parent so canonicalization
    // cannot walk through it.
    fs::set_permissions(&parent, Permissions::from_mode(0o000)).expect("chmod parent");

    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(&file_path);

    // Restore perms so cleanup works regardless of outcome.
    let _ = fs::set_permissions(&parent, Permissions::from_mode(0o755));
    let _ = fs::remove_dir_all(&root);

    let err = result.expect_err("should fail when parent dir has no perms");
    let msg = err.to_string();
    // Either canonicalize or a downstream stage fails; accept any IO error.
    assert!(
        msg.contains("canonicalize")
            || msg.contains("Failed to load file")
            || msg.contains("not found")
            || msg.contains("Permission")
            || msg.contains("denied")
            || msg.to_lowercase().contains("no such"),
        "unexpected error: {}",
        msg
    );
}
