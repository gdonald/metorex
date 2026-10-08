// RubyVM: what MRI says about its own virtual machine.

use super::run_example;

/// The expected output of both `ruby_vm/ruby_vm` variants.
const RUBY_VM_OUTPUT: &str = concat!(
    "Class\n",
    "Object\n",
    "[\"direct threaded code\", \"operands unification\", \"inline method cache\"]\n",
    "248\n",
    "[\"nop\", \"getlocal\", \"setlocal\", \"getblockparam\", \"setblockparam\"]\n",
    "true\n",
    "{thread_vm_stack_size: 1048576, thread_machine_stack_size: 1048576, fiber_vm_stack_size: 131072, fiber_machine_stack_size: 524288}\n",
    "[:constant_cache_invalidations, :constant_cache_misses, :global_cvar_state, :next_shape_id, :shape_cache_size]\n",
    "true\n",
    "true\n",
    "false\n",
    "true\n",
    "[:keep_script_lines, :keep_script_lines=, :stat]\n",
    "[ArgumentError, \"unknown key: nope\"]\n",
    "[TypeError, \"non-hash or symbol given\"]\n",
    "[NoMethodError, \"undefined method 'new' for class RubyVM\"]\n",
    "[TypeError, \"allocator undefined for RubyVM\"]\n",
    "\"constant\"\n",
    "\"constant\"\n",
    "false\n",
    "false\n",
);

#[test]
fn test_ruby_vm_ruby_vm_execution() {
    let output = run_example("ruby_vm/ruby_vm.rb");
    assert_eq!(output, RUBY_VM_OUTPUT);
}

#[test]
fn test_ruby_vm_ruby_vm_no_parens_execution() {
    let output = run_example("ruby_vm/ruby_vm_no_parens.rb");
    assert_eq!(output, RUBY_VM_OUTPUT);
}
