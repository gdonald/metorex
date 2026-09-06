#!/usr/bin/env bash

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
METOREX="$REPO_ROOT/target/debug/metorex"
MSPEC="$REPO_ROOT/ruby/mspec/bin/mspec"
SPEC_DIR="$REPO_ROOT/ruby/spec"

# mspec's ruby_exe helper resolves an interpreter at load time and raises when
# it finds none. Point it at metorex so loading succeeds; specs that shell out
# to a subprocess stay commented out regardless.
export RUBY_EXE="$METOREX"

if [ ! -f "$METOREX" ]; then
  echo "Building metorex..."
  cargo build --manifest-path "$REPO_ROOT/Cargo.toml"
fi

# Concurrency for the parallel spec runner. Defaults to the CPU count so a
# fresh checkout uses all cores; override with `JOBS=1 …` for a strictly
# sequential run when debugging interleaving issues.
JOBS="${JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"

# Aggregate counts. Updated after the parallel run drains, then printed by
# the EXIT trap so partial totals still surface if something blows up.
TOTAL_FILES=0
TOTAL_EXAMPLES=0
TOTAL_EXPECTATIONS=0
TOTAL_FAILURES=0
TOTAL_ERRORS=0
TOTAL_TAGGED=0
ANY_FAILED=0

# Use an explicit XXXXXX template so this works on both BSD mktemp (macOS,
# where `-t` is a prefix) and GNU mktemp (Linux/CI, where `-t` requires a
# template with X placeholders).
WORK_DIR=$(mktemp -d -t metorex_specs.XXXXXX)
cleanup_workdir() { rm -rf "$WORK_DIR"; }

print_totals() {
  printf '\n%s files, %s examples, %s expectations, %s failures, %s errors, %s tagged\n' \
    "$TOTAL_FILES" "$TOTAL_EXAMPLES" "$TOTAL_EXPECTATIONS" \
    "$TOTAL_FAILURES" "$TOTAL_ERRORS" "$TOTAL_TAGGED"
}
trap 'print_totals; cleanup_workdir' EXIT

# `run_spec` only queues — actual mspec invocations happen in parallel after
# the queue is fully populated. Indexed via a counter so output ordering
# below stays deterministic regardless of which worker finishes first.
SPEC_COUNT=0
SPEC_PATHS=()
run_spec() {
  SPEC_PATHS[$SPEC_COUNT]="$1"
  SPEC_COUNT=$((SPEC_COUNT + 1))
}

# Passing specs - add new lines as specs reach a passing state

# 4.1 - 4.3: Primitives and Singletons
run_spec "$SPEC_DIR/core/true"
run_spec "$SPEC_DIR/core/nil"
run_spec "$SPEC_DIR/core/false"

# 4.4 - 4.9: Core Classes and Modules
run_spec "$SPEC_DIR/core/comparable"
run_spec "$SPEC_DIR/core/main/to_s_spec.rb"
run_spec "$SPEC_DIR/core/main/define_method_spec.rb"
run_spec "$SPEC_DIR/core/main/include_spec.rb"
run_spec "$SPEC_DIR/core/main/private_spec.rb"
run_spec "$SPEC_DIR/core/main/public_spec.rb"
run_spec "$SPEC_DIR/core/main/ruby2_keywords_spec.rb"
run_spec "$SPEC_DIR/core/main/using_spec.rb"
run_spec "$SPEC_DIR/core/class/allocate_spec.rb"
run_spec "$SPEC_DIR/core/class/attached_object_spec.rb"
run_spec "$SPEC_DIR/core/class/dup_spec.rb"
run_spec "$SPEC_DIR/core/class/inherited_spec.rb"
run_spec "$SPEC_DIR/core/class/initialize_spec.rb"
run_spec "$SPEC_DIR/core/class/new_spec.rb"
run_spec "$SPEC_DIR/core/class/subclasses_spec.rb"
run_spec "$SPEC_DIR/core/class/superclass_spec.rb"
run_spec "$SPEC_DIR/core/module/alias_method_spec.rb"
run_spec "$SPEC_DIR/core/module/ancestors_spec.rb"
run_spec "$SPEC_DIR/core/module/append_features_spec.rb"
run_spec "$SPEC_DIR/core/module/attr_accessor_spec.rb"
run_spec "$SPEC_DIR/core/module/attr_reader_spec.rb"
run_spec "$SPEC_DIR/core/module/attr_spec.rb"
run_spec "$SPEC_DIR/core/module/attr_writer_spec.rb"
run_spec "$SPEC_DIR/core/module/autoload_relative_spec.rb"
run_spec "$SPEC_DIR/core/module/autoload_spec.rb"
run_spec "$SPEC_DIR/core/module/case_compare_spec.rb"
run_spec "$SPEC_DIR/core/module/class_eval_spec.rb"
run_spec "$SPEC_DIR/core/module/class_exec_spec.rb"
run_spec "$SPEC_DIR/core/module/class_variable_defined_spec.rb"
run_spec "$SPEC_DIR/core/module/class_variable_get_spec.rb"
run_spec "$SPEC_DIR/core/module/class_variable_set_spec.rb"
run_spec "$SPEC_DIR/core/module/class_variables_spec.rb"
run_spec "$SPEC_DIR/core/module/comparison_spec.rb"
run_spec "$SPEC_DIR/core/module/const_added_spec.rb"
run_spec "$SPEC_DIR/core/module/const_defined_spec.rb"
run_spec "$SPEC_DIR/core/module/const_get_spec.rb"
run_spec "$SPEC_DIR/core/module/const_missing_spec.rb"
run_spec "$SPEC_DIR/core/module/const_set_spec.rb"
run_spec "$SPEC_DIR/core/module/const_source_location_spec.rb"
run_spec "$SPEC_DIR/core/module/constants_spec.rb"
run_spec "$SPEC_DIR/core/module/define_method_spec.rb"
run_spec "$SPEC_DIR/core/module/define_singleton_method_spec.rb"
run_spec "$SPEC_DIR/core/module/deprecate_constant_spec.rb"
run_spec "$SPEC_DIR/core/module/eql_spec.rb"
run_spec "$SPEC_DIR/core/module/equal_spec.rb"
run_spec "$SPEC_DIR/core/module/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/module/extend_object_spec.rb"
run_spec "$SPEC_DIR/core/module/extended_spec.rb"
run_spec "$SPEC_DIR/core/module/freeze_spec.rb"
run_spec "$SPEC_DIR/core/module/gt_spec.rb"
run_spec "$SPEC_DIR/core/module/gte_spec.rb"
run_spec "$SPEC_DIR/core/module/include_spec.rb"
run_spec "$SPEC_DIR/core/module/included_modules_spec.rb"
run_spec "$SPEC_DIR/core/module/included_spec.rb"
run_spec "$SPEC_DIR/core/module/initialize_copy_spec.rb"
run_spec "$SPEC_DIR/core/module/initialize_spec.rb"
run_spec "$SPEC_DIR/core/module/instance_method_spec.rb"
run_spec "$SPEC_DIR/core/module/instance_methods_spec.rb"
run_spec "$SPEC_DIR/core/module/lt_spec.rb"
run_spec "$SPEC_DIR/core/module/lte_spec.rb"
run_spec "$SPEC_DIR/core/module/method_added_spec.rb"
run_spec "$SPEC_DIR/core/module/method_defined_spec.rb"
run_spec "$SPEC_DIR/core/module/method_removed_spec.rb"
run_spec "$SPEC_DIR/core/module/method_undefined_spec.rb"
run_spec "$SPEC_DIR/core/module/module_eval_spec.rb"
run_spec "$SPEC_DIR/core/module/module_exec_spec.rb"
run_spec "$SPEC_DIR/core/module/module_function_spec.rb"
run_spec "$SPEC_DIR/core/module/name_spec.rb"
run_spec "$SPEC_DIR/core/module/nesting_spec.rb"
run_spec "$SPEC_DIR/core/module/new_spec.rb"
run_spec "$SPEC_DIR/core/module/prepend_features_spec.rb"
# run_spec "$SPEC_DIR/core/module/prepend_spec.rb"  # 54 of its 55 examples pass; the last needs `1 + 2` to dispatch to a user-defined Integer#+ and `super` from it to reach the native operator
run_spec "$SPEC_DIR/core/module/prepended_spec.rb"
run_spec "$SPEC_DIR/core/module/private_class_method_spec.rb"
run_spec "$SPEC_DIR/core/module/private_constant_spec.rb"
run_spec "$SPEC_DIR/core/module/private_instance_methods_spec.rb"
run_spec "$SPEC_DIR/core/module/private_method_defined_spec.rb"
run_spec "$SPEC_DIR/core/module/private_spec.rb"
run_spec "$SPEC_DIR/core/module/protected_instance_methods_spec.rb"
run_spec "$SPEC_DIR/core/module/protected_method_defined_spec.rb"
run_spec "$SPEC_DIR/core/module/protected_spec.rb"
run_spec "$SPEC_DIR/core/module/public_class_method_spec.rb"
run_spec "$SPEC_DIR/core/module/public_constant_spec.rb"
run_spec "$SPEC_DIR/core/module/public_instance_method_spec.rb"
run_spec "$SPEC_DIR/core/module/public_instance_methods_spec.rb"
run_spec "$SPEC_DIR/core/module/public_method_defined_spec.rb"
run_spec "$SPEC_DIR/core/module/public_spec.rb"
# run_spec "$SPEC_DIR/core/module/refine_spec.rb"  # 21 of its 38 examples pass; the rest need refinement-aware dispatch through send, Symbol#to_proc, interpolation, method objects, respond_to?, and refining a module a class includes
run_spec "$SPEC_DIR/core/module/refinements_spec.rb"
run_spec "$SPEC_DIR/core/module/remove_class_variable_spec.rb"
run_spec "$SPEC_DIR/core/module/remove_const_spec.rb"
run_spec "$SPEC_DIR/core/module/remove_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/ruby2_keywords_spec.rb"  # 6 of its 14 examples pass; the other 8 need a keyword-hash flag on Hash that survives a splat, with Hash.ruby2_keywords_hash and .ruby2_keywords_hash?
run_spec "$SPEC_DIR/core/module/set_temporary_name_spec.rb"
run_spec "$SPEC_DIR/core/module/singleton_class_spec.rb"
run_spec "$SPEC_DIR/core/module/to_s_spec.rb"
run_spec "$SPEC_DIR/core/module/undef_method_spec.rb"
run_spec "$SPEC_DIR/core/module/undefined_instance_methods_spec.rb"
run_spec "$SPEC_DIR/core/module/used_refinements_spec.rb"
run_spec "$SPEC_DIR/core/module/using_spec.rb"
run_spec "$SPEC_DIR/core/kernel/Array_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/Complex_spec.rb"  # 64 of its 66 examples pass; the last two need String to carry an encoding so a UTF-16 argument is refused
run_spec "$SPEC_DIR/core/kernel/Float_spec.rb"
run_spec "$SPEC_DIR/core/kernel/Hash_spec.rb"
run_spec "$SPEC_DIR/core/kernel/Integer_spec.rb"
run_spec "$SPEC_DIR/core/kernel/Rational_spec.rb"
run_spec "$SPEC_DIR/core/kernel/String_spec.rb"
run_spec "$SPEC_DIR/core/kernel/__callee___spec.rb"
run_spec "$SPEC_DIR/core/kernel/__dir___spec.rb"
run_spec "$SPEC_DIR/core/kernel/__method___spec.rb"
run_spec "$SPEC_DIR/core/kernel/abort_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/at_exit_spec.rb"  # 10 of its 12 examples pass; the other two need Ruby's uncaught-exception report format and a lexical __FILE__ inside a block
run_spec "$SPEC_DIR/core/kernel/autoload_relative_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/autoload_spec.rb"  # 19 of its 23 examples pass; the other four need `autoload` inside a module's instance method to register on that module
# run_spec "$SPEC_DIR/core/kernel/backtick_spec.rb"  # 5 of its 8 examples pass; the rest need String to carry an encoding and mspec's stderr-fd capture
run_spec "$SPEC_DIR/core/kernel/binding_spec.rb"
run_spec "$SPEC_DIR/core/kernel/block_given_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/caller_locations_spec.rb"  # 11 of its 12 examples pass; the custom-offset one needs the call stack to hold the same frames MRI does
run_spec "$SPEC_DIR/core/kernel/caller_spec.rb"
run_spec "$SPEC_DIR/core/kernel/case_compare_spec.rb"
run_spec "$SPEC_DIR/core/kernel/catch_spec.rb"
run_spec "$SPEC_DIR/core/kernel/chomp_spec.rb"
run_spec "$SPEC_DIR/core/kernel/chop_spec.rb"
run_spec "$SPEC_DIR/core/kernel/class_spec.rb"
run_spec "$SPEC_DIR/core/kernel/clone_spec.rb"
run_spec "$SPEC_DIR/core/kernel/comparison_spec.rb"
run_spec "$SPEC_DIR/core/kernel/define_singleton_method_spec.rb"
run_spec "$SPEC_DIR/core/kernel/display_spec.rb"
run_spec "$SPEC_DIR/core/kernel/dup_spec.rb"
run_spec "$SPEC_DIR/core/kernel/enum_for_spec.rb"
run_spec "$SPEC_DIR/core/kernel/eql_spec.rb"
run_spec "$SPEC_DIR/core/kernel/equal_value_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/eval_spec.rb"  # 23 of its 56 examples pass; the rest need per-string encodings with magic comments, binding and default-definee semantics, and flip-flop
run_spec "$SPEC_DIR/core/kernel/exec_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/exit_spec.rb"  # 26 of its 30 examples pass; the rest need real Thread semantics and a Fiber class
run_spec "$SPEC_DIR/core/kernel/extend_spec.rb"
run_spec "$SPEC_DIR/core/kernel/fail_spec.rb"
run_spec "$SPEC_DIR/core/kernel/fork_spec.rb"
run_spec "$SPEC_DIR/core/kernel/format_spec.rb"
run_spec "$SPEC_DIR/core/kernel/freeze_spec.rb"
run_spec "$SPEC_DIR/core/kernel/frozen_spec.rb"
run_spec "$SPEC_DIR/core/kernel/gets_spec.rb"
run_spec "$SPEC_DIR/core/kernel/global_variables_spec.rb"
run_spec "$SPEC_DIR/core/kernel/gsub_spec.rb"
run_spec "$SPEC_DIR/core/kernel/initialize_clone_spec.rb"
run_spec "$SPEC_DIR/core/kernel/initialize_copy_spec.rb"
run_spec "$SPEC_DIR/core/kernel/initialize_dup_spec.rb"
run_spec "$SPEC_DIR/core/kernel/inspect_spec.rb"
run_spec "$SPEC_DIR/core/kernel/instance_of_spec.rb"
run_spec "$SPEC_DIR/core/kernel/instance_variable_defined_spec.rb"
run_spec "$SPEC_DIR/core/kernel/instance_variable_get_spec.rb"
run_spec "$SPEC_DIR/core/kernel/instance_variable_set_spec.rb"
run_spec "$SPEC_DIR/core/kernel/instance_variables_spec.rb"
run_spec "$SPEC_DIR/core/kernel/is_a_spec.rb"
run_spec "$SPEC_DIR/core/kernel/itself_spec.rb"
run_spec "$SPEC_DIR/core/kernel/kind_of_spec.rb"
run_spec "$SPEC_DIR/core/kernel/lambda_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/load_spec.rb"  # 97 of its 103 examples pass; the rest need main modelled as its own object and the circular-require warning
run_spec "$SPEC_DIR/core/kernel/local_variables_spec.rb"
run_spec "$SPEC_DIR/core/kernel/loop_spec.rb"
run_spec "$SPEC_DIR/core/kernel/match_spec.rb"
run_spec "$SPEC_DIR/core/kernel/method_spec.rb"
run_spec "$SPEC_DIR/core/kernel/methods_spec.rb"
run_spec "$SPEC_DIR/core/kernel/nil_spec.rb"
run_spec "$SPEC_DIR/core/kernel/not_match_spec.rb"
run_spec "$SPEC_DIR/core/kernel/object_id_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/open_spec.rb"  # 14 of its 15 examples pass; the last needs the open-uri standard library
# run_spec "$SPEC_DIR/core/kernel/p_spec.rb"  # 5 of its 6 examples pass; the last needs STDOUT and STDERR as IO objects for mspec's output_to_fd
run_spec "$SPEC_DIR/core/kernel/pp_spec.rb"
run_spec "$SPEC_DIR/core/kernel/print_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/printf_spec.rb"  # 44 of its 286 examples pass; the rest exercise the shared sprintf suite, whose width, precision, and coercion rules are its own piece of work
run_spec "$SPEC_DIR/core/kernel/private_methods_spec.rb"
run_spec "$SPEC_DIR/core/kernel/proc_spec.rb"
run_spec "$SPEC_DIR/core/kernel/protected_methods_spec.rb"
run_spec "$SPEC_DIR/core/kernel/public_method_spec.rb"
run_spec "$SPEC_DIR/core/kernel/public_methods_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/public_send_spec.rb"  # needs real module ancestry so `super` walks included modules
# run_spec "$SPEC_DIR/core/kernel/putc_spec.rb"  # every example needs a File IO object for `new_io`
# run_spec "$SPEC_DIR/core/kernel/puts_spec.rb"  # every example needs a File IO object for `new_io`
# run_spec "$SPEC_DIR/core/kernel/raise_spec.rb"  # needs Exception#data, three-argument raise with a backtrace, cause chaining, and backtrace preservation on re-raise
# run_spec "$SPEC_DIR/core/kernel/rand_spec.rb"  # needs a Random class, custom range endpoint types, and ruby_exe subprocess support
run_spec "$SPEC_DIR/core/kernel/readline_spec.rb"
run_spec "$SPEC_DIR/core/kernel/readlines_spec.rb"
run_spec "$SPEC_DIR/core/kernel/remove_instance_variable_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/require_relative_spec.rb"  # needs $LOADED_FEATURES tracking, extension resolution, Dir.chdir, and File.symlink
# run_spec "$SPEC_DIR/core/kernel/require_spec.rb"  # hangs, and needs the same $LOADED_FEATURES tracking as require_relative_spec
run_spec "$SPEC_DIR/core/kernel/respond_to_missing_spec.rb"
run_spec "$SPEC_DIR/core/kernel/respond_to_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/select_spec.rb"  # needs IO.pipe and fd-level select
# run_spec "$SPEC_DIR/core/kernel/send_spec.rb"  # needs real module ancestry so `super` walks included modules
# run_spec "$SPEC_DIR/core/kernel/set_trace_func_spec.rb"  # needs a tracing hook the interpreter calls on each event
# run_spec "$SPEC_DIR/core/kernel/singleton_class_spec.rb"  # needs per-string frozen state for a deduplicated String, and File IO for the reopen example
run_spec "$SPEC_DIR/core/kernel/singleton_method_spec.rb"
run_spec "$SPEC_DIR/core/kernel/singleton_methods_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/sleep_spec.rb"  # hangs: needs Thread scheduling (Thread.pass, wakeup, status) and a Fiber scheduler
# run_spec "$SPEC_DIR/core/kernel/spawn_spec.rb"  # needs subprocess execution and fd-level output capture
# run_spec "$SPEC_DIR/core/kernel/sprintf_spec.rb"  # needs the full format surface: %e %g %G %B, the # flag, width and precision, RangeError, and KeyError for named references
# run_spec "$SPEC_DIR/core/kernel/srand_spec.rb"  # its one remaining example needs ruby_exe subprocess support
run_spec "$SPEC_DIR/core/kernel/sub_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/syscall_spec.rb"  # needs raw OS syscall dispatch
# run_spec "$SPEC_DIR/core/kernel/system_spec.rb"  # needs subprocess execution and Process::Status
run_spec "$SPEC_DIR/core/kernel/taint_spec.rb"
run_spec "$SPEC_DIR/core/kernel/tainted_spec.rb"
run_spec "$SPEC_DIR/core/kernel/tap_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/test_spec.rb"  # needs the file-test operators and File IO
run_spec "$SPEC_DIR/core/kernel/then_spec.rb"
run_spec "$SPEC_DIR/core/kernel/throw_spec.rb"
run_spec "$SPEC_DIR/core/kernel/to_enum_spec.rb"
run_spec "$SPEC_DIR/core/kernel/to_s_spec.rb"
run_spec "$SPEC_DIR/core/kernel/trace_var_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/trap_spec.rb"
run_spec "$SPEC_DIR/core/kernel/trust_spec.rb"
run_spec "$SPEC_DIR/core/kernel/untaint_spec.rb"
run_spec "$SPEC_DIR/core/kernel/untrace_var_spec.rb"
run_spec "$SPEC_DIR/core/kernel/untrust_spec.rb"
run_spec "$SPEC_DIR/core/kernel/untrusted_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/warn_spec.rb"  # its last 4 examples shell out through IO.popen
run_spec "$SPEC_DIR/core/kernel/yield_self_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/__id__spec.rb"
run_spec "$SPEC_DIR/core/basicobject/__send___spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/basicobject_spec.rb"  # its method_missing example shells out through IO.popen
run_spec "$SPEC_DIR/core/basicobject/equal_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/initialize_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/instance_eval_spec.rb"  # needs a default definee separate from the lexical scope, eval source locations, and block-local scoping
# run_spec "$SPEC_DIR/core/basicobject/instance_exec_spec.rb"  # its last 2 examples need a default definee separate from the lexical scope
run_spec "$SPEC_DIR/core/basicobject/method_missing_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/not_equal_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/not_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/singleton_method_added_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/singleton_method_removed_spec.rb"
run_spec "$SPEC_DIR/core/basicobject/singleton_method_undefined_spec.rb"

# 4.10 - 4.13: Exceptions and Signals
run_spec "$SPEC_DIR/core/exception/backtrace_spec.rb"
run_spec "$SPEC_DIR/core/exception/backtrace_locations_spec.rb"
run_spec "$SPEC_DIR/core/exception/case_compare_spec.rb"
run_spec "$SPEC_DIR/core/exception/cause_spec.rb"
run_spec "$SPEC_DIR/core/exception/detailed_message_spec.rb"
run_spec "$SPEC_DIR/core/exception/dup_spec.rb"
run_spec "$SPEC_DIR/core/exception/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/exception/errno_spec.rb"
run_spec "$SPEC_DIR/core/exception/exception_spec.rb"
run_spec "$SPEC_DIR/core/exception/exit_value_spec.rb"
run_spec "$SPEC_DIR/core/exception/frozen_error_spec.rb"
# run_spec "$SPEC_DIR/core/exception/full_message_spec.rb"  # needs the caller when an exception has no backtrace, and file attribution for a raise inside a block
run_spec "$SPEC_DIR/core/exception/hierarchy_spec.rb"
run_spec "$SPEC_DIR/core/exception/inspect_spec.rb"
# run_spec "$SPEC_DIR/core/exception/interrupt_spec.rb"  # 3 of its 5 examples pass; the other 2 need ruby_exe and IO.popen subprocess support
run_spec "$SPEC_DIR/core/exception/io_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/key_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/load_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/message_spec.rb"
run_spec "$SPEC_DIR/core/exception/name_spec.rb"
run_spec "$SPEC_DIR/core/exception/name_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/new_spec.rb"
run_spec "$SPEC_DIR/core/exception/no_method_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/reason_spec.rb"
run_spec "$SPEC_DIR/core/exception/receiver_spec.rb"
run_spec "$SPEC_DIR/core/exception/result_spec.rb"
run_spec "$SPEC_DIR/core/exception/set_backtrace_spec.rb"
# run_spec "$SPEC_DIR/core/exception/signal_exception_spec.rb"  # 12 of its 16 examples pass; the other 4 need ruby_exe and IO.popen subprocess support
run_spec "$SPEC_DIR/core/exception/signm_spec.rb"
run_spec "$SPEC_DIR/core/exception/signo_spec.rb"
run_spec "$SPEC_DIR/core/exception/standard_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/status_spec.rb"
run_spec "$SPEC_DIR/core/exception/success_spec.rb"
run_spec "$SPEC_DIR/core/exception/syntax_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/system_call_error_spec.rb"
run_spec "$SPEC_DIR/core/exception/system_exit_spec.rb"
run_spec "$SPEC_DIR/core/exception/to_s_spec.rb"
# run_spec "$SPEC_DIR/core/exception/top_level_spec.rb"  # its last two examples hang the runner, needing real Thread and Fiber with a bare `sleep` and `Thread#stop?`
run_spec "$SPEC_DIR/core/exception/uncaught_throw_error_spec.rb"
run_spec "$SPEC_DIR/core/systemexit"
run_spec "$SPEC_DIR/core/signal/list_spec.rb"
run_spec "$SPEC_DIR/core/signal/signame_spec.rb"
# run_spec "$SPEC_DIR/core/signal/trap_spec.rb"  # needs Thread#group with Thread.main.group, and IO.pipe with a write to a closed pipe dying by SIGPIPE
run_spec "$SPEC_DIR/core/warning/categories_spec.rb"
run_spec "$SPEC_DIR/core/warning/element_reference_spec.rb"
run_spec "$SPEC_DIR/core/warning/element_set_spec.rb"
run_spec "$SPEC_DIR/core/warning/warn_spec.rb"
# run_spec "$SPEC_DIR/core/warning/performance_warning_spec.rb"  # needs `1 + 2` to dispatch to a user-defined Integer#+, the same gap module/prepend_spec waits on

# 4.14 - 4.19: Numeric Types
run_spec "$SPEC_DIR/core/integer/abs_spec.rb"
run_spec "$SPEC_DIR/core/integer/allbits_spec.rb"
run_spec "$SPEC_DIR/core/integer/anybits_spec.rb"
run_spec "$SPEC_DIR/core/integer/bit_and_spec.rb"
run_spec "$SPEC_DIR/core/integer/bit_or_spec.rb"
run_spec "$SPEC_DIR/core/integer/bit_xor_spec.rb"
run_spec "$SPEC_DIR/core/integer/case_compare_spec.rb"
run_spec "$SPEC_DIR/core/integer/ceil_spec.rb"
run_spec "$SPEC_DIR/core/integer/ceildiv_spec.rb"
run_spec "$SPEC_DIR/core/integer/coerce_spec.rb"
run_spec "$SPEC_DIR/core/integer/comparison_spec.rb"
run_spec "$SPEC_DIR/core/integer/complement_spec.rb"
run_spec "$SPEC_DIR/core/integer/constants_spec.rb"
run_spec "$SPEC_DIR/core/integer/denominator_spec.rb"
run_spec "$SPEC_DIR/core/integer/digits_spec.rb"
run_spec "$SPEC_DIR/core/integer/divide_spec.rb"
run_spec "$SPEC_DIR/core/integer/divmod_spec.rb"
run_spec "$SPEC_DIR/core/integer/dup_spec.rb"
run_spec "$SPEC_DIR/core/integer/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/integer/even_spec.rb"
run_spec "$SPEC_DIR/core/integer/fdiv_spec.rb"
run_spec "$SPEC_DIR/core/integer/floor_spec.rb"
run_spec "$SPEC_DIR/core/integer/gcd_spec.rb"
run_spec "$SPEC_DIR/core/integer/gt_spec.rb"
run_spec "$SPEC_DIR/core/integer/gte_spec.rb"
run_spec "$SPEC_DIR/core/integer/gcdlcm_spec.rb"
run_spec "$SPEC_DIR/core/integer/integer_spec.rb"
run_spec "$SPEC_DIR/core/integer/lcm_spec.rb"
run_spec "$SPEC_DIR/core/integer/lt_spec.rb"
run_spec "$SPEC_DIR/core/integer/lte_spec.rb"
run_spec "$SPEC_DIR/core/integer/magnitude_spec.rb"
run_spec "$SPEC_DIR/core/integer/minus_spec.rb"
run_spec "$SPEC_DIR/core/integer/modulo_spec.rb"
run_spec "$SPEC_DIR/core/integer/multiply_spec.rb"
run_spec "$SPEC_DIR/core/integer/nobits_spec.rb"
run_spec "$SPEC_DIR/core/integer/numerator_spec.rb"
run_spec "$SPEC_DIR/core/integer/odd_spec.rb"
run_spec "$SPEC_DIR/core/integer/ord_spec.rb"
run_spec "$SPEC_DIR/core/integer/rationalize_spec.rb"
run_spec "$SPEC_DIR/core/integer/remainder_spec.rb"
run_spec "$SPEC_DIR/core/integer/round_spec.rb"
run_spec "$SPEC_DIR/core/integer/to_f_spec.rb"
run_spec "$SPEC_DIR/core/integer/to_i_spec.rb"
run_spec "$SPEC_DIR/core/integer/size_spec.rb"
run_spec "$SPEC_DIR/core/integer/sqrt_spec.rb"
run_spec "$SPEC_DIR/core/integer/to_int_spec.rb"
run_spec "$SPEC_DIR/core/integer/to_r_spec.rb"
run_spec "$SPEC_DIR/core/integer/truncate_spec.rb"
run_spec "$SPEC_DIR/core/integer/try_convert_spec.rb"
# run_spec "$SPEC_DIR/core/integer"  # the other 17 files are enabled one at a time as they pass; exponent_spec and pow_spec also need `1 ** 4611686018427387904` to answer without computing it
run_spec "$SPEC_DIR/core/float/abs_spec.rb"
run_spec "$SPEC_DIR/core/float/angle_spec.rb"
run_spec "$SPEC_DIR/core/float/arg_spec.rb"
run_spec "$SPEC_DIR/core/float/case_compare_spec.rb"
run_spec "$SPEC_DIR/core/float/ceil_spec.rb"
run_spec "$SPEC_DIR/core/float/coerce_spec.rb"
run_spec "$SPEC_DIR/core/float/denominator_spec.rb"
run_spec "$SPEC_DIR/core/float/divide_spec.rb"
run_spec "$SPEC_DIR/core/float/divmod_spec.rb"
run_spec "$SPEC_DIR/core/float/dup_spec.rb"
run_spec "$SPEC_DIR/core/float/eql_spec.rb"
run_spec "$SPEC_DIR/core/float/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/float/fdiv_spec.rb"
run_spec "$SPEC_DIR/core/float/finite_spec.rb"
run_spec "$SPEC_DIR/core/float/float_spec.rb"
run_spec "$SPEC_DIR/core/float/floor_spec.rb"
run_spec "$SPEC_DIR/core/float/gt_spec.rb"
run_spec "$SPEC_DIR/core/float/gte_spec.rb"
run_spec "$SPEC_DIR/core/float/hash_spec.rb"
run_spec "$SPEC_DIR/core/float/infinite_spec.rb"
run_spec "$SPEC_DIR/core/float/lt_spec.rb"
run_spec "$SPEC_DIR/core/float/lte_spec.rb"
run_spec "$SPEC_DIR/core/float/magnitude_spec.rb"
run_spec "$SPEC_DIR/core/float/minus_spec.rb"
run_spec "$SPEC_DIR/core/float/modulo_spec.rb"
run_spec "$SPEC_DIR/core/float/multiply_spec.rb"
run_spec "$SPEC_DIR/core/float/nan_spec.rb"
run_spec "$SPEC_DIR/core/float/negative_spec.rb"
run_spec "$SPEC_DIR/core/float/next_float_spec.rb"
run_spec "$SPEC_DIR/core/float/numerator_spec.rb"
run_spec "$SPEC_DIR/core/float/phase_spec.rb"
run_spec "$SPEC_DIR/core/float/plus_spec.rb"
run_spec "$SPEC_DIR/core/float/positive_spec.rb"
run_spec "$SPEC_DIR/core/float/prev_float_spec.rb"
run_spec "$SPEC_DIR/core/float/quo_spec.rb"
run_spec "$SPEC_DIR/core/float/to_f_spec.rb"
run_spec "$SPEC_DIR/core/float/to_i_spec.rb"
run_spec "$SPEC_DIR/core/float/to_int_spec.rb"
run_spec "$SPEC_DIR/core/float/to_r_spec.rb"
run_spec "$SPEC_DIR/core/float/truncate_spec.rb"
run_spec "$SPEC_DIR/core/float/uminus_spec.rb"
run_spec "$SPEC_DIR/core/float/uplus_spec.rb"
run_spec "$SPEC_DIR/core/float/zero_spec.rb"
# run_spec "$SPEC_DIR/core/float"  # the other 7 files are enabled one at a time as they pass
run_spec "$SPEC_DIR/core/numeric/abs_spec.rb"
run_spec "$SPEC_DIR/core/numeric/ceil_spec.rb"
run_spec "$SPEC_DIR/core/numeric/clone_spec.rb"
run_spec "$SPEC_DIR/core/numeric/coerce_spec.rb"
run_spec "$SPEC_DIR/core/numeric/comparison_spec.rb"
run_spec "$SPEC_DIR/core/numeric/conj_spec.rb"
run_spec "$SPEC_DIR/core/numeric/conjugate_spec.rb"
run_spec "$SPEC_DIR/core/numeric/denominator_spec.rb"
run_spec "$SPEC_DIR/core/numeric/div_spec.rb"
run_spec "$SPEC_DIR/core/numeric/divmod_spec.rb"
run_spec "$SPEC_DIR/core/numeric/dup_spec.rb"
run_spec "$SPEC_DIR/core/numeric/eql_spec.rb"
run_spec "$SPEC_DIR/core/numeric/fdiv_spec.rb"
run_spec "$SPEC_DIR/core/numeric/finite_spec.rb"
run_spec "$SPEC_DIR/core/numeric/floor_spec.rb"
run_spec "$SPEC_DIR/core/numeric/imag_spec.rb"
run_spec "$SPEC_DIR/core/numeric/imaginary_spec.rb"
run_spec "$SPEC_DIR/core/numeric/infinite_spec.rb"
run_spec "$SPEC_DIR/core/numeric/integer_spec.rb"
run_spec "$SPEC_DIR/core/numeric/magnitude_spec.rb"
run_spec "$SPEC_DIR/core/numeric/modulo_spec.rb"
run_spec "$SPEC_DIR/core/numeric/negative_spec.rb"
run_spec "$SPEC_DIR/core/numeric/nonzero_spec.rb"
run_spec "$SPEC_DIR/core/numeric/numerator_spec.rb"
run_spec "$SPEC_DIR/core/numeric/numeric_spec.rb"
run_spec "$SPEC_DIR/core/numeric/positive_spec.rb"
run_spec "$SPEC_DIR/core/numeric/real_spec.rb"
run_spec "$SPEC_DIR/core/numeric/round_spec.rb"
run_spec "$SPEC_DIR/core/numeric/to_int_spec.rb"
run_spec "$SPEC_DIR/core/numeric/truncate_spec.rb"
run_spec "$SPEC_DIR/core/numeric/uplus_spec.rb"
run_spec "$SPEC_DIR/core/numeric/zero_spec.rb"
# run_spec "$SPEC_DIR/core/numeric"  # the other 14 files are enabled one at a time as they pass
run_spec "$SPEC_DIR/core/complex/abs_spec.rb"
run_spec "$SPEC_DIR/core/complex/abs2_spec.rb"
run_spec "$SPEC_DIR/core/complex/angle_spec.rb"
run_spec "$SPEC_DIR/core/complex/arg_spec.rb"
run_spec "$SPEC_DIR/core/complex/comparison_spec.rb"
run_spec "$SPEC_DIR/core/complex/conj_spec.rb"
run_spec "$SPEC_DIR/core/complex/conjugate_spec.rb"
run_spec "$SPEC_DIR/core/complex/constants_spec.rb"
run_spec "$SPEC_DIR/core/complex/denominator_spec.rb"
run_spec "$SPEC_DIR/core/complex/eql_spec.rb"
run_spec "$SPEC_DIR/core/complex/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/complex/finite_spec.rb"
run_spec "$SPEC_DIR/core/complex/hash_spec.rb"
run_spec "$SPEC_DIR/core/complex/imag_spec.rb"
run_spec "$SPEC_DIR/core/complex/imaginary_spec.rb"
run_spec "$SPEC_DIR/core/complex/infinite_spec.rb"
run_spec "$SPEC_DIR/core/complex/inspect_spec.rb"
run_spec "$SPEC_DIR/core/complex/integer_spec.rb"
run_spec "$SPEC_DIR/core/complex/magnitude_spec.rb"
run_spec "$SPEC_DIR/core/complex/negative_spec.rb"
run_spec "$SPEC_DIR/core/complex/numerator_spec.rb"
run_spec "$SPEC_DIR/core/complex/phase_spec.rb"
run_spec "$SPEC_DIR/core/complex/polar_spec.rb"
run_spec "$SPEC_DIR/core/complex/positive_spec.rb"
run_spec "$SPEC_DIR/core/complex/rationalize_spec.rb"
run_spec "$SPEC_DIR/core/complex/real_spec.rb"
run_spec "$SPEC_DIR/core/complex/rect_spec.rb"
run_spec "$SPEC_DIR/core/complex/rectangular_spec.rb"
run_spec "$SPEC_DIR/core/complex/to_c_spec.rb"
run_spec "$SPEC_DIR/core/complex/to_f_spec.rb"
run_spec "$SPEC_DIR/core/complex/to_i_spec.rb"
run_spec "$SPEC_DIR/core/complex/to_r_spec.rb"
run_spec "$SPEC_DIR/core/complex/to_s_spec.rb"
run_spec "$SPEC_DIR/core/complex/uminus_spec.rb"
# run_spec "$SPEC_DIR/core/complex"  # the other 9 files are enabled one at a time as they pass
run_spec "$SPEC_DIR/core/rational/abs_spec.rb"
run_spec "$SPEC_DIR/core/rational/ceil_spec.rb"
run_spec "$SPEC_DIR/core/rational/comparison_spec.rb"
run_spec "$SPEC_DIR/core/rational/denominator_spec.rb"
run_spec "$SPEC_DIR/core/rational/div_spec.rb"
run_spec "$SPEC_DIR/core/rational/divide_spec.rb"
run_spec "$SPEC_DIR/core/rational/divmod_spec.rb"
run_spec "$SPEC_DIR/core/rational/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/rational/fdiv_spec.rb"
run_spec "$SPEC_DIR/core/rational/floor_spec.rb"
run_spec "$SPEC_DIR/core/rational/hash_spec.rb"
run_spec "$SPEC_DIR/core/rational/inspect_spec.rb"
run_spec "$SPEC_DIR/core/rational/integer_spec.rb"
run_spec "$SPEC_DIR/core/rational/magnitude_spec.rb"
run_spec "$SPEC_DIR/core/rational/minus_spec.rb"
run_spec "$SPEC_DIR/core/rational/modulo_spec.rb"
run_spec "$SPEC_DIR/core/rational/multiply_spec.rb"
run_spec "$SPEC_DIR/core/rational/numerator_spec.rb"
run_spec "$SPEC_DIR/core/rational/plus_spec.rb"
run_spec "$SPEC_DIR/core/rational/rational_spec.rb"
run_spec "$SPEC_DIR/core/rational/remainder_spec.rb"
run_spec "$SPEC_DIR/core/rational/round_spec.rb"
run_spec "$SPEC_DIR/core/rational/to_f_spec.rb"
run_spec "$SPEC_DIR/core/rational/to_i_spec.rb"
run_spec "$SPEC_DIR/core/rational/to_s_spec.rb"
run_spec "$SPEC_DIR/core/rational/truncate_spec.rb"
run_spec "$SPEC_DIR/core/rational/zero_spec.rb"
# run_spec "$SPEC_DIR/core/rational"  # the other 5 files are enabled one at a time as they pass
run_spec "$SPEC_DIR/core/math/acos_spec.rb"
run_spec "$SPEC_DIR/core/math/acosh_spec.rb"
run_spec "$SPEC_DIR/core/math/asin_spec.rb"
run_spec "$SPEC_DIR/core/math/asinh_spec.rb"
run_spec "$SPEC_DIR/core/math/atan_spec.rb"
run_spec "$SPEC_DIR/core/math/atan2_spec.rb"
run_spec "$SPEC_DIR/core/math/atanh_spec.rb"
run_spec "$SPEC_DIR/core/math/cbrt_spec.rb"
run_spec "$SPEC_DIR/core/math/constants_spec.rb"
run_spec "$SPEC_DIR/core/math/cos_spec.rb"
run_spec "$SPEC_DIR/core/math/cosh_spec.rb"
run_spec "$SPEC_DIR/core/math/erf_spec.rb"
run_spec "$SPEC_DIR/core/math/erfc_spec.rb"
run_spec "$SPEC_DIR/core/math/exp_spec.rb"
run_spec "$SPEC_DIR/core/math/expm1_spec.rb"
run_spec "$SPEC_DIR/core/math/frexp_spec.rb"
run_spec "$SPEC_DIR/core/math/hypot_spec.rb"
run_spec "$SPEC_DIR/core/math/ldexp_spec.rb"
run_spec "$SPEC_DIR/core/math/log_spec.rb"
run_spec "$SPEC_DIR/core/math/log10_spec.rb"
run_spec "$SPEC_DIR/core/math/log1p_spec.rb"
run_spec "$SPEC_DIR/core/math/log2_spec.rb"
run_spec "$SPEC_DIR/core/math/sin_spec.rb"
run_spec "$SPEC_DIR/core/math/sinh_spec.rb"
run_spec "$SPEC_DIR/core/math/sqrt_spec.rb"
run_spec "$SPEC_DIR/core/math/tan_spec.rb"
run_spec "$SPEC_DIR/core/math/tanh_spec.rb"
# run_spec "$SPEC_DIR/core/math"  # gamma_spec and lgamma_spec need a gamma function accurate to the last digit

# 4.20 - 4.25: Collections
run_spec "$SPEC_DIR/core/array/append_spec.rb"
run_spec "$SPEC_DIR/core/array/array_spec.rb"
run_spec "$SPEC_DIR/core/array/clear_spec.rb"
run_spec "$SPEC_DIR/core/array/deconstruct_spec.rb"
run_spec "$SPEC_DIR/core/array/delete_spec.rb"
run_spec "$SPEC_DIR/core/array/delete_at_spec.rb"
run_spec "$SPEC_DIR/core/array/delete_if_spec.rb"
run_spec "$SPEC_DIR/core/array/each_index_spec.rb"
run_spec "$SPEC_DIR/core/array/empty_spec.rb"
run_spec "$SPEC_DIR/core/array/find_index_spec.rb"
run_spec "$SPEC_DIR/core/array/frozen_spec.rb"
run_spec "$SPEC_DIR/core/array/include_spec.rb"
run_spec "$SPEC_DIR/core/array/index_spec.rb"
run_spec "$SPEC_DIR/core/array/keep_if_spec.rb"
run_spec "$SPEC_DIR/core/array/length_spec.rb"
run_spec "$SPEC_DIR/core/array/push_spec.rb"
run_spec "$SPEC_DIR/core/array/reverse_each_spec.rb"
run_spec "$SPEC_DIR/core/array/size_spec.rb"
# run_spec "$SPEC_DIR/core/array"  # the other 84 files are enabled one at a time as they pass
run_spec "$SPEC_DIR/core/hash/clone_spec.rb"
# run_spec "$SPEC_DIR/core/hash"  # the other 68 files are enabled one at a time as they pass
# run_spec "$SPEC_DIR/core/set"
run_spec "$SPEC_DIR/core/range/begin_spec.rb"
run_spec "$SPEC_DIR/core/range/end_spec.rb"
run_spec "$SPEC_DIR/core/range/hash_spec.rb"
run_spec "$SPEC_DIR/core/range/range_spec.rb"
# run_spec "$SPEC_DIR/core/range"  # the other 29 files are enabled one at a time as they pass
run_spec "$SPEC_DIR/core/struct/clone_spec.rb"
run_spec "$SPEC_DIR/core/struct/constants_spec.rb"
run_spec "$SPEC_DIR/core/struct/deconstruct_spec.rb"
run_spec "$SPEC_DIR/core/struct/deconstruct_keys_spec.rb"
run_spec "$SPEC_DIR/core/struct/dig_spec.rb"
run_spec "$SPEC_DIR/core/struct/dup_spec.rb"
run_spec "$SPEC_DIR/core/struct/each_spec.rb"
run_spec "$SPEC_DIR/core/struct/each_pair_spec.rb"
run_spec "$SPEC_DIR/core/struct/element_reference_spec.rb"
run_spec "$SPEC_DIR/core/struct/element_set_spec.rb"
run_spec "$SPEC_DIR/core/struct/eql_spec.rb"
run_spec "$SPEC_DIR/core/struct/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/struct/filter_spec.rb"
run_spec "$SPEC_DIR/core/struct/hash_spec.rb"
run_spec "$SPEC_DIR/core/struct/inspect_spec.rb"
run_spec "$SPEC_DIR/core/struct/instance_variable_get_spec.rb"
run_spec "$SPEC_DIR/core/struct/instance_variables_spec.rb"
run_spec "$SPEC_DIR/core/struct/keyword_init_spec.rb"
run_spec "$SPEC_DIR/core/struct/length_spec.rb"
run_spec "$SPEC_DIR/core/struct/members_spec.rb"
run_spec "$SPEC_DIR/core/struct/select_spec.rb"
run_spec "$SPEC_DIR/core/struct/size_spec.rb"
run_spec "$SPEC_DIR/core/struct/struct_spec.rb"
run_spec "$SPEC_DIR/core/struct/to_a_spec.rb"
run_spec "$SPEC_DIR/core/struct/to_h_spec.rb"
run_spec "$SPEC_DIR/core/struct/to_s_spec.rb"
run_spec "$SPEC_DIR/core/struct/values_spec.rb"
run_spec "$SPEC_DIR/core/struct/values_at_spec.rb"
# run_spec "$SPEC_DIR/core/struct"  # the other 2 files are enabled one at a time as they pass
# run_spec "$SPEC_DIR/core/data"

# 4.26 - 4.30: Strings and Patterns
run_spec "$SPEC_DIR/core/string/empty_spec.rb"
run_spec "$SPEC_DIR/core/string/hash_spec.rb"
run_spec "$SPEC_DIR/core/string/string_spec.rb"
run_spec "$SPEC_DIR/core/string/to_s_spec.rb"
# run_spec "$SPEC_DIR/core/string"  # the other 110 files are enabled one at a time as they pass
run_spec "$SPEC_DIR/core/symbol/case_compare_spec.rb"
run_spec "$SPEC_DIR/core/symbol/dup_spec.rb"
run_spec "$SPEC_DIR/core/symbol/empty_spec.rb"
run_spec "$SPEC_DIR/core/symbol/equal_value_spec.rb"
run_spec "$SPEC_DIR/core/symbol/to_sym_spec.rb"
# run_spec "$SPEC_DIR/core/symbol"  # the other 24 files are enabled one at a time as they pass
# run_spec "$SPEC_DIR/core/regexp"
# run_spec "$SPEC_DIR/core/encoding"
# run_spec "$SPEC_DIR/core/matchdata"

# 4.31 - 4.34: Callables and Introspection
# run_spec "$SPEC_DIR/core/proc"
# run_spec "$SPEC_DIR/core/method"
# run_spec "$SPEC_DIR/core/unboundmethod"
# run_spec "$SPEC_DIR/core/binding"

# 4.35 - 4.39: IO and Filesystem
# run_spec "$SPEC_DIR/core/io"
# run_spec "$SPEC_DIR/core/file"
# run_spec "$SPEC_DIR/core/dir"
# run_spec "$SPEC_DIR/core/filetest"
# run_spec "$SPEC_DIR/core/env"

# 4.40 - 4.46: Concurrency
# run_spec "$SPEC_DIR/core/thread"
# run_spec "$SPEC_DIR/core/fiber"
# run_spec "$SPEC_DIR/core/mutex"
# run_spec "$SPEC_DIR/core/conditionvariable"
# run_spec "$SPEC_DIR/core/queue"
# run_spec "$SPEC_DIR/core/sizedqueue"
# run_spec "$SPEC_DIR/core/threadgroup"

# 4.47 - 4.55: Other
# run_spec "$SPEC_DIR/core/gc"
# run_spec "$SPEC_DIR/core/objectspace"
# run_spec "$SPEC_DIR/core/random"
# run_spec "$SPEC_DIR/core/time"
# run_spec "$SPEC_DIR/core/process"
# run_spec "$SPEC_DIR/core/marshal"
# run_spec "$SPEC_DIR/core/tracepoint"
# run_spec "$SPEC_DIR/core/refinement"
# run_spec "$SPEC_DIR/core/builtin_constants"

# ──────────────────────────────────────────────────────────────────────────
# Parallel execution. mspec invocations are independent processes, so we
# fan them out via xargs and rejoin them in queue order for output.
# ──────────────────────────────────────────────────────────────────────────

if [ "$SPEC_COUNT" -eq 0 ]; then
  exit 0
fi

# Worker script. xargs invokes it as `worker <idx> <spec>`. mspec output is
# tee'd to "$WORK_DIR/<idx>.out" (consulted later for the summary line) AND
# streamed through `grep --line-buffered` to stdout, stripping the per-spec
# framing noise — empty lines, the `$ /path/...` command echo, the
# `metorex (ruby-compatible)` banner, the `Finished in …` timing, and the
# `N files, …` summary. What's left is the dot/E/F progress lines and any
# failure detail — in real time as each spec finishes. (Mspec emits all dots
# on a single line, so they appear together when that spec completes; one
# spec's dot line per spec arrival, not per individual test.)
export MSPEC METOREX WORK_DIR
WORKER="$WORK_DIR/worker.sh"
cat > "$WORKER" <<'WORKER_EOF'
#!/usr/bin/env bash
idx=$1
spec=$2
"$MSPEC" -t "$METOREX" "$spec" 2>&1 \
  | tee "$WORK_DIR/$idx.out" \
  | grep --line-buffered -v -E '^$|^\$ |^metorex \(ruby-compatible\)$|^Finished in [0-9.]+ seconds$|^[0-9]+ files?, [0-9]+ examples?, [0-9]+ expectations?, [0-9]+ failures?, [0-9]+ errors?, [0-9]+ tagged$' \
  || true
echo "${PIPESTATUS[0]}" > "$WORK_DIR/$idx.code"
WORKER_EOF
chmod +x "$WORKER"

# Feed null-delimited (idx, path) pairs to xargs so workers don't have to
# split arguments themselves and paths with spaces stay intact.
{
  i=0
  while [ "$i" -lt "$SPEC_COUNT" ]; do
    printf '%s\0%s\0' "$i" "${SPEC_PATHS[$i]}"
    i=$((i + 1))
  done
} | xargs -0 -n 2 -P "$JOBS" "$WORKER"

# Workers already streamed their output. Walk the .out files silently to
# fold summary lines into the running totals and detect failures.
i=0
while [ "$i" -lt "$SPEC_COUNT" ]; do
  out_file="$WORK_DIR/$i.out"
  code_file="$WORK_DIR/$i.code"
  if [ -f "$code_file" ]; then
    code=$(cat "$code_file")
    if [ "$code" != "0" ]; then
      ANY_FAILED=1
    fi
  fi
  if [ -f "$out_file" ]; then
    summary=$(grep -E '^[0-9]+ files?, [0-9]+ examples?, [0-9]+ expectations?, [0-9]+ failures?, [0-9]+ errors?, [0-9]+ tagged$' "$out_file" | tail -1)
    if [ -n "$summary" ]; then
      read -r f _ e _ ex _ fl _ er _ tg _ <<< "$summary"
      TOTAL_FILES=$((TOTAL_FILES + f))
      TOTAL_EXAMPLES=$((TOTAL_EXAMPLES + e))
      TOTAL_EXPECTATIONS=$((TOTAL_EXPECTATIONS + ex))
      TOTAL_FAILURES=$((TOTAL_FAILURES + fl))
      TOTAL_ERRORS=$((TOTAL_ERRORS + er))
      TOTAL_TAGGED=$((TOTAL_TAGGED + tg))
    fi
  fi
  i=$((i + 1))
done

if [ "$ANY_FAILED" != "0" ]; then
  exit 1
fi
