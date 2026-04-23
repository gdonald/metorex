// Targeted coverage tests for uncovered lines in src/vm/loading.rs.

use metorex::object::Object;
use metorex::vm::VirtualMachine;
use std::path::Path;
use std::rc::Rc;

// ── require_library: happy path via $LOAD_PATH ──────────────────────────────

#[test]
fn require_library_resolves_via_load_path() {
    let mut vm = VirtualMachine::new();
    vm.prepend_load_path("tests/_examples/require/lib".to_string());

    let result = vm.require_library("helper");
    assert!(result.is_ok(), "require_library failed: {:?}", result);
}

// ── require_library: nonexistent file (lines 73-81) ─────────────────────────

#[test]
fn require_library_nonexistent_file_errors() {
    let mut vm = VirtualMachine::new();
    vm.prepend_load_path("tests/_examples/require/lib".to_string());
    let err = vm.require_library("nonexistent_xyz_abc").unwrap_err();
    assert!(err.to_string().contains("cannot load"));
}

// ── require_library: $LOAD_PATH with non-string element (line 51) ───────────

#[test]
fn require_library_ignores_non_string_load_path_entries() {
    let mut vm = VirtualMachine::new();
    // Inject a non-string into $LOAD_PATH so filter_map's `_ => None`
    // branch (line 51) is exercised.
    if let Some(Object::Array(arr)) = vm.globals().get(":") {
        arr.borrow_mut().insert(0, Object::Int(42));
    }
    vm.prepend_load_path("tests/_examples/require/lib".to_string());
    let result = vm.require_library("helper");
    assert!(result.is_ok(), "require_library failed: {:?}", result);
}

// ── require_library: $LOAD_PATH is not an Array (line 54) ───────────────────

#[test]
fn require_library_with_non_array_load_path_errors() {
    let mut vm = VirtualMachine::new();
    // Overwrite $LOAD_PATH so it's not an Array.
    vm.globals_mut().set(
        ":".to_string(),
        Object::String(Rc::new("not-array".to_string())),
    );
    let err = vm.require_library("helper").unwrap_err();
    assert!(err.to_string().contains("cannot load"));
}

// ── require_library: execute_file fails (lines 84-86) ───────────────────────

#[test]
fn require_library_propagates_execute_file_errors() {
    let mut vm = VirtualMachine::new();
    vm.prepend_load_path("tests/_examples/require".to_string());
    // `bad_runtime.rb` raises a runtime error when executed.
    let err = vm.require_library("bad_runtime").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("require") || msg.contains("bad_runtime") || msg.contains("error"));
}

// ── execute_file: find_file_path error path (line 104) ──────────────────────

#[test]
fn execute_file_nonexistent_path_errors() {
    let mut vm = VirtualMachine::new();
    let err = vm
        .execute_file(Path::new("/nonexistent_xyz_abc_path.rb"))
        .unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("No such"));
}

// ── prepend_load_path: $LOAD_PATH present ───────────────────────────────────

#[test]
fn prepend_load_path_adds_to_front() {
    let mut vm = VirtualMachine::new();
    vm.prepend_load_path("/my_custom_path_abc".to_string());
    match vm.globals().get(":") {
        Some(Object::Array(arr)) => {
            let first = arr.borrow().first().cloned();
            match first {
                Some(Object::String(s)) => assert_eq!(s.as_ref(), "/my_custom_path_abc"),
                other => panic!("expected first to be String, got {:?}", other),
            }
        }
        other => panic!("expected $LOAD_PATH to be Array, got {:?}", other),
    }
}
