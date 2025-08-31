use metorex::object::Object;
use metorex::vm::{CallFrame, VirtualMachine};
use std::path::PathBuf;

#[test]
fn vm_initializes_with_builtins_in_global_scope() {
    let vm = VirtualMachine::new();

    assert_eq!(vm.environment().current_depth(), 0);

    let global_scope = vm.environment().global_scope();
    let scope = global_scope.borrow();

    assert!(scope.get("Object").is_some());
    assert!(scope.get("String").is_some());
    assert_eq!(scope.get("nil"), Some(Object::Nil));
    assert_eq!(scope.get("true"), Some(Object::Bool(true)));
    assert_eq!(scope.get("false"), Some(Object::Bool(false)));

    assert!(vm.globals().contains("Object"));
    assert_eq!(vm.globals().get("nil"), Some(Object::Nil));
    assert!(vm.call_stack().is_empty());
}

#[test]
fn vm_heap_allocation_is_accessible() {
    let vm = VirtualMachine::new();
    let heap = vm.heap();

    assert_eq!(heap.borrow().allocation_count(), 0);
}

#[test]
fn call_frame_helper_manages_stack() {
    let mut vm = VirtualMachine::new();
    assert!(vm.call_stack().is_empty());

    let result = vm.with_call_frame(CallFrame::new("main", None), |inner_vm| {
        assert_eq!(inner_vm.call_stack().len(), 1);
        inner_vm
            .call_stack()
            .last()
            .map(|frame| frame.name().to_string())
    });

    assert_eq!(result.as_deref(), Some("main"));
    assert!(vm.call_stack().is_empty());
}

#[test]
fn vm_initializes_with_no_current_file() {
    let vm = VirtualMachine::new();
    assert!(vm.get_current_file().is_none());
}

#[test]
fn vm_can_set_and_get_current_file() {
    let mut vm = VirtualMachine::new();
    let test_path = PathBuf::from("/tmp/test.mx");

    vm.set_current_file(test_path.clone());
    assert_eq!(vm.get_current_file(), Some(&test_path));
}

#[test]
fn vm_can_update_current_file() {
    let mut vm = VirtualMachine::new();
    let path1 = PathBuf::from("/tmp/test1.mx");
    let path2 = PathBuf::from("/tmp/test2.mx");

    vm.set_current_file(path1.clone());
    assert_eq!(vm.get_current_file(), Some(&path1));

    vm.set_current_file(path2.clone());
    assert_eq!(vm.get_current_file(), Some(&path2));
}

#[test]
fn vm_initializes_with_no_loaded_files() {
    let vm = VirtualMachine::new();
    let test_path = PathBuf::from("/tmp/test.mx");
    assert!(!vm.is_file_loaded(&test_path));
}

#[test]
fn vm_can_mark_file_as_loaded() {
    let mut vm = VirtualMachine::new();
    let test_path = PathBuf::from("/tmp/test.mx");

    assert!(!vm.is_file_loaded(&test_path));
    vm.mark_file_loaded(test_path.clone());
    assert!(vm.is_file_loaded(&test_path));
}

#[test]
fn vm_tracks_multiple_loaded_files() {
    let mut vm = VirtualMachine::new();
    let path1 = PathBuf::from("/tmp/test1.mx");
    let path2 = PathBuf::from("/tmp/test2.mx");
    let path3 = PathBuf::from("/tmp/test3.mx");

    vm.mark_file_loaded(path1.clone());
    vm.mark_file_loaded(path2.clone());

    assert!(vm.is_file_loaded(&path1));
    assert!(vm.is_file_loaded(&path2));
    assert!(!vm.is_file_loaded(&path3));
}

#[test]
fn vm_mark_file_loaded_is_idempotent() {
    let mut vm = VirtualMachine::new();
    let test_path = PathBuf::from("/tmp/test.mx");

    vm.mark_file_loaded(test_path.clone());
    vm.mark_file_loaded(test_path.clone());
    vm.mark_file_loaded(test_path.clone());

    assert!(vm.is_file_loaded(&test_path));
}
