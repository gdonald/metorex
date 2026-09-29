use super::run_example;

#[test]
fn test_threads_thread_locals_across_threads_execution() {
    let expected = ":saw_it\n:done\ntrue\n";
    let output = run_example("threads/thread_locals_across_threads.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_threads_autoload_one_at_a_time_execution() {
    let expected = "[:pre, :post, :first_done, :second_done]\n[1, 1]\n";
    let output = run_example("threads/autoload_one_at_a_time.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_threads_condition_variable_barrier_execution() {
    let expected = "[1, 2, 3, 4]\n[:last, :woken, :woken, :woken]\n4\n";
    let output = run_example("threads/condition_variable_barrier.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_threads_reloading_a_required_file_execution() {
    let expected = "1\n1\ntrue\nfalse\n2\n";
    let output = run_example("threads/reloading_a_required_file.rb");
    assert_eq!(output, expected);
}

const AUTOLOAD_UNDER_LOAD: &str = "true\n3\n[:ready, :ready, :ready]\n";

#[test]
fn test_threads_autoload_under_load_execution() {
    let output = run_example("threads/autoload_under_load.rb");
    assert_eq!(output, AUTOLOAD_UNDER_LOAD);
}

#[test]
fn test_threads_autoload_under_load_no_parens_execution() {
    let output = run_example("threads/autoload_under_load_no_parens.rb");
    assert_eq!(output, AUTOLOAD_UNDER_LOAD);
}

const EXITING_FROM_A_THREAD: &str =
    "[:in_the_thread, :in_the_main_thread, 42]\n\"kept to itself\"\n";

#[test]
fn test_threads_exiting_from_a_thread_execution() {
    let output = run_example("threads/exiting_from_a_thread.rb");
    assert_eq!(output, EXITING_FROM_A_THREAD);
}

#[test]
fn test_threads_exiting_from_a_thread_no_parens_execution() {
    let output = run_example("threads/exiting_from_a_thread_no_parens.rb");
    assert_eq!(output, EXITING_FROM_A_THREAD);
}

const SLEEPING: &str = concat!(
    "true\ntrue\ntrue\n",
    "\"time interval must not be negative\"\n",
    "[[0.01], []]\nnil\n"
);

#[test]
fn test_threads_sleeping_execution() {
    let output = run_example("threads/sleeping.rb");
    assert_eq!(output, SLEEPING);
}

#[test]
fn test_threads_sleeping_no_parens_execution() {
    let output = run_example("threads/sleeping_no_parens.rb");
    assert_eq!(output, SLEEPING);
}

const ENDING_WITH_THREADS_ALIVE: &str = "the waiting fiber unwinds\n";

#[test]
fn test_threads_ending_with_threads_alive_execution() {
    let output = run_example("threads/ending_with_threads_alive.rb");
    assert_eq!(output, ENDING_WITH_THREADS_ALIVE);
}

#[test]
fn test_threads_ending_with_threads_alive_no_parens_execution() {
    let output = run_example("threads/ending_with_threads_alive_no_parens.rb");
    assert_eq!(output, ENDING_WITH_THREADS_ALIVE);
}

/// The expected output of both `threads/raising_into_threads` variants.
const RAISING_INTO_THREADS_OUTPUT: &str = concat!(
    "[ArgumentError, \"at once\"]\n",
    "[TypeError, \"exception class/object expected\"]\n",
    "nil\n",
    "nil\n",
    "\"handed over\"\n",
    "nil\n",
    "\"with a message\"\n",
    "[[\"with a message\"], []]\n",
    "\"in a block\"\n",
    "nil\n",
    "\"outer\"\n"
);

#[test]
fn test_threads_raising_into_threads_execution() {
    let output = run_example("threads/raising_into_threads.rb");
    assert_eq!(output, RAISING_INTO_THREADS_OUTPUT);
}

#[test]
fn test_threads_raising_into_threads_parens_execution() {
    let output = run_example("threads/raising_into_threads_parens.rb");
    assert_eq!(output, RAISING_INTO_THREADS_OUTPUT);
}

/// The expected output of `threads/interrupted_backtraces`, where the file
/// name each backtrace entry carries is the example's own.
fn interrupted_backtraces_output(file: &str) -> String {
    format!(
        "[\"{file}:11:in 'Kernel#sleep'\", \"{file}:11:in 'block (2 levels) in <main>'\"]\n\
         [\"somewhere:1\"]\n\
         \"a\\\\#1\"\n\
         true\n\
         true\n"
    )
}

#[test]
fn test_threads_interrupted_backtraces_execution() {
    let output = run_example("threads/interrupted_backtraces.rb");
    assert_eq!(
        output,
        interrupted_backtraces_output("interrupted_backtraces.rb")
    );
}

#[test]
fn test_threads_interrupted_backtraces_parens_execution() {
    let output = run_example("threads/interrupted_backtraces_parens.rb");
    assert_eq!(
        output,
        interrupted_backtraces_output("interrupted_backtraces_parens.rb")
    );
}

/// The expected output of both `threads/locking_from_two_fibers` variants.
const LOCKING_FROM_TWO_FIBERS_OUTPUT: &str = concat!(
    "\"deadlock; lock already owned by another fiber belonging to the same thread\"\n",
    "\"deadlock; lock already owned by another fiber belonging to the same thread\"\n",
    "false\n",
);

#[test]
fn test_threads_locking_from_two_fibers_execution() {
    let output = run_example("threads/locking_from_two_fibers.rb");
    assert_eq!(output, LOCKING_FROM_TWO_FIBERS_OUTPUT);
}

#[test]
fn test_threads_locking_from_two_fibers_no_parens_execution() {
    let output = run_example("threads/locking_from_two_fibers_no_parens.rb");
    assert_eq!(output, LOCKING_FROM_TWO_FIBERS_OUTPUT);
}

/// The expected output of both `threads/interrupting_a_locking_fiber`
/// variants.
const INTERRUPTING_A_LOCKING_FIBER_OUTPUT: &str = concat!("true\n", "false\n");

#[test]
fn test_threads_interrupting_a_locking_fiber_execution() {
    let output = run_example("threads/interrupting_a_locking_fiber.rb");
    assert_eq!(output, INTERRUPTING_A_LOCKING_FIBER_OUTPUT);
}

#[test]
fn test_threads_interrupting_a_locking_fiber_no_parens_execution() {
    let output = run_example("threads/interrupting_a_locking_fiber_no_parens.rb");
    assert_eq!(output, INTERRUPTING_A_LOCKING_FIBER_OUTPUT);
}

#[test]
fn test_threads_signaling_a_child_a_thread_waits_on_execution() {
    let output = run_example("threads/signaling_a_child_a_thread_waits_on.rb");
    assert_eq!(output, "\"signaled\\n\"\n");
}

#[test]
fn test_threads_signaling_a_child_a_thread_waits_on_no_parens_execution() {
    let output = run_example("threads/signaling_a_child_a_thread_waits_on_no_parens.rb");
    assert_eq!(output, "\"signaled\\n\"\n");
}
