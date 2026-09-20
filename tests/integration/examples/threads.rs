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
