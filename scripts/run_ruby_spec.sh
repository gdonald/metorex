#!/usr/bin/env bash

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
METOREX="$REPO_ROOT/target/debug/metorex"
MSPEC="$REPO_ROOT/ruby/mspec/bin/mspec"
SPEC_DIR="$REPO_ROOT/ruby/spec"

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
# run_spec "$SPEC_DIR/core/module/comparison_spec.rb"
# run_spec "$SPEC_DIR/core/module/const_added_spec.rb"
# run_spec "$SPEC_DIR/core/module/const_defined_spec.rb"
# run_spec "$SPEC_DIR/core/module/const_get_spec.rb"
# run_spec "$SPEC_DIR/core/module/const_missing_spec.rb"
# run_spec "$SPEC_DIR/core/module/const_set_spec.rb"
# run_spec "$SPEC_DIR/core/module/const_source_location_spec.rb"
# run_spec "$SPEC_DIR/core/module/constants_spec.rb"
# run_spec "$SPEC_DIR/core/module/define_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/define_singleton_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/deprecate_constant_spec.rb"
# run_spec "$SPEC_DIR/core/module/eql_spec.rb"
# run_spec "$SPEC_DIR/core/module/equal_spec.rb"
# run_spec "$SPEC_DIR/core/module/equal_value_spec.rb"
# run_spec "$SPEC_DIR/core/module/extend_object_spec.rb"
# run_spec "$SPEC_DIR/core/module/extended_spec.rb"
# run_spec "$SPEC_DIR/core/module/freeze_spec.rb"
# run_spec "$SPEC_DIR/core/module/gt_spec.rb"
# run_spec "$SPEC_DIR/core/module/gte_spec.rb"
# run_spec "$SPEC_DIR/core/module/include_spec.rb"
# run_spec "$SPEC_DIR/core/module/included_modules_spec.rb"
# run_spec "$SPEC_DIR/core/module/included_spec.rb"
# run_spec "$SPEC_DIR/core/module/initialize_copy_spec.rb"
# run_spec "$SPEC_DIR/core/module/initialize_spec.rb"
# run_spec "$SPEC_DIR/core/module/instance_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/instance_methods_spec.rb"
# run_spec "$SPEC_DIR/core/module/lt_spec.rb"
# run_spec "$SPEC_DIR/core/module/lte_spec.rb"
# run_spec "$SPEC_DIR/core/module/method_added_spec.rb"
# run_spec "$SPEC_DIR/core/module/method_defined_spec.rb"
# run_spec "$SPEC_DIR/core/module/method_removed_spec.rb"
# run_spec "$SPEC_DIR/core/module/method_undefined_spec.rb"
# run_spec "$SPEC_DIR/core/module/module_eval_spec.rb"
# run_spec "$SPEC_DIR/core/module/module_exec_spec.rb"
# run_spec "$SPEC_DIR/core/module/module_function_spec.rb"
# run_spec "$SPEC_DIR/core/module/name_spec.rb"
# run_spec "$SPEC_DIR/core/module/nesting_spec.rb"
# run_spec "$SPEC_DIR/core/module/new_spec.rb"
# run_spec "$SPEC_DIR/core/module/prepend_features_spec.rb"
# run_spec "$SPEC_DIR/core/module/prepend_spec.rb"
# run_spec "$SPEC_DIR/core/module/prepended_spec.rb"
# run_spec "$SPEC_DIR/core/module/private_class_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/private_constant_spec.rb"
# run_spec "$SPEC_DIR/core/module/private_instance_methods_spec.rb"
# run_spec "$SPEC_DIR/core/module/private_method_defined_spec.rb"
# run_spec "$SPEC_DIR/core/module/private_spec.rb"
# run_spec "$SPEC_DIR/core/module/protected_instance_methods_spec.rb"
# run_spec "$SPEC_DIR/core/module/protected_method_defined_spec.rb"
# run_spec "$SPEC_DIR/core/module/protected_spec.rb"
# run_spec "$SPEC_DIR/core/module/public_class_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/public_constant_spec.rb"
# run_spec "$SPEC_DIR/core/module/public_instance_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/public_instance_methods_spec.rb"
# run_spec "$SPEC_DIR/core/module/public_method_defined_spec.rb"
# run_spec "$SPEC_DIR/core/module/public_spec.rb"
# run_spec "$SPEC_DIR/core/module/refine_spec.rb"
# run_spec "$SPEC_DIR/core/module/refinements_spec.rb"
# run_spec "$SPEC_DIR/core/module/remove_class_variable_spec.rb"
# run_spec "$SPEC_DIR/core/module/remove_const_spec.rb"
# run_spec "$SPEC_DIR/core/module/remove_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/ruby2_keywords_spec.rb"
# run_spec "$SPEC_DIR/core/module/set_temporary_name_spec.rb"
# run_spec "$SPEC_DIR/core/module/singleton_class_spec.rb"
# run_spec "$SPEC_DIR/core/module/to_s_spec.rb"
# run_spec "$SPEC_DIR/core/module/undef_method_spec.rb"
# run_spec "$SPEC_DIR/core/module/undefined_instance_methods_spec.rb"
# run_spec "$SPEC_DIR/core/module/used_refinements_spec.rb"
# run_spec "$SPEC_DIR/core/module/using_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/Array_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/Complex_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/Float_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/Hash_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/Integer_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/Rational_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/String_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/__callee___spec.rb"
# run_spec "$SPEC_DIR/core/kernel/__dir___spec.rb"
# run_spec "$SPEC_DIR/core/kernel/__method___spec.rb"
# run_spec "$SPEC_DIR/core/kernel/abort_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/at_exit_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/autoload_relative_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/autoload_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/backtick_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/binding_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/block_given_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/caller_locations_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/caller_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/case_compare_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/catch_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/chomp_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/chop_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/class_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/clone_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/comparison_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/define_singleton_method_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/display_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/dup_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/enum_for_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/eql_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/equal_value_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/eval_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/exec_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/exit_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/extend_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/fail_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/fork_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/format_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/freeze_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/frozen_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/gets_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/global_variables_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/gsub_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/initialize_clone_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/initialize_copy_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/initialize_dup_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/inspect_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/instance_of_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/instance_variable_defined_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/instance_variable_get_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/instance_variable_set_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/instance_variables_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/is_a_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/itself_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/kind_of_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/lambda_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/load_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/local_variables_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/loop_spec.rb"
run_spec "$SPEC_DIR/core/kernel/match_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/method_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/methods_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/nil_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/not_match_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/object_id_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/open_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/p_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/pp_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/print_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/printf_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/private_methods_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/proc_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/protected_methods_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/public_method_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/public_methods_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/public_send_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/putc_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/puts_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/raise_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/rand_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/readline_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/readlines_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/remove_instance_variable_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/require_relative_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/require_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/respond_to_missing_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/respond_to_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/select_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/send_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/set_trace_func_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/singleton_class_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/singleton_method_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/singleton_methods_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/sleep_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/spawn_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/sprintf_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/srand_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/sub_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/syscall_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/system_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/taint_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/tainted_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/tap_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/test_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/then_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/throw_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/to_enum_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/to_s_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/trace_var_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/trap_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/trust_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/untaint_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/untrace_var_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/untrust_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/untrusted_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/warn_spec.rb"
# run_spec "$SPEC_DIR/core/kernel/yield_self_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/__id__spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/__send___spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/basicobject_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/equal_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/equal_value_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/initialize_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/instance_eval_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/instance_exec_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/method_missing_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/not_equal_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/not_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/singleton_method_added_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/singleton_method_removed_spec.rb"
# run_spec "$SPEC_DIR/core/basicobject/singleton_method_undefined_spec.rb"

# 4.10 - 4.13: Exceptions and Signals
# run_spec "$SPEC_DIR/core/exception"
# run_spec "$SPEC_DIR/core/systemexit"
# run_spec "$SPEC_DIR/core/signal"
# run_spec "$SPEC_DIR/core/warning"

# 4.14 - 4.19: Numeric Types
# run_spec "$SPEC_DIR/core/integer"
# run_spec "$SPEC_DIR/core/float"
# run_spec "$SPEC_DIR/core/numeric"
# run_spec "$SPEC_DIR/core/complex"
# run_spec "$SPEC_DIR/core/rational"
# run_spec "$SPEC_DIR/core/math"

# 4.20 - 4.25: Collections
# run_spec "$SPEC_DIR/core/array"
# run_spec "$SPEC_DIR/core/hash"
# run_spec "$SPEC_DIR/core/set"
# run_spec "$SPEC_DIR/core/range"
# run_spec "$SPEC_DIR/core/struct"
# run_spec "$SPEC_DIR/core/data"

# 4.26 - 4.30: Strings and Patterns
# run_spec "$SPEC_DIR/core/string"
# run_spec "$SPEC_DIR/core/symbol"
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
