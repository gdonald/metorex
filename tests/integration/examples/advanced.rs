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

#[test]
fn test_advanced_threads_take_turns_execution() {
    let output = run_example("advanced/threads_take_turns.rb");
    assert_eq!(output, "true\ntrue\n");
}

#[test]
fn test_advanced_threads_take_turns_no_parens_execution() {
    let output = run_example("advanced/threads_take_turns_no_parens.rb");
    assert_eq!(output, "true\ntrue\n");
}

const THREADS_THAT_DEADLOCK: &str = "fatal\nNo live threads left. Deadlock?\n:locked\n\"stopping only thread\\n\\tnote: use sleep to stop forever\"\nmain woken\nfalse\ntrue\n";
const DEADLOCK_RAISED: &str = "fatal\nNo live threads left. Deadlock?\n";

#[test]
fn test_advanced_threads_that_deadlock_execution() {
    let output = run_example("advanced/threads_that_deadlock.rb");
    assert_eq!(output, THREADS_THAT_DEADLOCK);
}

#[test]
fn test_advanced_threads_that_deadlock_no_parens_execution() {
    let output = run_example("advanced/threads_that_deadlock_no_parens.rb");
    assert_eq!(output, THREADS_THAT_DEADLOCK);
}

#[test]
fn test_advanced_queues_that_deadlock_execution() {
    let output = run_example("advanced/queues_that_deadlock.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_queues_that_deadlock_no_parens_execution() {
    let output = run_example("advanced/queues_that_deadlock_no_parens.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_condition_variables_that_deadlock_execution() {
    let output = run_example("advanced/condition_variables_that_deadlock.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_condition_variables_that_deadlock_no_parens_execution() {
    let output = run_example("advanced/condition_variables_that_deadlock_no_parens.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_threads_that_stop_together_execution() {
    let output = run_example("advanced/threads_that_stop_together.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_threads_that_stop_together_no_parens_execution() {
    let output = run_example("advanced/threads_that_stop_together_no_parens.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_threads_keep_their_class_variables_execution() {
    let output = run_example("advanced/threads_keep_their_class_variables.rb");
    assert_eq!(output, "\"reader\"\n\"worker\"\n");
}

#[test]
fn test_advanced_threads_keep_their_class_variables_no_parens_execution() {
    let output = run_example("advanced/threads_keep_their_class_variables_no_parens.rb");
    assert_eq!(output, "\"reader\"\n\"worker\"\n");
}

const THREADS_WAIT_ON_STREAMS: &str = concat!(
    "IO#read: true \"hi\\n\"\n",
    "IO#read with a length: true \"hi\"\n",
    "IO#readpartial: true \"hi\\n\"\n",
    "IO#sysread: true \"hi\\n\"\n",
    "IO#gets: true \"hi\\n\"\n",
    "IO#getc: true \"h\"\n",
    "IO#getbyte: true 104\n",
    "IO#readchar: true \"h\"\n",
    "IO#readbyte: true 104\n",
    "IO#readline: true \"hi\\n\"\n",
    "IO#readlines: true [\"a\\n\", \"b\\n\"]\n",
    "IO#each_line: true [\"a\\n\", \"b\\n\"]\n",
    "IO#each_byte: true [97, 98]\n",
    "IO#each_char: true [\"a\", \"b\"]\n",
    "IO#wait_readable: true IO\n",
    "IO#wait: true IO\n",
    "IO.select: true 1\n",
    "Kernel#select: true 1\n",
    "IO.copy_stream: true 3\n",
    "Kernel#gets: true \"hi\\n\"\n",
    "Kernel#readline: true \"hi\\n\"\n",
    "Kernel#readlines: true [\"a\", \"b\"]\n",
    "IO#write: true 100000\n",
    "IO#syswrite: true true\n",
    "IO#wait_writable: true IO\n",
    "\"h\"\n",
    "100000\n",
);

#[test]
fn test_advanced_threads_wait_on_streams_execution() {
    let output = run_example("advanced/threads_wait_on_streams.rb");
    assert_eq!(output, THREADS_WAIT_ON_STREAMS);
}

#[test]
fn test_advanced_threads_wait_on_streams_no_parens_execution() {
    let output = run_example("advanced/threads_wait_on_streams_no_parens.rb");
    assert_eq!(output, THREADS_WAIT_ON_STREAMS);
}

const THREADS_WAIT_ON_PROCESSES_AND_SOCKETS: &str = concat!(
    "Kernel#sleep: true Integer\n",
    "Kernel#system: true true\n",
    "Kernel#`: true \"hi\\n\"\n",
    "IO.popen: true \"hi\\n\"\n",
    "IO.popen writing: true 200000\n",
    "Process.wait: true Integer\n",
    "Process.wait2: true true\n",
    "Process.waitpid: true Integer\n",
    "Process.waitpid2: true true\n",
    "Process.waitall: true 1\n",
    "Process::Status.wait: true true\n",
    "File#flock: true 0\n",
    "File.open on a FIFO: true \"hi\\n\"\n",
    "File.read on a FIFO: true \"hi\\n\"\n",
    "File.write on a FIFO: true 3\n",
    "partway\n",
    "alone\n",
    "UNIXServer#accept: true UNIXSocket\n",
    "TCPServer#accept: true TCPSocket\n",
    "BasicSocket#recv: true \"hi\"\n",
    "BasicSocket#recvmsg: true \"hi\"\n",
    "UDPSocket#recvfrom: true \"hi\"\n",
    "Socket#accept: true Socket\n",
    "[\"127.0.0.1\"]\n",
    "TCPSocket\n",
    "Socket\n",
);

#[test]
fn test_advanced_threads_wait_on_processes_and_sockets_execution() {
    let output = run_example("advanced/threads_wait_on_processes_and_sockets.rb");
    assert_eq!(output, THREADS_WAIT_ON_PROCESSES_AND_SOCKETS);
}

#[test]
fn test_advanced_threads_wait_on_processes_and_sockets_no_parens_execution() {
    let output = run_example("advanced/threads_wait_on_processes_and_sockets_no_parens.rb");
    assert_eq!(output, THREADS_WAIT_ON_PROCESSES_AND_SOCKETS);
}

const THREADS_LEAVE_THEIR_WAITS: &str = concat!(
    "\"IOError: stream closed in another thread\"\n",
    "\"IOError: stream closed in another thread\"\n",
    ":resumed\n",
    "\"stopped the wait\"\n",
    "true\n",
);

#[test]
fn test_advanced_threads_leave_their_waits_execution() {
    let output = run_example("advanced/threads_leave_their_waits.rb");
    assert_eq!(output, THREADS_LEAVE_THEIR_WAITS);
}

#[test]
fn test_advanced_threads_leave_their_waits_no_parens_execution() {
    let output = run_example("advanced/threads_leave_their_waits_no_parens.rb");
    assert_eq!(output, THREADS_LEAVE_THEIR_WAITS);
}

#[test]
fn test_advanced_mutexes_that_deadlock_execution() {
    let output = run_example("advanced/mutexes_that_deadlock.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_mutexes_that_deadlock_no_parens_execution() {
    let output = run_example("advanced/mutexes_that_deadlock_no_parens.rb");
    assert_eq!(output, DEADLOCK_RAISED);
}

#[test]
fn test_advanced_threads_that_die_while_waited_on_execution() {
    let output = run_example("advanced/threads_that_die_while_waited_on.rb");
    assert_eq!(output, "{[RuntimeError, \"\"] => 20}\n");
}

#[test]
fn test_advanced_threads_that_die_while_waited_on_no_parens_execution() {
    let output = run_example("advanced/threads_that_die_while_waited_on_no_parens.rb");
    assert_eq!(output, "{[RuntimeError, \"\"] => 20}\n");
}

const NATIVE_THREAD_IDS: &str = concat!(
    "Integer\n",
    "true\n",
    "true\n",
    "true\n",
    "nil\n",
    "Integer\n",
    "nil\n",
    "true\n",
    ":ran\n",
    "true\n",
);

#[test]
fn test_advanced_native_thread_ids_execution() {
    let output = run_example("advanced/native_thread_ids.rb");
    assert_eq!(output, NATIVE_THREAD_IDS);
}

#[test]
fn test_advanced_native_thread_ids_no_parens_execution() {
    let output = run_example("advanced/native_thread_ids_no_parens.rb");
    assert_eq!(output, NATIVE_THREAD_IDS);
}
