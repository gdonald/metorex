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
