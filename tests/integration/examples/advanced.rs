use super::run_example;

// 10.4.11 — Advanced Features (partial — traits.rb works)

#[test]
fn test_advanced_traits() {
    // traits.rb defines a module and class but produces no output
    // Verify it runs without error
    let output = run_example("advanced/traits.rb");
    assert_eq!(output, "");
}

#[test]
fn test_method_rescue_ensure_execution() {
    let expected = "rescued\nbody\nensure ran\ncaught\ncleanup\n";
    let output = run_example("advanced/method_rescue_ensure.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_method_rescue_ensure_parens_execution() {
    let expected = "rescued\nbody\nensure ran\ncaught\ncleanup\n";
    let output = run_example("advanced/method_rescue_ensure_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_block_as_arg() {
    let output = run_example("advanced/block_as_arg.rb");
    assert_eq!(output, "15\n");
}

#[test]
fn test_block_param_inspect() {
    let output = run_example("advanced/block_param_inspect.rb");
    assert_eq!(output, "NilClass\ntrue\n");
}

#[test]
fn test_block_param_nil() {
    let output = run_example("advanced/block_param_nil.rb");
    assert_eq!(output, "no block\n84\n");
}

#[test]
fn test_case_in_coverage() {
    let output = run_example("advanced/case_in_coverage.rb");
    assert_eq!(output, "one\ntwo\none\ntwo\n");
}

#[test]
fn test_case_when_value() {
    let output = run_example("advanced/case_when_value.rb");
    assert_eq!(output, "A\nB\nC\nF\n");
}

#[test]
fn test_multi_bracket_args() {
    let output = run_example("advanced/multi_bracket_args.rb");
    assert_eq!(output, "1,2,3\n");
}

#[test]
fn test_parenless_splat() {
    let output = run_example("advanced/parenless_splat.rb");
    assert_eq!(output, "4\n1\n2\n");
}

#[test]
fn test_stabby_expr_body() {
    let output = run_example("advanced/stabby_expr_body.rb");
    assert_eq!(output, "5\n");
}

/// The expected output of both `advanced/threads_that_wait` variants, which
/// differ only in whether the calls are written with parentheses.
const THREADS_THAT_WAIT_OUTPUT: &str = concat!(
    "\"heard \\\"ping\\\\r\\\\n\\\"\"\n",
    "\"head\\r\\n\"\n",
    "\"tail\"\n",
);

#[test]
fn test_advanced_threads_that_wait_execution() {
    let output = run_example("advanced/threads_that_wait.rb");
    assert_eq!(output, THREADS_THAT_WAIT_OUTPUT);
}

#[test]
fn test_advanced_threads_that_wait_no_parens_execution() {
    let output = run_example("advanced/threads_that_wait_no_parens.rb");
    assert_eq!(output, THREADS_THAT_WAIT_OUTPUT);
}

/// The expected output of both `advanced/threads_that_wake` variants, which
/// differ only in whether the calls are written with parentheses.
const THREADS_THAT_WAKE_OUTPUT: &str = concat!(
    "\"sleep\"\n",
    ":ready\n",
    "false\n",
    "false\n",
    "[:woken]\n",
    "[true]\n",
    "false\n",
);

#[test]
fn test_advanced_threads_that_wake_execution() {
    let output = run_example("advanced/threads_that_wake.rb");
    assert_eq!(output, THREADS_THAT_WAKE_OUTPUT);
}

#[test]
fn test_advanced_threads_that_wake_no_parens_execution() {
    let output = run_example("advanced/threads_that_wake_no_parens.rb");
    assert_eq!(output, THREADS_THAT_WAKE_OUTPUT);
}

/// The expected output of both `advanced/queues_that_wait` variants, which
/// differ only in whether the calls are written with parentheses.
const QUEUES_THAT_WAIT_OUTPUT: &str = concat!(
    "nil\n",
    ":first\n",
    "\"queue empty\"\n",
    "1\n",
    ":one\n",
    ":two\n",
);

#[test]
fn test_advanced_queues_that_wait_execution() {
    let output = run_example("advanced/queues_that_wait.rb");
    assert_eq!(output, QUEUES_THAT_WAIT_OUTPUT);
}

#[test]
fn test_advanced_queues_that_wait_no_parens_execution() {
    let output = run_example("advanced/queues_that_wait_no_parens.rb");
    assert_eq!(output, QUEUES_THAT_WAIT_OUTPUT);
}

/// The expected output of both `advanced/fibers_that_schedule` variants, which
/// differ only in whether the calls are written with parentheses.
const FIBERS_THAT_SCHEDULE_OUTPUT: &str = concat!(
    "nil\n",
    "\"Scheduler must implement #io_wait\"\n",
    "true\n",
    "true\n",
    "nil\n",
    "nil\n",
    "[true, \"later\"]\n",
);

#[test]
fn test_advanced_fibers_that_schedule_execution() {
    let output = run_example("advanced/fibers_that_schedule.rb");
    assert_eq!(output, FIBERS_THAT_SCHEDULE_OUTPUT);
}

#[test]
fn test_advanced_fibers_that_schedule_no_parens_execution() {
    let output = run_example("advanced/fibers_that_schedule_no_parens.rb");
    assert_eq!(output, FIBERS_THAT_SCHEDULE_OUTPUT);
}
