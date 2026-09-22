// What a refused constant read or write reports.

use super::*;
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
        Some(Object::String(s)) => assert!(!s.as_str().is_empty()),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn dir_getwd_returns_string() {
    let result = run("Dir.getwd");
    match result {
        Some(Object::String(s)) => assert!(!s.as_str().is_empty()),
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

// ── Time.now ─────────────────────────────────────────────────────────────

#[test]
fn time_now_answers_a_time() {
    let result = run("Time.now.class.name");
    assert_eq!(result, Some(Object::string("Time")));
}

#[test]
fn time_new_without_arguments_answers_the_current_time() {
    let result = run("Time.new.to_i > 1600000000");
    assert_eq!(result, Some(Object::Bool(true)));
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
    // Every process has a parent, so the id is a positive number rather than
    // the zero a stub would answer.
    let result = run("Process.ppid");
    assert!(
        matches!(result, Some(Object::Int(parent)) if parent > 0),
        "unexpected parent process id: {:?}",
        result
    );
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
        Some(Object::String(s)) => assert!(s.as_str().contains("nonexistent_xyz_abc")),
        other => panic!("expected String, got {:?}", other),
    }
}

// ── object_methods.rs: to_s with args error (lines 45-49) ────────────────────

#[test]
fn to_s_with_too_many_args_errors() {
    let err = run_err("42.to_s(10, 2)");
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
