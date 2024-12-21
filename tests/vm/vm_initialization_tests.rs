use metorex::object::Object;
use metorex::vm::{CallFrame, VirtualMachine};

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
