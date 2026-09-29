// The options Hash `Process.spawn` and `Process.exec` take.

use super::*;

/// Run a child with the options given and answer what it wrote to stdout.
fn child_output(command: &str, options: &str) -> Option<Object> {
    run(&format!(
        "reader, writer = IO.pipe; \
         pid = Process.spawn({command:?}, out: writer, {options}); \
         writer.close; \
         Process.wait(pid); \
         reader.read"
    ))
}

#[test]
fn spawn_sets_the_limits_named_by_rlimit_options() {
    let output = child_output(
        "ulimit -n; ulimit -c",
        "rlimit_nofile: 64, rlimit_core: [0, 0]",
    );
    assert_eq!(output, Some(Object::string("64\n0\n")));
}

#[test]
fn spawn_sets_the_umask_named() {
    let output = child_output("umask; true", "umask: 0o027");
    assert_eq!(output, Some(Object::string("0027\n")));
}

#[test]
fn spawn_runs_the_child_under_the_ids_named() {
    let output = child_output("id -u; id -g", "uid: Process.uid, gid: Process.gid");
    let expected = format!("{}\n{}\n", unsafe { libc::getuid() }, unsafe {
        libc::getgid()
    });
    assert_eq!(output, Some(Object::string(expected)));
}

#[test]
fn spawn_starts_the_child_in_the_process_group_named() {
    let output = run("reader, writer = IO.pipe; \
         pid = Process.spawn(\"ps -o pgid= -p $$\", out: writer, pgroup: true); \
         writer.close; \
         Process.wait(pid); \
         reader.read.strip.to_i == pid");
    assert_eq!(output, Some(Object::Bool(true)));
}

#[test]
fn spawn_redirects_through_an_object_answering_to_io() {
    let output = run("reader, writer = IO.pipe; \
         wrapped = Object.new; \
         wrapped.define_singleton_method(:to_io) { writer }; \
         pid = Process.spawn(\"echo wrapped\", out: wrapped); \
         writer.close; \
         Process.wait(pid); \
         reader.read");
    assert_eq!(output, Some(Object::string("wrapped\n")));
}

#[test]
fn spawn_redirects_a_stream_named_by_an_io_key_to_a_file_opened_with_a_mode() {
    let output = run("path = \"/tmp/metorex_spawn_mode_#{Process.pid}.txt\"; \
         File.write(path, \"first\\n\"); \
         pid = Process.spawn(\"echo second\", STDOUT => [path, \"a\"]); \
         Process.wait(pid); \
         written = File.read(path); \
         File.delete(path); \
         written");
    assert_eq!(output, Some(Object::string("first\nsecond\n")));
}

#[test]
fn spawn_sends_a_stream_to_one_of_this_process_named_by_symbol() {
    let output = run("reader, writer = IO.pipe; \
         pid = Process.spawn(\"echo to_err >&2\", out: writer, err: :out); \
         writer.close; \
         Process.wait(pid); \
         reader.read.empty?");
    assert_eq!(output, Some(Object::Bool(true)));
}

#[test]
fn spawn_closes_descriptors_left_open_when_close_others_is_true() {
    let output = run("reader, writer = IO.pipe; \
         writer.close_on_exec = false; \
         pid = Process.spawn(\"sleep 0.2\", close_others: true); \
         writer.close; \
         ended = reader.read; \
         Process.wait(pid); \
         ended");
    assert_eq!(output, Some(Object::string("")));
}

#[test]
fn spawn_refuses_a_redirect_symbol_it_does_not_know() {
    let error = run_err("Process.spawn(\"true\", [:out, :nowhere] => \"/dev/null\")");
    assert!(
        error.contains("wrong exec redirect symbol: nowhere"),
        "{}",
        error
    );
}

#[test]
fn spawn_refuses_a_redirect_action_it_does_not_know() {
    let error = run_err("Process.spawn(\"true\", out: :nowhere)");
    assert!(error.contains("wrong exec redirect action"), "{}", error);
}

#[test]
fn spawn_refuses_a_child_redirect_that_names_no_stream() {
    let error = run_err("Process.spawn(\"true\", err: [:child])");
    assert!(error.contains("wrong exec redirect action"), "{}", error);
}

#[test]
fn spawn_refuses_a_redirect_key_that_is_no_stream() {
    let error = run_err("Process.spawn(\"true\", Object.new => \"/dev/null\")");
    assert!(error.contains("wrong exec option"), "{}", error);
}

#[test]
fn spawn_refuses_an_environment_value_holding_a_null_byte() {
    let error = run_err("Process.spawn({ \"NAME\" => \"a\\0b\" }, \"true\")");
    assert!(error.contains("string contains null byte"), "{}", error);
}

#[test]
fn standard_streams_open_at_start_are_left_open() {
    metorex::standard_streams::close_the_ones_closed_at_start();
    // SAFETY: `fcntl` with F_GETFD only reads the descriptor's flags.
    let open = (0..3).all(|descriptor| unsafe { libc::fcntl(descriptor, libc::F_GETFD) } >= 0);
    assert!(open);
}
